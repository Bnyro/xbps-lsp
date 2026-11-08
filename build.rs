use std::{fs::File, io::Write, path::PathBuf};

use anyhow::anyhow;

const VOID_FILE_URL_TEMPLATE: &str =
    "https://raw.githubusercontent.com/void-linux/void-packages/refs/heads/master/{path}";

fn download_file(upstream_path: &str) -> anyhow::Result<()> {
    println!("cargo::rerun-if-changed=build.rs");

    let url = VOID_FILE_URL_TEMPLATE.replace("{path}", upstream_path);

    let upstream_path = PathBuf::from(upstream_path);
    let filename = upstream_path
        .file_name()
        .ok_or(anyhow!("invalid file path"))?;
    let out_path = PathBuf::from(std::env::var("OUT_DIR")?).join(filename);

    let file_resp = reqwest::blocking::get(url)?;
    assert_eq!(file_resp.status(), 200);
    File::create(&out_path)?.write_all(&file_resp.bytes()?)?;

    Ok(())
}

fn main() {
    download_file("Manual.md").unwrap();
    download_file("common/xbps-src/shutils/update_check.sh").unwrap();
}
