use crate::install::Payload;
use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signature, Verifier, VerifyingKey, pkcs8::DecodePublicKey};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs::OpenOptions, io::Read, os::unix::fs::OpenOptionsExt, path::Path};

pub(super) fn canonical_json(value: &Value) -> Result<Vec<u8>, String> {
    fn append(value: &Value, out: &mut String) -> Result<(), String> {
        match value {
            Value::Null | Value::Bool(_) | Value::String(_) => out.push_str(&value.to_string()),
            Value::Number(number) => {
                if number.as_i64().is_none() && number.as_u64().is_none() {
                    return Err("canonical JSON permits integer numbers only".into());
                }
                out.push_str(&number.to_string());
            }
            Value::Array(items) => {
                out.push('[');
                for (index, item) in items.iter().enumerate() {
                    if index != 0 {
                        out.push(',');
                    }
                    append(item, out)?;
                }
                out.push(']');
            }
            Value::Object(items) => {
                out.push('{');
                let mut keys = items.keys().collect::<Vec<_>>();
                keys.sort();
                for (index, key) in keys.into_iter().enumerate() {
                    if index != 0 {
                        out.push(',');
                    }
                    out.push_str(
                        &serde_json::to_string(key).map_err(|_| "canonical JSON encoding")?,
                    );
                    out.push(':');
                    append(items.get(key).ok_or("canonical JSON key changed")?, out)?;
                }
                out.push('}');
            }
        }
        Ok(())
    }
    let mut output = String::new();
    append(value, &mut output)?;
    Ok(output.into_bytes())
}

pub(super) fn verify_signature_bytes(
    pem: &[u8],
    p: &Payload,
    signature: &str,
) -> Result<(), String> {
    let pem = String::from_utf8(pem.to_vec()).map_err(|_| "trusted key is not utf8")?;
    let key = VerifyingKey::from_public_key_pem(&pem).map_err(|_| "trusted public key invalid")?;
    let bytes = STANDARD.decode(signature).map_err(|_| "signature base64 invalid")?;
    if bytes.len() != 64 || STANDARD.encode(&bytes) != signature {
        return Err("signature base64 invalid".into());
    }
    let value = serde_json::json!({"agentProtocolContractId":p.agent_protocol_contract_id,"algorithm":p.algorithm,"archiveSha256":p.archive_sha256,"artifactClass":p.artifact_class,"buildId":p.build_id,"compositionId":p.composition_id,"domain":p.domain,"envelopeVersion":p.envelope_version,"keyId":p.key_id,"manifestSha256":p.manifest_sha256,"releaseClass":p.release_class,"releaseVersion":p.release_version,"target":p.target});
    key.verify(
        serde_json::to_string(&value).map_err(|_| "payload encoding")?.as_bytes(),
        &Signature::from_slice(&bytes).map_err(|_| "signature invalid")?,
    )
    .map_err(|_| "signature invalid".into())
}
pub(super) fn read_regular(path: &Path, max: u64) -> Result<Vec<u8>, String> {
    let mut f =
        OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(path).map_err(io)?;
    let before = f.metadata().map_err(io)?;
    if !before.file_type().is_file() || before.len() > max {
        return Err("input is not an allowed regular file".into());
    }
    let mut bytes = Vec::with_capacity(before.len() as usize);
    f.read_to_end(&mut bytes).map_err(io)?;
    let after = f.metadata().map_err(io)?;
    use std::os::unix::fs::MetadataExt;
    if bytes.len() as u64 != before.len()
        || before.dev() != after.dev()
        || before.ino() != after.ino()
        || before.mode() != after.mode()
        || before.len() != after.len()
        || before.ctime() != after.ctime()
        || before.ctime_nsec() != after.ctime_nsec()
        || before.mtime() != after.mtime()
        || before.mtime_nsec() != after.mtime_nsec()
    {
        return Err("input changed while read".into());
    }
    Ok(bytes)
}
pub(super) fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(super) fn io(error: std::io::Error) -> String {
    error.to_string()
}
