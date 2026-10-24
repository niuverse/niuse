//! Write-time policy: secret scanning and near-duplicate detection.
//!
//! Both are deliberately conservative. A false positive costs one `[f]orce`
//! keystroke; a false negative puts a credential in every future request.

use std::collections::HashSet;
use std::sync::OnceLock;

use regex::Regex;

/// Similarity at which two entries are treated as the same claim.
pub const TEXT_THRESHOLD: f64 = 0.5;
/// Lower bar when the entries share a tag: short entries rarely reach 0.5.
pub const TAG_THRESHOLD: f64 = 0.35;

/// Returns a human-readable reason when `text` looks like it carries a
/// credential. This is a backstop, not a guarantee.
pub fn secret_reason(text: &str) -> Option<&'static str> {
    patterns()
        .iter()
        .find(|(pattern, _)| pattern.is_match(text))
        .map(|(_, reason)| *reason)
}

fn patterns() -> &'static [(Regex, &'static str)] {
    static PATTERNS: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        let pattern = |source: &str| Regex::new(source).expect("valid built-in regex");
        vec![
            (pattern(r"sk-[A-Za-z0-9_-]{16,}"), "API key (sk-)"),
            (pattern(r"ghp_[A-Za-z0-9]{20,}"), "GitHub token (ghp_)"),
            (
                pattern(r"github_pat_[A-Za-z0-9_]{20,}"),
                "GitHub token (github_pat_)",
            ),
            (pattern(r"AKIA[0-9A-Z]{16}"), "AWS access key (AKIA)"),
            (
                pattern(r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
                "private key block",
            ),
            (
                pattern(r"xox[baprs]-[A-Za-z0-9-]{10,}"),
                "Slack token (xox)",
            ),
            (pattern(r"AIza[0-9A-Za-z_-]{20,}"), "Google API key (AIza)"),
            (
                pattern(r"\b[0-9a-fA-F]{32,}\b"),
                "long hex string (possible key or digest)",
            ),
        ]
    })
}

/// Character-bigram Jaccard similarity. Works for Chinese and English without
/// a tokenizer, and is cheap enough to run over a few hundred entries.
pub fn similarity(left: &str, right: &str) -> f64 {
    let left_normalized = normalized_chars(left);
    let right_normalized = normalized_chars(right);
    if left_normalized.is_empty() || right_normalized.is_empty() {
        return f64::from(left_normalized == right_normalized);
    }

    let left_shingles = shingles(&left_normalized);
    let right_shingles = shingles(&right_normalized);
    let intersection = left_shingles.intersection(&right_shingles).count() as f64;
    let union = left_shingles.union(&right_shingles).count() as f64;
    if union == 0.0 {
        return 0.0;
    }
    intersection / union
}

/// Rank a memory against a free-text query for `/mem recall`.
///
/// Substring matches and tag matches deliberately outrank bigram similarity:
/// a short query like "nextest" has too few bigrams to score well against a
/// long sentence, but an exact substring is an obviously good hit.
pub fn search_score(query: &str, text: &str, tags: &[String]) -> f64 {
    let raw_tokens: Vec<String> = query
        .split_whitespace()
        .map(normalize_string)
        .filter(|tok| !tok.is_empty())
        .collect();
    let query = normalize_string(query);
    if query.is_empty() {
        return 0.0;
    }

    let normalized_text = normalize_string(text);
    let normalized_tags: Vec<String> = tags.iter().map(|tag| normalize_string(tag)).collect();

    let text_score: f64 = if normalized_text.contains(&query) {
        0.9
    } else if raw_tokens.len() > 1
        && raw_tokens.iter().all(|tok| {
            normalized_text.contains(tok) || normalized_tags.iter().any(|t| t.contains(tok))
        })
    {
        0.85
    } else {
        0.0
    };

    let tag_score = normalized_tags
        .iter()
        .map(|tag| {
            if tag == &query {
                1.0
            } else if tag.contains(&query) {
                0.8
            } else {
                0.0
            }
        })
        .fold(0.0, f64::max);

    text_score.max(tag_score).max(similarity(&query, text))
}

pub fn conflicts(
    left_text: &str,
    left_tags: &[String],
    right_text: &str,
    right_tags: &[String],
) -> bool {
    let score = similarity(left_text, right_text);
    if score >= TEXT_THRESHOLD {
        return true;
    }
    let shared = left_tags
        .iter()
        .any(|tag| right_tags.iter().any(|other| other == tag));
    shared && score >= TAG_THRESHOLD
}

fn normalize_string(input: &str) -> String {
    input
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|character| character.is_alphanumeric())
        .collect()
}

fn normalized_chars(input: &str) -> Vec<char> {
    input
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|character| character.is_alphanumeric())
        .collect()
}

fn shingles(chars: &[char]) -> HashSet<(char, char)> {
    chars
        .windows(2)
        .map(|window| (window[0], window[1]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_common_key_shapes() {
        assert!(secret_reason("token sk-abcdefghijklmnopqrstuvwxyz").is_some());
        assert!(secret_reason("ghp_abcdefghijklmnopqrstuvwx").is_some());
        assert!(secret_reason("AKIAABCDEFGHIJKLMNOP").is_some());
        assert!(secret_reason("-----BEGIN RSA PRIVATE KEY-----").is_some());
        assert!(secret_reason("xoxb-1234567890-abcdef").is_some());
        assert!(secret_reason("0123456789abcdef0123456789abcdef").is_some());
    }

    #[test]
    fn ordinary_prose_is_not_flagged() {
        assert!(secret_reason("Rust 仓库一律用 cargo nextest").is_none());
        assert!(secret_reason("commit deadbeef").is_none());
        assert!(secret_reason("sk-短").is_none());
    }

    #[test]
    fn identical_claims_are_conflicts() {
        assert!(conflicts("用 nextest", &[], "用 nextest", &[]));
    }

    #[test]
    fn similar_claims_are_conflicts() {
        let left = "Rust 仓库一律使用 cargo nextest，不要用 cargo test";
        let right = "Rust 仓库一律用 cargo nextest，不用 cargo test";
        assert!(similarity(left, right) >= TEXT_THRESHOLD);
    }

    #[test]
    fn unrelated_claims_are_not_conflicts() {
        assert!(!conflicts("用 nextest 跑测试", &[], "先给结论再展开", &[]));
    }

    #[test]
    fn search_prefers_substrings_and_tags() {
        assert_eq!(
            search_score("nextest", "Rust 仓库一律用 cargo nextest", &[]),
            0.9
        );
        assert_eq!(
            search_score("rust", "用 cargo nextest", &["rust".into()]),
            1.0
        );
        assert_eq!(
            search_score("rust", "rust 仓库用 cargo nextest", &["rust".into()]),
            1.0,
            "exact tag match must not be shadowed by text substring match"
        );
        assert_eq!(
            search_score("rust nextest", "仓库一律用 cargo nextest", &["rust".into()]),
            0.85,
            "multi-token query should match across tags and text"
        );
        assert!(search_score("rust", "用 cargo nextest", &["rust-tooling".into()]) >= 0.8);
        assert!(search_score("kubernetes", "用 cargo nextest", &["rust".into()]) < 0.2);
    }

    #[test]
    fn shared_tag_lowers_the_bar() {
        let tags = vec!["rust".to_string()];
        assert!(conflicts(
            "Rust 项目用 nextest",
            &tags,
            "Rust 项目不要用 nextest",
            &tags
        ));
    }
}
