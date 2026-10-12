mod install;
mod validation;

pub use install::installed_release_arch;
use install::{
    collect_installed_files, extract_archive, installed_mode_matches,
    normalize_release_config_ownership, normalize_release_directory_modes, sync_directory,
    sync_release_tree,
};

use validation::inspect_archive;

use std::{
    collections::BTreeSet,
    fs,
    io::{Cursor, Read},
    path::{Path, PathBuf},
};

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::common::error::ServiceError;

pub const SIGNED_MARKER_BEGIN: &[u8] = b"\nRUSTZEN_BUNDLE_SIGNED_MARKER_BEGIN\n";
pub const SIGNED_MARKER_END: &[u8] = b"\nRUSTZEN_BUNDLE_SIGNED_MARKER_END\n";
const SIGNATURE_PAYLOAD_VERSION: &str = "rustzen-release-v2";
/// The composition this binary was built as; a bundle of any other
/// composition is rejected before any write (anti-misplacement).
#[cfg(feature = "modules")]
pub const COMPILED_COMPOSITION: &str = "full";
#[cfg(not(feature = "modules"))]
pub const COMPILED_COMPOSITION: &str = "core";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BundleInfo {
    pub arch: String,
    pub composition: &'static str,
    pub content_len: usize,
    pub frontend_sha256: String,
    pub backend_sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BundleSignatureMarker {
    schema_version: u8,
    component: String,
    version: String,
    arch: String,
    content_sha256: String,
    frontend_sha256: String,
    backend_sha256: String,
    /// Composition carried as signed-marker metadata since schema v2.1:
    /// absent means the legacy full inventory. The signature payload itself
    /// is unchanged so legacy workers keep verifying full bundles.
    composition: Option<String>,
    signature: String,
}

pub fn validate_bundle(
    data: &[u8],
    version: &str,
    signature_required: bool,
    verify_key: Option<&str>,
) -> Result<BundleInfo, ServiceError> {
    let (content, marker) = split_signed_content(data)?;
    let archive = inspect_archive(content, version)?;
    if archive.composition != COMPILED_COMPOSITION {
        return Err(invalid("Release bundle composition does not match this build"));
    }
    if let Some(marker) = &marker {
        // A marker without the composition field is the legacy full shape;
        // any explicit value must match the archive's derived composition.
        let marker_composition = marker.composition.as_deref().unwrap_or("full");
        if marker_composition != archive.composition {
            return Err(invalid("Signed release bundle composition does not match the archive"));
        }
    }
    if signature_required {
        let marker = marker.ok_or_else(|| invalid("Signed release bundle is required"))?;
        verify_signature(content, &marker, version, &archive, verify_key)?;
    }
    Ok(BundleInfo {
        arch: archive.arch,
        composition: archive.composition,
        content_len: content.len(),
        frontend_sha256: archive.frontend_sha256,
        backend_sha256: archive.backend_sha256,
    })
}

pub fn install_bundle(
    data: &[u8],
    info: &BundleInfo,
    version: &str,
    release_id: i64,
    runtime_root: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let content = data
        .get(..info.content_len)
        .ok_or_else(|| std::io::Error::other("invalid verified bundle length"))?;
    let releases = runtime_root.join("releases");
    let destination = releases.join(version);
    if destination.exists() {
        return Err(std::io::Error::other(format!(
            "release directory already exists: {}",
            destination.display()
        ))
        .into());
    }
    fs::create_dir_all(&releases)?;
    let staging = releases.join(format!(".{version}.{release_id}.installing"));
    if staging.exists() {
        fs::remove_dir_all(&staging)?;
    }
    fs::create_dir(&staging)?;

    let result = extract_archive(content, version, &info.arch, &staging).and_then(|()| {
        normalize_release_directory_modes(&staging)?;
        normalize_release_config_ownership(&staging, runtime_root)?;
        sync_release_tree(&staging)?;
        fs::rename(&staging, &destination)?;
        sync_directory(&releases)?;
        Ok(())
    });
    if result.is_err()
        && staging.exists()
        && let Err(error) = fs::remove_dir_all(&staging)
    {
        tracing::warn!(%error, path = %staging.display(), "Failed to remove release staging directory");
    }
    result.map(|()| destination)
}

pub fn verify_installed_bundle(
    data: &[u8],
    info: &BundleInfo,
    version: &str,
    release_dir: &Path,
) -> Result<(), ServiceError> {
    if !fs::symlink_metadata(release_dir).is_ok_and(|metadata| metadata.is_dir()) {
        return Err(invalid("Installed release directory is unavailable"));
    }
    let content = data
        .get(..info.content_len)
        .ok_or_else(|| invalid("Verified release bundle length is invalid"))?;
    let root = format!("rz-{version}-{}", info.arch);
    let mut expected = BTreeSet::new();
    let mut archive = tar::Archive::new(Cursor::new(content));
    for entry in archive.entries().map_err(|_| invalid("Release bundle is not a valid tar"))? {
        let mut entry = entry.map_err(|_| invalid("Release bundle contains an invalid entry"))?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = entry
            .path()
            .map_err(|_| invalid("Release bundle contains an invalid path"))?
            .into_owned();
        let relative = path
            .strip_prefix(&root)
            .map_err(|_| invalid("Release bundle root is invalid"))?
            .to_path_buf();
        let installed = release_dir.join(&relative);
        let metadata = fs::symlink_metadata(&installed)
            .map_err(|_| invalid("Installed release is incomplete"))?;
        if !metadata.file_type().is_file() {
            return Err(invalid("Installed release contains a non-regular member"));
        }
        let expected_mode = entry
            .header()
            .mode()
            .map_err(|_| invalid("Release bundle contains an invalid file mode"))?;
        if !installed_mode_matches(&metadata, expected_mode) {
            return Err(invalid("Installed release member mode does not match the signed bundle"));
        }
        let mut expected_data = Vec::new();
        entry
            .read_to_end(&mut expected_data)
            .map_err(|_| invalid("Release bundle contains an unreadable file"))?;
        if fs::read(&installed).map_err(|_| invalid("Installed release member is unreadable"))?
            != expected_data
        {
            return Err(invalid("Installed release does not match the signed bundle"));
        }
        expected.insert(relative);
    }

    let mut actual = BTreeSet::new();
    collect_installed_files(release_dir, release_dir, &mut actual)?;
    if actual != expected {
        return Err(invalid("Installed release contains an unexpected file set"));
    }
    Ok(())
}

fn split_signed_content(
    data: &[u8],
) -> Result<(&[u8], Option<BundleSignatureMarker>), ServiceError> {
    let Some(begin) = find_last(data, SIGNED_MARKER_BEGIN) else {
        return Ok((data, None));
    };
    let marker_start = begin + SIGNED_MARKER_BEGIN.len();
    let marker_end = data[marker_start..]
        .windows(SIGNED_MARKER_END.len())
        .position(|window| window == SIGNED_MARKER_END)
        .map(|offset| marker_start + offset)
        .ok_or_else(|| invalid("Signed release bundle marker is invalid"))?;
    if marker_end + SIGNED_MARKER_END.len() != data.len() {
        return Err(invalid("Signed release bundle marker must terminate the artifact"));
    }
    let marker = serde_json::from_slice::<BundleSignatureMarker>(&data[marker_start..marker_end])
        .map_err(|_| invalid("Signed release bundle marker is invalid"))?;
    verify_marker_metadata_without_signature(&marker, &data[..begin])?;
    Ok((&data[..begin], Some(marker)))
}

fn verify_marker_metadata_without_signature(
    marker: &BundleSignatureMarker,
    content: &[u8],
) -> Result<(), ServiceError> {
    if marker.schema_version != 2
        || marker.component != "release"
        || marker.content_sha256 != sha256_hex(content)
    {
        return Err(invalid("Signed release bundle metadata does not match the artifact"));
    }
    Ok(())
}

fn verify_signature(
    content: &[u8],
    marker: &BundleSignatureMarker,
    version: &str,
    archive: &validation::InspectedArchive,
    verify_key: Option<&str>,
) -> Result<(), ServiceError> {
    let content_hash = sha256_hex(content);
    if marker.schema_version != 2
        || marker.component != "release"
        || marker.version != version
        || marker.arch != archive.arch
        || marker.content_sha256 != content_hash
        || marker.frontend_sha256 != archive.frontend_sha256
        || marker.backend_sha256 != archive.backend_sha256
    {
        return Err(invalid("Signed release bundle metadata does not match the upload"));
    }
    let key_hex =
        verify_key.ok_or_else(|| invalid("Release bundle verify key is not configured"))?;
    let key_bytes: [u8; 32] = hex::decode(key_hex)
        .ok()
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or_else(|| invalid("Release bundle verify key is invalid"))?;
    let signature = Signature::from_slice(
        &hex::decode(&marker.signature)
            .map_err(|_| invalid("Release bundle signature is invalid"))?,
    )
    .map_err(|_| invalid("Release bundle signature is invalid"))?;
    let payload = signature_payload(
        version,
        &archive.arch,
        &content_hash,
        &archive.frontend_sha256,
        &archive.backend_sha256,
    );
    VerifyingKey::from_bytes(&key_bytes)
        .map_err(|_| invalid("Release bundle verify key is invalid"))?
        .verify(payload.as_bytes(), &signature)
        .map_err(|_| invalid("Release bundle signature verification failed"))
}

fn signature_payload(
    version: &str,
    arch: &str,
    content_hash: &str,
    frontend_hash: &str,
    backend_hash: &str,
) -> String {
    format!(
        "{SIGNATURE_PAYLOAD_VERSION}\ncomponent=release\nversion={version}\narch={arch}\ncontent_sha256={content_hash}\nfrontend_sha256={frontend_hash}\nbackend_sha256={backend_hash}\n"
    )
}

fn find_last(data: &[u8], needle: &[u8]) -> Option<usize> {
    data.windows(needle.len()).rposition(|window| window == needle)
}

fn sha256_hex(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

fn invalid(message: &str) -> ServiceError {
    ServiceError::InvalidOperation(message.to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use std::{fs, io::Write, path::Path};

    use super::validation::ArchiveShape;
    #[cfg(feature = "modules")]
    use super::validation::FULL_SHAPE;
    use super::validation::validate_path;

    use ed25519_dalek::{Signer, SigningKey};
    use sha2::{Digest, Sha256};

    use super::validation::inspect_archive;
    use super::{
        BundleInfo, COMPILED_COMPOSITION, SIGNED_MARKER_BEGIN, SIGNED_MARKER_END, install_bundle,
        sha256_hex, signature_payload, validate_bundle, verify_installed_bundle,
    };

    fn compiled_shape() -> &'static ArchiveShape {
        super::validation::shape_for(COMPILED_COMPOSITION).expect("compiled composition shape")
    }

    fn foreign_shape() -> &'static ArchiveShape {
        super::validation::shape_for(if COMPILED_COMPOSITION == "full" { "core" } else { "full" })
            .expect("foreign composition shape")
    }

    fn fixture_for(shape: &ArchiveShape, version: &str, arch: &str) -> Vec<u8> {
        let root = format!("rz-{version}-{arch}");
        let mut output = Vec::new();
        let mut monitor_sha256 = String::new();
        {
            let mut builder = tar::Builder::new(&mut output);
            for binary in shape.binaries {
                let mut elf = vec![0_u8; 64];
                elf[..4].copy_from_slice(b"\x7fELF");
                let machine = if arch == "x86_64" { 62_u16 } else { 183_u16 };
                elf[18..20].copy_from_slice(&machine.to_le_bytes());
                write!(
                    elf,
                    "RUSTZEN_RELEASE_MARKER\nartifact=rz-bundle-member\nbinary={binary}\nversion={version}\n{}",
                    if *binary == "rz-admin" {
                        "frontend_sha256=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n"
                    } else {
                        ""
                    }
                )
                .expect("marker");
                if *binary == "rz-monitor" {
                    monitor_sha256 = format!("{:x}", Sha256::digest(&elf));
                }
                append(&mut builder, &format!("{root}/bin/{binary}"), 0o755, &elf);
            }
            let service_units = shape.service_units().join(" ");
            let deployed_units = shape.deployed_units().join(" ");
            append(
                &mut builder,
                &format!("{root}/systemd/rz-full.service"),
                0o644,
                format!("[Unit]\nWants={service_units}\nAfter=network.target rz-recovery.service\n[Service]\nType=oneshot\nExecStart=/bin/true\nRemainAfterExit=yes\n[Install]\nWantedBy=multi-user.target\n").as_bytes(),
            );
            append(
                &mut builder,
                &format!("{root}/systemd/rz-recovery.service"),
                0o644,
                format!("[Unit]\nPartOf=rz-full.service\nBefore={deployed_units}\nStartLimitIntervalSec=60\nStartLimitBurst=3\n[Service]\nType=oneshot\nExecStart=/opt/rz/current/bin/rz-admin update recover\nRestart=on-failure\n").as_bytes(),
            );
            let specs: &[(&str, &str)] = if shape.name == "full" {
                &[
                    ("rz-admin.service", "/opt/rz/current/bin/rz-admin serve"),
                    ("rz-monitor.service", "/opt/rz/current/bin/rz-monitor controller"),
                    ("rz-insights.service", "/opt/rz/current/bin/rz-insights serve"),
                    ("rz-reports.service", "/opt/rz/current/bin/rz-reports serve"),
                ]
            } else {
                &[("rz-admin.service", "/opt/rz/current/bin/rz-admin serve")]
            };
            for (unit, command) in specs {
                append(
                    &mut builder,
                    &format!("{root}/systemd/{unit}"),
                    0o644,
                    format!(
                        "[Unit]\nAfter=network.target rz-recovery.service\nPartOf=rz-full.service\nStartLimitIntervalSec=60\nStartLimitBurst=3\n[Service]\nExecCondition=/usr/bin/test ! -e /opt/rz/data/recovery-blocked\nExecStart={command}\nRestart=on-failure\n"
                    )
                    .as_bytes(),
                );
            }
            append(&mut builder, &format!("{root}/systemd/rz-update.service"), 0o644, b"[Service]\nType=oneshot\nUser=root\nExecStart=/opt/rz/current/bin/rz-admin update request-worker\n");
            append(&mut builder, &format!("{root}/systemd/rz-update.path"), 0o644, b"[Path]\nPathChanged=/opt/rz/data/update-requests/pending.json\nUnit=rz-update.service\n");
            if shape.wants_controller_identity {
                append(&mut builder, &format!("{root}/identity/controller.json"), 0o644, format!("{{\"schemaVersion\":1,\"artifactClass\":\"full-controller\",\"version\":\"{version}\",\"arch\":\"{arch}\",\"monitorBinarySha256\":\"{monitor_sha256}\",\"agentProtocolContractId\":\"{}\"}}\n", "b".repeat(64)).as_bytes());
            }
            append(
                &mut builder,
                &format!("{root}/config/rz.env"),
                0o600,
                b"RUSTZEN_ENV=production\n",
            );
            if shape.wants_reports_config {
                append(
                    &mut builder,
                    &format!("{root}/config/rz-reports.env"),
                    0o600,
                    b"RUSTZEN_ENV=production\nRUSTZEN_IPC_TOKEN=replace-me\nRUSTZEN_REPORTS_CREDENTIAL_KEY=replace-me\n",
                );
            }
            append(&mut builder, &format!("{root}/setup-layout.sh"), 0o755, b"#!/bin/sh\n");
            builder.finish().expect("finish tar");
        }
        output
    }

    pub(crate) fn fixture(version: &str, arch: &str) -> Vec<u8> {
        fixture_for(compiled_shape(), version, arch)
    }

    fn append(builder: &mut tar::Builder<&mut Vec<u8>>, path: &str, mode: u32, data: &[u8]) {
        let mut header = tar::Header::new_gnu();
        header.set_path(path).expect("path");
        header.set_size(data.len() as u64);
        header.set_mode(mode);
        header.set_cksum();
        builder.append(&header, data).expect("append");
    }

    #[test]
    fn validates_and_installs_exact_release_binary_bundle() {
        let version = "1.2.3";
        let shape = compiled_shape();
        let data = fixture(version, "x86_64");
        let info = validate_bundle(&data, version, false, None).expect("valid bundle");
        assert_eq!(info.arch, "x86_64");
        assert_eq!(info.composition, COMPILED_COMPOSITION);
        let root = std::env::temp_dir().join(format!("rz-bundle-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(root.join("data/db/admin")).expect("admin data owner source");
        let release = install_bundle(&data, &info, version, 9, &root).expect("install");
        for binary in shape.binaries {
            assert!(release.join("bin").join(binary).is_file());
        }
        verify_installed_bundle(&data, &info, version, &release).expect("verified install");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut mode_checks = vec![
                (release.clone(), 0o755),
                (release.join("bin"), 0o755),
                (release.join("systemd"), 0o755),
                (release.join("config"), 0o700),
            ];
            if shape.wants_controller_identity {
                mode_checks.push((release.join("identity"), 0o755));
            }
            for (path, expected) in mode_checks {
                assert_eq!(
                    fs::metadata(path).expect("directory").permissions().mode() & 0o777,
                    expected
                );
            }
            let tamper_target = if shape.name == "full" { "rz-reports" } else { "rz-admin" };
            let tampered = release.join(format!("bin/{tamper_target}"));
            fs::set_permissions(&tampered, fs::Permissions::from_mode(0o644)).expect("chmod");
            assert!(verify_installed_bundle(&data, &info, version, &release).is_err());
            fs::set_permissions(&tampered, fs::Permissions::from_mode(0o755))
                .expect("restore mode");
            fs::write(&tampered, b"tampered").expect("tamper install");
            assert!(verify_installed_bundle(&data, &info, version, &release).is_err());
        }
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn rejects_missing_duplicate_unsafe_and_mismatched_members() {
        let version = "1.2.3";
        let valid = fixture(version, "x86_64");
        assert!(validate_bundle(&valid, "9.9.9", false, None).is_err());

        let mut truncated = valid.clone();
        truncated.truncate(valid.len() / 2);
        assert!(validate_bundle(&truncated, version, false, None).is_err());

        assert!(validate_path(Path::new("../escape")).is_err());
        assert!(validate_path(Path::new("/absolute")).is_err());
    }

    #[test]
    fn install_rejects_an_existing_release_directory() {
        let version = "1.2.3";
        let data = fixture(version, "aarch64");
        let info = validate_bundle(&data, version, false, None).expect("valid bundle");
        let root = std::env::temp_dir().join(format!("rz-existing-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(root.join("releases").join(version)).expect("existing release");
        assert!(
            install_bundle(
                &data,
                &BundleInfo {
                    arch: info.arch,
                    composition: info.composition,
                    content_len: info.content_len,
                    frontend_sha256: info.frontend_sha256,
                    backend_sha256: info.backend_sha256,
                },
                version,
                4,
                &root
            )
            .is_err()
        );
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn verifies_one_signature_over_the_complete_bundle_and_rejects_tampering() {
        let version = "1.2.3";
        let arch = "x86_64";
        let content = fixture(version, arch);
        let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
        let content_hash = sha256_hex(&content);
        let archive = inspect_archive(&content, version).expect("archive identity");
        let signature = signing_key.sign(
            signature_payload(
                version,
                arch,
                &content_hash,
                &archive.frontend_sha256,
                &archive.backend_sha256,
            )
            .as_bytes(),
        );
        let marker = serde_json::json!({
            "schemaVersion": 2,
            "component": "release",
            "version": version,
            "arch": arch,
            "contentSha256": content_hash,
            "frontendSha256": archive.frontend_sha256,
            "backendSha256": archive.backend_sha256,
            "composition": COMPILED_COMPOSITION,
            "signature": hex::encode(signature.to_bytes()),
        });
        let mut signed = content;
        signed.extend_from_slice(SIGNED_MARKER_BEGIN);
        signed.extend_from_slice(serde_json::to_string(&marker).expect("marker").as_bytes());
        signed.extend_from_slice(SIGNED_MARKER_END);
        let verify_key = hex::encode(signing_key.verifying_key().to_bytes());
        validate_bundle(&signed, version, true, Some(&verify_key)).expect("signed bundle");

        signed[512] ^= 1;
        assert!(validate_bundle(&signed, version, true, Some(&verify_key)).is_err());
    }

    #[test]
    fn rejects_a_foreign_composition_archive_before_any_write() {
        let version = "1.2.3";
        let foreign = fixture_for(foreign_shape(), version, "x86_64");
        let error = validate_bundle(&foreign, version, false, None)
            .expect_err("a foreign-composition archive must be rejected");
        assert!(
            error.to_string().contains("composition does not match this build"),
            "unexpected rejection: {error}"
        );
    }

    #[test]
    fn rejects_a_marker_composition_that_differs_from_the_archive() {
        let version = "1.2.3";
        let arch = "x86_64";
        let content = fixture(version, arch);
        let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
        let content_hash = sha256_hex(&content);
        let archive = inspect_archive(&content, version).expect("archive identity");
        let signature = signing_key.sign(
            signature_payload(
                version,
                arch,
                &content_hash,
                &archive.frontend_sha256,
                &archive.backend_sha256,
            )
            .as_bytes(),
        );
        let foreign_composition = if COMPILED_COMPOSITION == "full" { "core" } else { "full" };
        let marker = serde_json::json!({
            "schemaVersion": 2,
            "component": "release",
            "version": version,
            "arch": arch,
            "contentSha256": content_hash,
            "frontendSha256": archive.frontend_sha256,
            "backendSha256": archive.backend_sha256,
            "composition": foreign_composition,
            "signature": hex::encode(signature.to_bytes()),
        });
        let mut signed = content;
        signed.extend_from_slice(SIGNED_MARKER_BEGIN);
        signed.extend_from_slice(serde_json::to_string(&marker).expect("marker").as_bytes());
        signed.extend_from_slice(SIGNED_MARKER_END);
        let verify_key = hex::encode(signing_key.verifying_key().to_bytes());
        let error = validate_bundle(&signed, version, true, Some(&verify_key))
            .expect_err("a marker claiming a foreign composition must be rejected");
        assert!(
            error.to_string().contains("composition does not match the archive"),
            "unexpected rejection: {error}"
        );
    }

    #[cfg(feature = "modules")]
    #[test]
    fn legacy_marker_without_composition_is_admitted_as_full() {
        let version = "1.2.3";
        let arch = "x86_64";
        let content = fixture_for(&FULL_SHAPE, version, arch);
        let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
        let content_hash = sha256_hex(&content);
        let archive = inspect_archive(&content, version).expect("archive identity");
        let signature = signing_key.sign(
            signature_payload(
                version,
                arch,
                &content_hash,
                &archive.frontend_sha256,
                &archive.backend_sha256,
            )
            .as_bytes(),
        );
        // The pre-composition marker shape: no composition field at all.
        let marker = serde_json::json!({
            "schemaVersion": 2,
            "component": "release",
            "version": version,
            "arch": arch,
            "contentSha256": content_hash,
            "frontendSha256": archive.frontend_sha256,
            "backendSha256": archive.backend_sha256,
            "signature": hex::encode(signature.to_bytes()),
        });
        let mut signed = content;
        signed.extend_from_slice(SIGNED_MARKER_BEGIN);
        signed.extend_from_slice(serde_json::to_string(&marker).expect("marker").as_bytes());
        signed.extend_from_slice(SIGNED_MARKER_END);
        let verify_key = hex::encode(signing_key.verifying_key().to_bytes());
        let info = validate_bundle(&signed, version, true, Some(&verify_key))
            .expect("legacy full marker is admitted");
        assert_eq!(info.composition, "full");
    }

    /// DOT-OE-1900-03 evidence: the real legacy signed artifact passes the
    /// new worker's legacy admission with the real deployment verify key.
    /// The artifact exists only on machines that built it; its absence
    /// fails here with an explicit NOT-RUN message — never a silent pass,
    /// never fabricated evidence. Ordinary CI never selects this test
    /// (it is `#[ignore]`d, so default `cargo test` counts it as
    /// ignored); run it only through the dedicated selection gate
    /// `just test-real-legacy`.
    #[cfg(feature = "modules")]
    #[test]
    #[ignore = "needs the real legacy artifact and deploy key that exist only on the build host; run via just test-real-legacy"]
    fn accepts_the_real_legacy_signed_artifact() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let artifact = root.join("target/rz/rz-0.5.1-x86_64.tar");
        let key_file = root.join(".rustzen-admin/config/deploy-sign-private.pem");
        if let Err(not_run) = require_real_legacy_inputs(&artifact, &key_file) {
            panic!("real legacy artifact admission {not_run}; absence is never a pass");
        }
        let data = fs::read(&artifact).expect("read the real legacy artifact");
        let key_pem = fs::read_to_string(&key_file).expect("read the deployment signing key");
        use ed25519_dalek::pkcs8::DecodePrivateKey;
        let signing =
            SigningKey::from_pkcs8_pem(&key_pem).expect("parse the deployment signing key");
        let verify_key = hex::encode(signing.verifying_key().to_bytes());
        let info = validate_bundle(&data, "0.5.1", true, Some(&verify_key))
            .expect("the real legacy signed artifact passes legacy admission");
        assert_eq!(info.arch, "x86_64");
        assert_eq!(info.composition, "full");
    }

    /// Real-input gate for the DOT-OE-1900-03 evidence test above: missing
    /// inputs produce an explicit NOT-RUN error naming them, so absence can
    /// never be counted as a pass.
    fn require_real_legacy_inputs(artifact: &Path, key_file: &Path) -> Result<(), String> {
        let mut missing = Vec::new();
        if !artifact.is_file() {
            missing.push(format!("artifact {}", artifact.display()));
        }
        if !key_file.is_file() {
            missing.push(format!("key file {}", key_file.display()));
        }
        if missing.is_empty() {
            return Ok(());
        }
        Err(format!("NOT-RUN: missing real legacy inputs: {}", missing.join("; ")))
    }

    /// The four input states of the real-input gate: both inputs missing,
    /// only the key file present, only the artifact present, and both
    /// present. Every missing state is an explicit NOT-RUN error naming
    /// exactly the missing inputs; only both-present passes.
    #[test]
    fn missing_real_legacy_inputs_are_an_explicit_not_run_never_a_pass() {
        let root = std::env::temp_dir().join(format!("rz-notrun-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).expect("temp root");
        let artifact = root.join("rz-0.5.1-x86_64.tar");
        let key_file = root.join("deploy-sign-private.pem");

        let both_missing = require_real_legacy_inputs(&artifact, &key_file)
            .expect_err("missing inputs must be an explicit NOT-RUN, never Ok");
        assert!(both_missing.starts_with("NOT-RUN:"), "{both_missing}");
        assert!(both_missing.contains("artifact"), "{both_missing}");
        assert!(both_missing.contains("key file"), "{both_missing}");

        fs::write(&artifact, b"synthetic artifact").expect("synthetic artifact");
        let missing_key = require_real_legacy_inputs(&artifact, &key_file)
            .expect_err("a missing key file alone must still be an explicit NOT-RUN");
        assert!(missing_key.starts_with("NOT-RUN:"), "{missing_key}");
        assert!(missing_key.contains("key file"), "{missing_key}");
        assert!(!missing_key.contains("artifact"), "{missing_key}");

        fs::write(&key_file, b"synthetic key placeholder").expect("synthetic key placeholder");
        require_real_legacy_inputs(&artifact, &key_file)
            .expect("present inputs pass the existence gate");

        fs::remove_file(&artifact).expect("synthetic artifact removal");
        let missing_artifact = require_real_legacy_inputs(&artifact, &key_file)
            .expect_err("a missing artifact alone must still be an explicit NOT-RUN");
        assert!(missing_artifact.starts_with("NOT-RUN:"), "{missing_artifact}");
        assert!(missing_artifact.contains("artifact"), "{missing_artifact}");
        assert!(!missing_artifact.contains("key file"), "{missing_artifact}");

        fs::remove_dir_all(&root).expect("cleanup");
    }
}
