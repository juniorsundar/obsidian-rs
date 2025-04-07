use std::borrow::Cow;
use std::env;
use std::path::{Path, PathBuf}; // To avoid allocation when no expansion happens

// Helper function to get the home directory path based on OS
fn get_home_dir() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        env::var("HOME").ok().map(PathBuf::from)
    }
    #[cfg(windows)]
    {
        env::var("USERPROFILE").ok().map(PathBuf::from)
    }
    #[cfg(not(any(unix, windows)))]
    {
        // Basic fallback or unsupported platform marker
        // You might choose to handle other env vars if needed
        None
    }
}

/// Expands a path starting with '\~' to the user's home directory.
///
/// Returns `None` if the path starts with '\~\' but the home directory
/// cannot be determined.
/// Returns `Some(Cow::Borrowed(input_path))` if the path doesn't start with '\~'.
/// Returns `Some(Cow::Owned(expanded_path))` if expansion occurred.
///
/// Note: This implementation relies on HOME (Unix) or USERPROFILE (Windows)
/// environment variables and does not cover all edge cases handled by
/// crates like `shellexpand` or `home`. Assumes UTF-8 path representation
/// for the '~' check.
pub fn expand_tilde(input_path: &Path) -> Option<Cow<Path>> {
    // Attempt to convert the Path to a &str to check for '~'.
    // If the path is not valid UTF-8, we cannot reliably check for '~',
    // so we return the original path without attempting expansion.
    let path_str = match input_path.to_str() {
        Some(s) => s,
        None => return Some(Cow::Borrowed(input_path)), // Not UTF-8, return original
    };

    if path_str == "~" {
        // Path is exactly "~"
        get_home_dir().map(Cow::Owned) // Map Option<PathBuf> to Option<Cow<'_, Path>>
    } else if path_str.starts_with("~/") {
        // Path starts with "~/"
        get_home_dir().map(|mut home_dir| {
            // Remove the "~/" prefix (2 bytes)
            let remainder = &path_str[2..];
            home_dir.push(remainder); // Append the rest of the path
            Cow::Owned(home_dir)
        })
    } else {
        // Path does not start with "~" or "~/", return original
        Some(Cow::Borrowed(input_path))
    }
}

// --- Unit tests (optional but recommended) ---
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Helper to run tests - set env var temporarily if needed
    // Note: Tests modifying env vars should be run serially (`cargo test -- --test-threads=1`)

    #[test]
    #[cfg(any(unix, windows))] // Only run where we expect HOME/USERPROFILE
    fn test_tilde_expansion() {
        // Set a known home dir for testing (be careful with parallel tests)
        let test_home = if cfg!(unix) {
            "/testhome"
        } else {
            "C:\\TestHome"
        };
        let key = if cfg!(unix) { "HOME" } else { "USERPROFILE" };
        unsafe {
            std::env::set_var(key, test_home);
        }

        let home_path = PathBuf::from(test_home);

        // Test "~"
        let input1 = Path::new("~");
        let expected1 = Some(Cow::Owned(home_path.clone()));
        assert_eq!(expand_tilde(input1), expected1);

        // Test "~/"
        let input2 = Path::new("~/Documents");
        let mut expected2_path = home_path.clone();
        expected2_path.push("Documents");
        let expected2 = Some(Cow::Owned(expected2_path));
        assert_eq!(expand_tilde(input2), expected2);

        // Test non-tilde path
        let input3 = Path::new("/absolute/path");
        let expected3 = Some(Cow::Borrowed(input3));
        assert_eq!(expand_tilde(input3), expected3);

        // Test path starting with different character
        let input4 = Path::new(".config");
        let expected4 = Some(Cow::Borrowed(input4));
        assert_eq!(expand_tilde(input4), expected4);

        // Clean up env var if necessary (or ignore if test isolation is handled)
        unsafe {
            std::env::remove_var(key);
        }
    }

    #[test]
    fn test_no_home_env() {
        // Temporarily remove HOME/USERPROFILE
        let key = if cfg!(unix) { "HOME" } else { "USERPROFILE" };
        let original_value = std::env::var(key).ok();
        unsafe {
            std::env::remove_var(key);
        }

        let input1 = Path::new("~");
        assert_eq!(expand_tilde(input1), None); // Should fail if HOME/USERPROFILE is not set

        let input2 = Path::new("~/Documents");
        assert_eq!(expand_tilde(input2), None); // Should fail

        // Restore original value
        if let Some(val) = original_value {
            unsafe {
                std::env::set_var(key, val);
            }
        }
    }
}
