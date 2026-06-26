fn main() -> Result<()> {
    let manifest_dir = webview2_path::get_manifest_dir()?;
    let out_dir = webview2_path::get_out_dir()?;
    webview2_link::output_loader_dlls(manifest_dir, out_dir)?;

    println!("cargo:rustc-link-lib=advapi32");
    Ok(())
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Var(#[from] std::env::VarError),
}

pub type Result<T> = std::result::Result<T, Error>;

#[macro_use]
extern crate thiserror;

mod webview2_path {
    use std::{convert::From, env, path::PathBuf};

    pub fn get_out_dir() -> super::Result<PathBuf> {
        Ok(PathBuf::from(env::var("OUT_DIR")?))
    }

    pub fn get_manifest_dir() -> super::Result<PathBuf> {
        Ok(PathBuf::from(env::var("CARGO_MANIFEST_DIR")?))
    }
}

mod webview2_link {
    use std::path::PathBuf;

    pub fn output_loader_dlls(manifest_dir: PathBuf, out_dir: PathBuf) -> super::Result<()> {
        const WEBVIEW2_TARGETS: &[&str] = &["arm64", "x64", "x86"];

        for target in WEBVIEW2_TARGETS {
            use std::fs;

            let mut lib_src = manifest_dir.clone();
            lib_src.push(target);
            lib_src.push("WebView2Loader.dll");
            if !lib_src.is_file() {
                eprintln!("Skip missing {:?}", lib_src);
                continue;
            }

            let mut lib_dest = out_dir.clone();
            lib_dest.push(target);
            if !lib_dest.is_dir() {
                fs::create_dir(lib_dest.as_path())?;
            }

            lib_dest.push("WebView2Loader.dll");
            eprintln!("Copy from {:?} -> {:?}", lib_src, lib_dest);
            fs::copy(lib_src.as_path(), lib_dest.as_path())?;
        }

        Ok(())
    }
}
