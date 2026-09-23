use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

fn main() {
    println!("cargo:rerun-if-changed=migrations");
    let web_root = Path::new("../web/dist");
    println!("cargo:rerun-if-changed={}", web_root.display());
    let digest = full_web_digest(web_root);
    println!("cargo:rustc-env=RUSTZEN_RELEASE_FRONTEND_DIGEST={digest}");
    println!("cargo:rustc-env=RUSTZEN_PACKAGED_WEB_DIGEST={digest}");
    println!(
        "cargo:rustc-env=RUSTZEN_PACKAGED_WEB_COMPOSITION_ID={:x}",
        Sha256::digest(b"rustzen-full-composition-v1")
    );
}

fn full_web_digest(root: &Path) -> String {
    let mut files = Vec::new();
    collect_files(root, root, &mut files);
    files.sort();
    assert!(!files.is_empty(), "full Admin requires a built Web distribution");

    let inventory = files
        .into_iter()
        .map(|relative| {
            let bytes = std::fs::read(root.join(&relative)).unwrap_or_else(|error| {
                panic!("cannot read full Web asset {}: {error}", relative.display())
            });
            serde_json::json!({
                "path": relative.to_string_lossy().replace('\\', "/"),
                "sha256": format!("{:x}", Sha256::digest(bytes)),
            })
        })
        .collect::<Vec<_>>();
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&inventory).expect("serialize full Web inventory"))
    )
}

fn collect_files(root: &Path, directory: &Path, files: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(directory).unwrap_or_else(|error| {
        panic!("cannot read Web directory {}: {error}", directory.display())
    });
    for entry in entries {
        let entry = entry.expect("read Web directory entry");
        let path = entry.path();
        let metadata = entry.metadata().expect("read Web asset metadata");
        if metadata.is_dir() {
            collect_files(root, &path, files);
        } else if metadata.is_file() {
            files.push(path.strip_prefix(root).expect("Web asset under root").to_path_buf());
        } else {
            panic!("full Web distribution contains a non-regular entry: {}", path.display());
        }
    }
}
