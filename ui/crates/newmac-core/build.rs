//! Collect every `flavours/*.toml` (repo root) into one embedded blob so the
//! prebuilt binary ships all built-in flavours self-contained. Each file is a
//! flat flavour table; we wrap each as one `[[flavour]]` array element and
//! concatenate. A community flavour is just a new file — no shared array to
//! merge (see CONTRIBUTING.md).

use std::{env, fs, path::PathBuf};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let flavours_dir = manifest.join("../../../flavours");
    println!("cargo:rerun-if-changed={}", flavours_dir.display());

    let mut files: Vec<PathBuf> = fs::read_dir(&flavours_dir)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("toml"))
                .collect()
        })
        .unwrap_or_default();
    files.sort();

    let mut combined = String::new();
    for f in files {
        println!("cargo:rerun-if-changed={}", f.display());
        let content = fs::read_to_string(&f).unwrap_or_default();
        combined.push_str("[[flavour]]\n");
        combined.push_str(&content);
        combined.push('\n');
    }

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("flavours.toml");
    fs::write(out, combined).expect("write flavours.toml");
}
