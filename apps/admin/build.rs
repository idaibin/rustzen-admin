use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const COMPOSITION_ID: &str = "8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b";
const SELECTED_WEB_ROOT: &str =
    "selected-web/8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b";
const INVENTORY_SHA256: &str = "cc4515d174b6b06e5a8df7b11c4d33a0b2f58880ec64123708e7aaeedf9e8e13";
const GENERATED_ROOT: &str =
    "apps/web/.selected-web/8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b";
const OUTPUT_DIRECTORY: &str = "target/distributions/8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b/web/dist";
const SELECTED_ROUTES: &[&str] = &[
    "403.tsx",
    "404.tsx",
    "__root.tsx",
    "index.tsx",
    "login.tsx",
    "monitoring.tsx",
    "monitoring/-global-alert-settings.tsx",
    "monitoring/-incident-drawer.tsx",
    "monitoring/-node-alert-policy.tsx",
    "monitoring/-node-details.tsx",
    "monitoring/-node-onboarding.tsx",
    "monitoring/-save-state.ts",
    "monitoring/incidents.tsx",
    "monitoring/nodes.tsx",
    "monitoring/overview.tsx",
    "monitoring/summaries.tsx",
    "profile.tsx",
    "system/-role-actions.tsx",
    "system/-role-delete-state.ts",
    "system/-role-dialog.tsx",
    "system/-role-permission-picker.tsx",
    "system/-user-actions.tsx",
    "system/-user-dialog.tsx",
    "system/role.tsx",
    "system/user.tsx",
];

fn main() {
    println!("cargo:rerun-if-changed=migrations");
    println!("cargo:rerun-if-changed={SELECTED_WEB_ROOT}");
    if std::env::var_os("CARGO_FEATURE_MONITOR_DISTRIBUTION").is_some() {
        validate_selected_web();
    }
}

fn validate_selected_web() {
    let binding_path = format!("{SELECTED_WEB_ROOT}/binding.json");
    let inventory_path = format!("{SELECTED_WEB_ROOT}/inventory.json");
    let api_path = format!("{SELECTED_WEB_ROOT}/api.ts");
    let dist_path = format!("{SELECTED_WEB_ROOT}/dist");
    let index_path = format!("{SELECTED_WEB_ROOT}/dist/index.html");
    let binding = read_json(&binding_path, "binding");
    let inventory = read_json(&inventory_path, "inventory");
    let binding_object = binding.as_object().expect("selected Web binding must be an object");
    let mut keys = binding_object.keys().map(String::as_str).collect::<Vec<_>>();
    keys.sort_unstable();
    assert_eq!(
        keys,
        ["bindingVersion", "compositionId", "selectedApiDigest", "webDigest"],
        "selected Web binding schema mismatch"
    );
    assert_eq!(binding["bindingVersion"], 1, "selected Web binding version mismatch");
    assert_eq!(
        binding["compositionId"].as_str(),
        Some(COMPOSITION_ID),
        "selected Web binding composition mismatch"
    );
    for field in ["compositionId", "selectedApiDigest", "webDigest"] {
        assert!(
            binding[field].as_str().is_some_and(is_sha256),
            "selected Web binding {field} must be a lowercase SHA-256"
        );
    }
    validate_inventory_contract(&inventory);
    assert_eq!(
        inventory["compositionId"], binding["compositionId"],
        "selected Web inventory composition differs from binding"
    );
    assert_eq!(inventory["binding"], binding, "selected Web inventory binding differs");

    let selected_api = std::fs::read(&api_path)
        .unwrap_or_else(|_| panic!("monitor distribution requires selected Web API: {api_path}"));
    assert_eq!(
        binding["selectedApiDigest"].as_str(),
        Some(sha256(&selected_api).as_str()),
        "selected Web API digest mismatch"
    );

    let index = std::fs::read_to_string(&index_path).unwrap_or_else(|_| {
        panic!("monitor distribution requires selected Web HTML: {index_path}")
    });
    let digest = binding["webDigest"].as_str().expect("validated Web digest");
    let expected = format!(r#"<meta name="rustzen-web-binding" content="{digest}" />"#);
    assert_eq!(
        index.match_indices("rustzen-web-binding").count(),
        1,
        "selected Web HTML must contain one binding marker"
    );
    assert_eq!(
        index.match_indices(&expected).count(),
        1,
        "selected Web HTML binding stamp mismatch"
    );
    assert!(
        !index.contains("__RUSTZEN_WEB_DIGEST__"),
        "selected Web HTML still contains the binding slot"
    );
    validate_dist(&inventory, Path::new(&dist_path), digest);
    let inventory_bytes = std::fs::read(&inventory_path).expect("validated selected Web inventory");
    assert_eq!(
        sha256(&inventory_bytes),
        INVENTORY_SHA256,
        "selected Web canonical inventory digest mismatch"
    );
    println!("cargo:rustc-env=RUSTZEN_PACKAGED_WEB_DIGEST={digest}");
    println!("cargo:rustc-env=RUSTZEN_PACKAGED_WEB_COMPOSITION_ID={COMPOSITION_ID}");
}

fn validate_inventory_contract(inventory: &Value) {
    let object = inventory.as_object().expect("selected Web inventory must be an object");
    let mut keys = object.keys().map(String::as_str).collect::<Vec<_>>();
    keys.sort_unstable();
    let mut expected = [
        "schemaVersion",
        "preset",
        "compositionId",
        "generatedRoot",
        "outputDirectory",
        "selectedRoutes",
        "publicAssets",
        "emittedFiles",
        "fileInventory",
        "moduleIds",
        "binding",
    ];
    expected.sort_unstable();
    assert_eq!(keys, expected, "selected Web inventory schema mismatch");
    assert_eq!(inventory["schemaVersion"], 2, "selected Web inventory version mismatch");
    assert_eq!(inventory["preset"], "monitor", "selected Web inventory preset mismatch");
    assert_eq!(
        inventory["compositionId"].as_str(),
        Some(COMPOSITION_ID),
        "selected Web inventory composition mismatch"
    );
    assert_eq!(inventory["generatedRoot"], GENERATED_ROOT, "selected Web generated root mismatch");
    assert_eq!(inventory["outputDirectory"], OUTPUT_DIRECTORY, "selected Web output mismatch");
    assert_eq!(
        string_array(&inventory["selectedRoutes"], "selected routes"),
        SELECTED_ROUTES,
        "selected Web inventory route set mismatch"
    );
    assert_eq!(
        string_array(&inventory["publicAssets"], "public assets"),
        ["rustzen.png"],
        "selected Web inventory public assets mismatch"
    );
    let modules = string_array(&inventory["moduleIds"], "module IDs");
    let prefix = format!("apps/web/.selected-web/{COMPOSITION_ID}/");
    assert!(
        modules.iter().any(|id| id.starts_with(&prefix))
            && modules.iter().all(|id| valid_module_id(id, &prefix)),
        "selected Web inventory module IDs mismatch"
    );
}

fn string_array<'a>(value: &'a Value, label: &str) -> Vec<&'a str> {
    let values =
        value.as_array().unwrap_or_else(|| panic!("selected Web {label} must be an array"));
    let result = values
        .iter()
        .map(|item| item.as_str().unwrap_or_else(|| panic!("selected Web {label} must be strings")))
        .collect::<Vec<_>>();
    assert!(
        result.windows(2).all(|pair| pair[0] < pair[1]),
        "selected Web {label} must be sorted and unique"
    );
    result
}

fn valid_module_id(id: &str, generated_prefix: &str) -> bool {
    if id == "\0vite/modulepreload-polyfill.js" || id == "\0vite/preload-helper.js" {
        return true;
    }
    !id.is_empty()
        && !id.starts_with('/')
        && !id.contains('\\')
        && id.split('/').all(|part| part != "." && part != "..")
        && (!id.starts_with("apps/web/.selected-web/") || id.starts_with(generated_prefix))
}

fn validate_dist(inventory: &Value, root: &Path, web_digest: &str) {
    let rows = inventory["fileInventory"]
        .as_array()
        .expect("selected Web inventory file table must be an array");
    let mut expected = Vec::with_capacity(rows.len());
    for row in rows {
        let object = row.as_object().expect("selected Web file entry must be an object");
        let mut keys = object.keys().map(String::as_str).collect::<Vec<_>>();
        keys.sort_unstable();
        assert_eq!(keys, ["path", "sha256", "size"], "selected Web file schema mismatch");
        let path = row["path"].as_str().expect("selected Web file path must be a string");
        assert_safe_path(path);
        let size = row["size"].as_u64().expect("selected Web file size must be unsigned");
        let digest = row["sha256"]
            .as_str()
            .filter(|value| is_sha256(value))
            .expect("selected Web file digest must be a lowercase SHA-256");
        let bytes = std::fs::read(root.join(path))
            .unwrap_or_else(|_| panic!("selected Web inventory file is missing: {path}"));
        assert_eq!(bytes.len() as u64, size, "selected Web file size mismatch: {path}");
        assert_eq!(sha256(&bytes), digest, "selected Web file digest mismatch: {path}");
        expected.push(path.to_owned());
    }
    assert!(
        expected.windows(2).all(|pair| pair[0] < pair[1]),
        "selected Web file inventory must be sorted and unique"
    );
    let mut actual = Vec::new();
    collect_files(root, root, &mut actual);
    actual.sort();
    assert_eq!(actual, expected, "selected Web inventory file set mismatch");
    assert_eq!(
        string_array(&inventory["emittedFiles"], "emitted files"),
        actual,
        "selected Web emitted files mismatch"
    );

    let mut normalized = expected
        .iter()
        .map(|path| {
            let mut bytes = std::fs::read(root.join(path)).expect("validated selected Web file");
            if path == "index.html" {
                let stamped =
                    format!(r#"<meta name="rustzen-web-binding" content="{web_digest}" />"#);
                let text = String::from_utf8(bytes).expect("selected Web HTML must be UTF-8");
                bytes = text
                    .replace(
                        &stamped,
                        r#"<meta name="rustzen-web-binding" content="__RUSTZEN_WEB_DIGEST__" />"#,
                    )
                    .into_bytes();
            }
            serde_json::json!({"path": path, "sha256": sha256(&bytes)})
        })
        .collect::<Vec<_>>();
    normalized.sort_by(|left, right| left["path"].as_str().cmp(&right["path"].as_str()));
    let canonical = serde_json::to_vec(&normalized).expect("selected Web digest table serializes");
    assert_eq!(sha256(&canonical), web_digest, "selected Web normalized digest mismatch");
}

fn collect_files(directory: &Path, root: &Path, files: &mut Vec<String>) {
    for entry in std::fs::read_dir(directory).expect("selected Web directory must be readable") {
        let entry = entry.expect("selected Web directory entry must be readable");
        let kind = entry.file_type().expect("selected Web file type must be readable");
        assert!(!kind.is_symlink(), "selected Web output must not contain symlinks");
        if kind.is_dir() {
            collect_files(&entry.path(), root, files);
        } else {
            assert!(kind.is_file(), "selected Web output contains unsupported entry");
            files.push(relative_path(entry.path(), root));
        }
    }
}

fn relative_path(path: PathBuf, root: &Path) -> String {
    path.strip_prefix(root)
        .expect("selected Web file must remain below root")
        .components()
        .map(|part| part.as_os_str().to_str().expect("selected Web path must be UTF-8"))
        .collect::<Vec<_>>()
        .join("/")
}

fn assert_safe_path(path: &str) {
    assert!(
        !path.is_empty()
            && !path.starts_with('/')
            && !path.contains('\\')
            && path.split('/').all(|part| !part.is_empty() && part != "." && part != ".."),
        "selected Web inventory path is unsafe"
    );
}

fn read_json(path: &str, label: &str) -> Value {
    let bytes = std::fs::read(path)
        .unwrap_or_else(|_| panic!("monitor distribution requires selected Web {label}: {path}"));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| panic!("selected Web {label} must be valid JSON: {path}"))
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
