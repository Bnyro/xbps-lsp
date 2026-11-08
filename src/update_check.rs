use std::path::PathBuf;

use crate::{bash_executor::BashExecutor, xbps_template::XBPSTemplate};

#[derive(Debug, PartialEq, Eq)]
pub enum UpdateCheckError {
    NoVersionFound,
    PackageDirNotFound,
    MissingField,
    IoError(String),
}

#[derive(Debug)]
pub struct UpdateChecker {
    template: String,
    template_path: PathBuf,
}
impl UpdateChecker {
    pub fn new(template: String, template_path: PathBuf) -> Self {
        Self {
            template,
            template_path,
        }
    }

    pub fn current_version(&self) -> Option<(usize, String)> {
        let template = XBPSTemplate::from_text(self.template.clone());
        template.variable_value("version")
    }

    pub fn new_version(&self) -> Result<Option<String>, UpdateCheckError> {
        let template = XBPSTemplate::from_text(self.template.clone());
        let package_dir = self
            .template_path
            .parent() // <pkgname> dir
            .ok_or(UpdateCheckError::PackageDirNotFound)?
            .parent() // void pkgs dir
            .ok_or(UpdateCheckError::PackageDirNotFound)?
            .to_string_lossy();

        let update_script = include_str!(concat!(env!("OUT_DIR"), "/update_check.sh"));
        let extra_script = format!(
            r#"pkgname="{pkgname}"
sourcepkg="{pkgname}"
version="{version}"
homepage="{homepage}"
distfiles="{distfiles}"

msg_verbose() {{ :; }}
XBPS_SRC_VERSION=1.0
XBPS_SRCPKGDIR={pkgdir}
XBPS_TARGET_PKG={pkgname}
update_check
"#,
            pkgname = template
                .variable_value("pkgname")
                .ok_or(UpdateCheckError::MissingField)?
                .1,
            version = template
                .variable_value("version")
                .ok_or(UpdateCheckError::MissingField)?
                .1,
            homepage = template
                .variable_value("homepage")
                .ok_or(UpdateCheckError::MissingField)?
                .1,
            distfiles = template
                .variable_value("distfiles")
                .ok_or(UpdateCheckError::MissingField)?
                .1,
            pkgdir = package_dir,
        );
        let script = update_script.to_string() + &extra_script;

        let result = BashExecutor::default()
            .execute(&script)
            .map_err(|err| UpdateCheckError::IoError(err.to_string()))?;
        dbg!(&result);
        if result.contains("NO VERSION found") {
            return Err(UpdateCheckError::NoVersionFound);
        }

        match result.rsplit_once("-") {
            Some((_, new_version)) => Ok(Some(new_version.to_string())),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::update_check::{UpdateCheckError, UpdateChecker};

    #[test]
    fn update_check_success() {
        let template_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test")
            .join("typos")
            .join("template");
        let update_checker = UpdateChecker::new(
            include_str!("../test/typos/template").to_string(),
            template_dir,
        );
        assert_eq!(
            update_checker.current_version(),
            Some((2, "1.51.1".to_string()))
        );
        assert!(update_checker.new_version().is_ok_and(|v| v.is_some()));
    }

    #[test]
    fn update_check_no_versions() {
        let template_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test")
            .join("foo")
            .join("template");
        let update_checker = UpdateChecker::new(
            include_str!("../test/foo/template").to_string(),
            template_dir,
        );
        assert_eq!(
            update_checker.new_version(),
            Err(UpdateCheckError::NoVersionFound)
        );
    }
}
