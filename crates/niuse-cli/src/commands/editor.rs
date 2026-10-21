//! Shared `$EDITOR` helpers.
//!
//! Memory text is normalised to one line; identity, user, and other Markdown
//! bodies keep their shape.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use niuse_core::model::one_line;

fn private_temp_file(initial: &str) -> Option<PathBuf> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or(0);
    let path = std::env::temp_dir().join(format!("niuse-edit-{}-{stamp}.md", std::process::id()));
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&path).ok()?;
    if file.write_all(initial.as_bytes()).is_err() {
        let _ = fs::remove_file(&path);
        return None;
    }
    Some(path)
}

fn editor_parts(editor: &str) -> (String, Vec<String>) {
    let mut parts = editor.split_whitespace();
    let program = parts.next().unwrap_or("vi").to_string();
    (program, parts.map(str::to_string).collect())
}

pub(crate) fn edit_markdown(initial: &str) -> Option<String> {
    let path = private_temp_file(initial)?;
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string());
    let (program, arguments) = editor_parts(&editor);
    let status = std::process::Command::new(&program)
        .args(&arguments)
        .arg(&path)
        .status();
    let text = fs::read_to_string(&path).ok();
    let _ = fs::remove_file(&path);
    status.ok()?.success().then_some(())?;
    text.map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

/// One-line variant used by candidate and memory editing.
pub(crate) fn edit_text(initial: &str) -> Option<String> {
    edit_markdown(initial)
        .map(one_line)
        .filter(|text| !text.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_strings_split_into_program_and_arguments() {
        assert_eq!(
            editor_parts("code --wait"),
            ("code".to_string(), vec!["--wait".to_string()])
        );
        assert_eq!(editor_parts(""), ("vi".to_string(), Vec::<String>::new()));
        assert_eq!(
            editor_parts("vim"),
            ("vim".to_string(), Vec::<String>::new())
        );
    }

    #[test]
    fn editor_temp_files_are_private_and_unique() {
        let first = private_temp_file("secret").unwrap();
        let second = private_temp_file("secret").unwrap();
        assert_ne!(first, second);
        assert_eq!(fs::read_to_string(&first).unwrap(), "secret");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&first).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
        let _ = fs::remove_file(&first);
        let _ = fs::remove_file(&second);
    }
}
