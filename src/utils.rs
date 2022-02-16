use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

/// Automatically implement the From between error types.
/// MyErr {
///     ForeignError {
///         source: foreign::Error,
///         description: &'static str,
///     },
/// }
///
/// map_err!(ChainStoreError, ForeignError, serde_json::Error)
///
/// This then implements the From<(err, &str)> for MyErr.
macro_rules! map_err {
    ($error_enum:ty, $branch:ident, $from_error:ty) => {
        impl From<($from_error, &'static str)> for $error_enum {
            fn from(tuple: ($from_error, &'static str)) -> Self {
                Self::$branch {
                    source: tuple.0,
                    description: tuple.1,
                }
            }
        }
    };
}
pub(crate) use map_err;

/// Utility to make path joining more ergonomic.
#[inline]
pub fn path_join(mut path: PathBuf, parts: &[&str]) -> PathBuf {
    for part in parts {
        path.push(part)
    }
    path
}

/// Call out to the "tree" command.
pub fn tree<P: AsRef<Path>>(dir: P) {
    let output = Command::new("tree")
        .current_dir(dir)
        .output()
        .expect("failed to execute `echo Hello` command");

    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
}

/// Get a Vec of lines, which is useful for test assertions.
pub fn tree_lines<P: AsRef<Path>>(dir: P) -> Vec<String> {
    let output = Command::new("tree")
        .current_dir(dir)
        .output()
        .expect("failed to execute `echo Hello` command");

    let mut result = String::new();

    result.push_str(match std::str::from_utf8(&output.stdout) {
        Ok(val) => val,
        Err(_) => panic!("failed to parse `tree` output as valid UTF-8"),
    });

    let mut lines: Vec<String> =
        result.lines().map(|l| l.replace("\u{a0}", " ")).collect();

    // Pop off things that aren't useful for testing assertions
    lines.pop(); // "n directories, n files"
    lines.pop(); // ""
    lines
}
