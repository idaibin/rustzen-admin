use super::*;

pub(super) fn parse_selector(
    selector: &ModuleLogFileSelector,
) -> Result<ParsedSelector, ServiceError> {
    let module = parse_module(&selector.module)?;
    let date = parse_date(&selector.date)?;
    Ok(ParsedSelector { module: module.to_owned(), date, file_name: format!("{module}.{date}") })
}

pub(super) fn parse_module(value: &str) -> Result<&'static str, ServiceError> {
    MODULE_IDS
        .iter()
        .copied()
        .find(|module| *module == value)
        .ok_or_else(|| ServiceError::InvalidOperation("Unknown module log selector".into()))
}

pub(super) fn parse_date(value: &str) -> Result<NaiveDate, ServiceError> {
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| ServiceError::InvalidOperation("Invalid module log date".into()))?;
    if date.format("%Y-%m-%d").to_string() != value {
        return Err(ServiceError::InvalidOperation("Invalid module log date".into()));
    }
    Ok(date)
}

pub(super) fn parse_file_name(value: &str) -> Option<(&'static str, NaiveDate)> {
    let (module, raw_date) = value.split_once('.')?;
    if value.matches('.').count() != 1 {
        return None;
    }
    let module = MODULE_IDS.iter().copied().find(|candidate| *candidate == module)?;
    let date = NaiveDate::parse_from_str(raw_date, "%Y-%m-%d").ok()?;
    (date.format("%Y-%m-%d").to_string() == raw_date).then_some((module, date))
}
pub(super) fn signature(metadata: &Metadata) -> FileSignature {
    #[cfg(unix)]
    let identity = {
        use std::os::unix::fs::MetadataExt;
        FileIdentity { device: metadata.dev(), inode: metadata.ino() }
    };
    #[cfg(not(unix))]
    let identity = FileIdentity;
    FileSignature { size: metadata.len(), modified: metadata.modified().ok(), identity }
}

pub(super) fn same_signature(left: FileSignature, right: FileSignature) -> bool {
    left == right
}

pub(super) fn digest_bytes(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    hex::encode(digest.finalize())
}

pub(super) fn tar_entry_size(size: u64) -> u64 {
    512_u64.saturating_add(size.div_ceil(512).saturating_mul(512))
}

pub(super) fn truncate_utf8(value: &str, max_bytes: usize) -> (String, bool) {
    if value.len() <= max_bytes {
        return (value.to_owned(), false);
    }
    let mut end = max_bytes;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    (value[..end].to_owned(), true)
}

pub(super) fn encode_cursor(
    selector: &ParsedSelector,
    signature: FileSignature,
    offset: u64,
) -> Result<String, ServiceError> {
    let token = Uuid::new_v4().simple().to_string();
    let mut cursors = MODULE_LOG_CURSORS.lock().map_err(|_| {
        ServiceError::InvalidOperation("Module log cursor state is unavailable".into())
    })?;
    let now = Instant::now();
    cursors.retain(|_, state| state.expires_at > now);
    if cursors.len() >= MAX_MODULE_LOG_CURSORS
        && let Some((oldest, _)) = cursors.iter().min_by_key(|(_, state)| state.expires_at)
    {
        let oldest = oldest.clone();
        cursors.remove(&oldest);
    }
    cursors.insert(
        token.clone(),
        ModuleLogCursorState {
            module: selector.module.clone(),
            date: selector.date,
            file_name: selector.file_name.clone(),
            signature,
            offset,
            expires_at: now + MODULE_LOG_CURSOR_TTL,
        },
    );
    Ok(token)
}

pub(super) fn decode_cursor(
    cursor: &str,
    selector: &ParsedSelector,
    signature: FileSignature,
) -> Result<u64, ServiceError> {
    let mut cursors = MODULE_LOG_CURSORS.lock().map_err(|_| {
        ServiceError::InvalidOperation("Module log cursor state is unavailable".into())
    })?;
    let now = Instant::now();
    cursors.retain(|_, state| state.expires_at > now);
    let state = cursors.get(cursor).ok_or_else(|| {
        ServiceError::InvalidOperation("Invalid or expired module log cursor".into())
    })?;
    if state.module != selector.module
        || state.date != selector.date
        || state.file_name != selector.file_name
        || state.signature != signature
        || state.offset > signature.size
    {
        return Err(ServiceError::InvalidOperation(
            "Module log cursor does not match the selected file".into(),
        ));
    }
    Ok(state.offset)
}
