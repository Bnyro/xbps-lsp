use std::{io, path::PathBuf, process::Command};

pub struct XlintDiagnosticsEngine {
    path: PathBuf,
}
impl XlintDiagnosticsEngine {
    pub fn new(path: PathBuf) -> Option<Self> {
        Some(XlintDiagnosticsEngine { path })
    }

    pub fn diagnostics(&self) -> io::Result<Vec<(u32, String)>> {
        let output = Command::new("xlint").arg(&self.path).output()?.stdout;
        let output = String::from_utf8(output).unwrap();

        Ok(output
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| {
                let [_path, lineno, message] = l.splitn(3, ':').collect::<Vec<_>>()[..] else {
                    unreachable!();
                };

                let lineno = lineno.parse::<u32>().unwrap();
                // xlint line numbers are 1-indexed
                (lineno - 1, message.trim().to_string())
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::xlint::XlintDiagnosticsEngine;

    #[test]
    fn simple_diagnostics() {
        let diagnostics_engine = XlintDiagnosticsEngine::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("vendored")
                .join("foo.template"),
        )
        .unwrap();
        let diagnostics = diagnostics_engine.diagnostics().unwrap();
        assert_eq!(diagnostics.len(), 8);

        // make sure the linenos are properly 0-based
        assert_eq!(
            diagnostics[0],
            (2, "variable set to empty string: version=".to_string())
        );
    }
}
