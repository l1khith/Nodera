use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use tempfile::NamedTempFile;
use tracing::{debug, trace};

use crate::error::{FileError, Result, ValidationError};

/// Characters that are illegal in file names across Windows, macOS, and Linux.
const ILLEGAL_FILENAME_CHARS: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

/// Windows reserved file names that should not be used.
const WINDOWS_RESERVED_NAMES: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Validates that a note or file name is safe and contains no illegal characters or reserved names.
pub fn validate_filename(filename: &str) -> Result<()> {
    let trimmed = filename.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::EmptyInput {
            field: "filename".to_string(),
        }
        .into());
    }

    if trimmed.starts_with('.') {
        return Err(FileError::InvalidFilename {
            filename: filename.to_string(),
            reason: "Filename cannot start with a dot '.'".to_string(),
        }
        .into());
    }

    for ch in ILLEGAL_FILENAME_CHARS {
        if trimmed.contains(*ch) {
            return Err(FileError::InvalidFilename {
                filename: filename.to_string(),
                reason: format!("Filename contains illegal character: '{ch}'"),
            }
            .into());
        }
    }

    for ch in trimmed.chars() {
        if ch.is_control() {
            return Err(FileError::InvalidFilename {
                filename: filename.to_string(),
                reason: "Filename contains control characters".to_string(),
            }
            .into());
        }
    }

    // Check Windows reserved names (case-insensitive without extension)
    let stem = Path::new(trimmed)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(trimmed);

    for reserved in WINDOWS_RESERVED_NAMES {
        if stem.eq_ignore_ascii_case(reserved) {
            return Err(FileError::InvalidFilename {
                filename: filename.to_string(),
                reason: format!("'{stem}' is a reserved filename on Windows"),
            }
            .into());
        }
    }

    Ok(())
}

/// Sanitizes an input string to create a valid, safe note filename.
pub fn sanitize_filename(input: &str) -> String {
    let mut sanitized = String::new();
    for ch in input.chars() {
        if ILLEGAL_FILENAME_CHARS.contains(&ch) || ch.is_control() {
            sanitized.push('_');
        } else {
            sanitized.push(ch);
        }
    }

    let trimmed = sanitized.trim().trim_start_matches('.').to_string();
    if trimmed.is_empty() {
        "Untitled".to_string()
    } else {
        trimmed
    }
}

/// Safely writes content to a target file using an atomic write strategy:
/// 1. Write to a temporary file in the destination's parent directory.
/// 2. Flush and sync all bytes to disk.
/// 3. Atomically rename the temporary file over the destination.
///
/// This ensures that if the system crashes or power is lost mid-write,
/// the existing destination file is never corrupted or truncated.
pub fn atomic_write(destination: impl AsRef<Path>, data: impl AsRef<[u8]>) -> Result<()> {
    let dest = destination.as_ref();
    let data = data.as_ref();

    trace!(path = %dest.display(), bytes = data.len(), "Performing atomic write");

    let parent = dest.parent().unwrap_or_else(|| Path::new("."));
    if !parent.exists() {
        fs::create_dir_all(parent).map_err(|source| FileError::WriteFailed {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    // Create tempfile in the same directory to guarantee atomic rename on the same filesystem
    let mut temp = NamedTempFile::new_in(parent).map_err(|source| FileError::WriteFailed {
        path: dest.to_path_buf(),
        source,
    })?;

    let temp_path = temp.path().to_path_buf();

    temp.write_all(data)
        .map_err(|source| FileError::AtomicWriteFailed {
            path: dest.to_path_buf(),
            temp_path: temp_path.clone(),
            source,
        })?;

    temp.flush()
        .map_err(|source| FileError::AtomicWriteFailed {
            path: dest.to_path_buf(),
            temp_path: temp_path.clone(),
            source,
        })?;

    temp.as_file()
        .sync_all()
        .map_err(|source| FileError::AtomicWriteFailed {
            path: dest.to_path_buf(),
            temp_path: temp_path.clone(),
            source,
        })?;

    // On Windows, NamedTempFile::persist replaces the destination file if it exists.
    temp.persist(dest)
        .map_err(|persist_err| FileError::AtomicWriteFailed {
            path: dest.to_path_buf(),
            temp_path: temp_path.clone(),
            source: persist_err.error,
        })?;

    debug!(path = %dest.display(), "Atomic write completed successfully");
    Ok(())
}

/// Convenience wrapper for atomic_write with UTF-8 string content.
pub fn atomic_write_str(destination: impl AsRef<Path>, content: &str) -> Result<()> {
    atomic_write(destination, content.as_bytes())
}

/// Reads a UTF-8 text file from disk, providing detailed diagnostic errors.
pub fn read_to_string(path: impl AsRef<Path>) -> Result<String> {
    let p = path.as_ref();
    if !p.exists() {
        return Err(FileError::NotFound {
            path: p.to_path_buf(),
        }
        .into());
    }

    let mut file = File::open(p).map_err(|source| FileError::ReadFailed {
        path: p.to_path_buf(),
        source,
    })?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|source| FileError::ReadFailed {
            path: p.to_path_buf(),
            source,
        })?;

    String::from_utf8(buffer).map_err(|source| {
        FileError::InvalidUtf8 {
            path: p.to_path_buf(),
            source,
        }
        .into()
    })
}

/// Ensures the directory for the given path exists.
pub fn ensure_parent_dir(path: impl AsRef<Path>) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|source| FileError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
    }
    Ok(())
}

/// Deletes a file safely, returning FileError::NotFound if it doesn't exist.
pub fn delete_file(path: impl AsRef<Path>) -> Result<()> {
    let p = path.as_ref();
    if !p.exists() {
        return Err(FileError::NotFound {
            path: p.to_path_buf(),
        }
        .into());
    }

    fs::remove_file(p).map_err(|source| FileError::Io {
        path: p.to_path_buf(),
        source,
    })?;

    debug!(path = %p.display(), "Deleted file");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_validate_filename_valid() {
        assert!(validate_filename("My Note").is_ok());
        assert!(validate_filename("2026-03-30 Daily Log.md").is_ok());
        assert!(validate_filename("Chapter 1 - Introduction").is_ok());
    }

    #[test]
    fn test_validate_filename_illegal_chars() {
        assert!(validate_filename("Note: Subtitle").is_err());
        assert!(validate_filename("What?").is_err());
        assert!(validate_filename("Path/Separators").is_err());
        assert!(validate_filename("Windows\\Backslash").is_err());
        assert!(validate_filename("Stars*").is_err());
        assert!(validate_filename("Pipe|").is_err());
    }

    #[test]
    fn test_validate_filename_reserved_windows() {
        assert!(validate_filename("CON.md").is_err());
        assert!(validate_filename("aux.txt").is_err());
        assert!(validate_filename("nul").is_err());
        assert!(validate_filename("PRN").is_err());
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Note: Subtitle?"), "Note_ Subtitle_");
        assert_eq!(sanitize_filename("  .hidden note  "), "hidden note");
        assert_eq!(sanitize_filename(""), "Untitled");
    }

    #[test]
    fn test_atomic_write_new_and_overwrite() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("Notes").join("Atomic.md");

        // Write new file
        atomic_write_str(&file_path, "# First Write\nHello").unwrap();
        let content = read_to_string(&file_path).unwrap();
        assert_eq!(content, "# First Write\nHello");

        // Overwrite atomically
        atomic_write_str(&file_path, "# Second Write\nUpdated").unwrap();
        let content_after = read_to_string(&file_path).unwrap();
        assert_eq!(content_after, "# Second Write\nUpdated");
    }

    #[test]
    fn test_read_nonexistent_fails() {
        let tmp = tempdir().unwrap();
        let missing = tmp.path().join("Missing.md");
        let result = read_to_string(&missing);
        assert!(result.is_err());
        match result.err().unwrap() {
            crate::error::NoderaError::File(FileError::NotFound { .. }) => {}
            other => panic!("Expected NotFound, got: {other:?}"),
        }
    }

    #[test]
    fn test_read_invalid_utf8_fails() {
        let tmp = tempdir().unwrap();
        let bad_file = tmp.path().join("bad.bin");
        // Write invalid UTF-8 byte sequence
        fs::write(&bad_file, [0xFF, 0xFE, 0xFD]).unwrap();

        let result = read_to_string(&bad_file);
        assert!(result.is_err());
        match result.err().unwrap() {
            crate::error::NoderaError::File(FileError::InvalidUtf8 { .. }) => {}
            other => panic!("Expected InvalidUtf8, got: {other:?}"),
        }
    }

    #[test]
    fn test_delete_file() {
        let tmp = tempdir().unwrap();
        let target = tmp.path().join("ToDelete.md");
        atomic_write_str(&target, "content").unwrap();
        assert!(target.exists());

        delete_file(&target).unwrap();
        assert!(!target.exists());

        // Deleting again fails with NotFound
        assert!(delete_file(&target).is_err());
    }
}
