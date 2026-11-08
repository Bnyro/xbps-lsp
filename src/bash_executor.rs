use std::{
    io::{self, Write},
    process::Command,
};

use tempfile::NamedTempFile;

/** Execute (text-form) shell scripts with Bash. */
#[derive(Default)]
pub struct BashExecutor {}

impl BashExecutor {
    pub fn execute(&self, script: &str) -> io::Result<String> {
        let mut script_file = NamedTempFile::new()?;
        script_file.write_all(script.as_bytes())?;

        let result = Command::new("bash").arg(script_file.path()).output()?;

        let output = if !result.stdout.is_empty() {
            result.stdout
        } else {
            result.stderr
        };
        Ok(String::from_utf8(output).unwrap())
    }
}
