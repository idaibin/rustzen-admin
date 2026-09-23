use crate::{
    install::Manifest,
    install_crypto::hash,
    install_fs::{canonical_ustar_header, mkdir_private, write_relative},
    install_manifest::validate_payload_contracts,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

const MAX_MEMBER: u64 = 128 * 1024 * 1024;

pub(super) fn extract(bytes: &[u8], parsed: &Manifest, payload: &Path) -> Result<(), String> {
    mkdir_private(payload, 0o750)?;
    let root = format!("rz-{}-{}", parsed.artifact_class, parsed.build_id);
    let mut seen = BTreeSet::new();
    let mut offset = 0usize;
    while offset + 512 <= bytes.len() && bytes[offset..offset + 512].iter().any(|b| *b != 0) {
        let h = &bytes[offset..offset + 512];
        let (path, size, mode) = canonical_ustar_header(h)?;
        if size > MAX_MEMBER {
            return Err("archive member exceeds limit".into());
        }
        let end = offset
            .checked_add(512)
            .and_then(|x| x.checked_add(size as usize))
            .ok_or("archive size overflow")?;
        if end > bytes.len() {
            return Err("archive payload truncated".into());
        }
        let padded = end.div_ceil(512) * 512;
        if padded > bytes.len() || bytes[end..padded].iter().any(|byte| *byte != 0) {
            return Err("archive padding is not canonical".into());
        }
        if path == format!("{root}/release-manifest.json") {
            offset = padded;
            continue;
        }
        let expected =
            { parsed.files.iter().find(|x| format!("{root}/payload/{}", x.path) == path) };
        let Some(entry) = expected else {
            return Err("archive member is not selected payload".into());
        };
        if entry.kind != "file"
            || entry.mode != mode
            || entry.size != size
            || entry.sha256 != hash(&bytes[offset + 512..end])
            || !seen.insert(entry.path.clone())
        {
            return Err("archive member differs from manifest".into());
        }
        write_relative(
            payload,
            &entry.path,
            &bytes[offset + 512..end],
            if mode == "0755" { 0o755 } else { 0o644 },
        )?;
        offset = padded;
    }
    if seen.len() != parsed.files.len()
        || offset + 1024 != bytes.len()
        || bytes[offset..].iter().any(|b| *b != 0)
    {
        return Err("archive omits manifest payload member".into());
    }
    Ok(())
}

pub(super) fn scan_archive(
    bytes: &[u8],
    manifest: &Manifest,
    manifest_bytes: &[u8],
) -> Result<usize, String> {
    let root = format!("rz-{}-{}", manifest.artifact_class, manifest.build_id);
    let mut expected = manifest
        .files
        .iter()
        .map(|entry| format!("{root}/payload/{}", entry.path))
        .collect::<Vec<_>>();
    expected.push(format!("{root}/release-manifest.json"));
    expected.sort();
    let mut count = 0;
    let mut seen = BTreeSet::new();
    let mut payload = BTreeMap::new();
    let mut offset = 0;
    while offset + 512 <= bytes.len() && bytes[offset..offset + 512].iter().any(|b| *b != 0) {
        let (path, size, mode) = canonical_ustar_header(&bytes[offset..offset + 512])?;
        if expected.get(count).is_none_or(|item| item != &path) {
            return Err("archive member inventory or order differs from manifest".into());
        }
        if size > MAX_MEMBER {
            return Err("archive member exceeds limit".into());
        }
        let end = offset + 512 + size as usize;
        if end > bytes.len() {
            return Err("archive payload truncated".into());
        };
        let padded = end.div_ceil(512) * 512;
        if padded > bytes.len() || bytes[end..padded].iter().any(|byte| *byte != 0) {
            return Err("archive padding is not canonical".into());
        }
        if path == format!("{root}/release-manifest.json") {
            if mode != "0644" || &bytes[offset + 512..end] != manifest_bytes {
                return Err("embedded manifest differs".into());
            }
        } else {
            let e = manifest
                .files
                .iter()
                .find(|e| format!("{root}/payload/{}", e.path) == path)
                .ok_or("unexpected archive member")?;
            if e.mode != mode
                || e.size != size
                || e.sha256 != hash(&bytes[offset + 512..end])
                || !seen.insert(e.path.clone())
            {
                return Err("archive member differs from manifest".into());
            }
            payload.insert(e.path.clone(), &bytes[offset + 512..end]);
        };
        count += 1;
        offset = padded;
    }
    if seen.len() != manifest.files.len()
        || offset + 1024 != bytes.len()
        || bytes[offset..].iter().any(|b| *b != 0)
    {
        return Err("archive ordering, padding or trailing bytes are invalid".into());
    }
    validate_payload_contracts(manifest, &payload)?;
    Ok(count)
}
