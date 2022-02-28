use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

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

#[cfg(test)]
thread_local! {
    pub static BLOCK_TIMESTAMP: std::cell::RefCell<i64> = std::cell::RefCell::new(0);
}

#[cfg(test)]
pub fn get_timestamp() -> i64 {
    let mut returns = 0;
    BLOCK_TIMESTAMP.with(|f| {
        returns = *f.borrow();
        *f.borrow_mut() = returns + 1;
    });
    return returns;
}

/// Gets a test-safe timestamp.
#[cfg(not(test))]
pub fn get_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

#[cfg(test)]
pub struct TimeStampScope {}

#[cfg(test)]
impl TimeStampScope {
    pub fn new() -> Self {
        BLOCK_TIMESTAMP.with(|f| {
            *f.borrow_mut() = 0;
        });
        TimeStampScope {}
    }
}

#[cfg(test)]
impl Drop for TimeStampScope {
    fn drop(&mut self) {
        BLOCK_TIMESTAMP.with(|f| {
            *f.borrow_mut() = 0;
        });
    }
}

pub fn touch(path: &PathBuf) {
    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(path.clone())
        .expect("Failed to touch path.");
}

/// Utility to print only once for debugging.
#[macro_export]
macro_rules! print_once {
    ( $($args:tt),* ) => {
        thread_local! {
            pub static HAS_PRINTED: std::cell::RefCell<bool> = std::cell::RefCell::new(false);
        }

        HAS_PRINTED.with(|f| {
            if *f.borrow() == false {
                println!( $($args),* );
                *f.borrow_mut() =  true;
            }
        });
    }
}
