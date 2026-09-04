use super::*;

pub(super) fn list_in(
    log_dir: &Path,
    module_filter: Option<&str>,
    date_filter: Option<NaiveDate>,
) -> Result<Vec<ModuleLogFileResp>, ServiceError> {
    let Some(root) = checked_log_root(log_dir)? else {
        return Ok(Vec::new());
    };
    let today = Utc::now().date_naive();
    let mut items = Vec::new();
    for entry in fs::read_dir(&root).map_err(io_error("read log directory"))? {
        let entry = entry.map_err(io_error("read log directory entry"))?;
        let Some(file_name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let Some((module, date)) = parse_file_name(&file_name) else {
            continue;
        };
        if module_filter.is_some_and(|filter| filter != module)
            || date_filter.is_some_and(|filter| filter != date)
        {
            continue;
        }
        let path = root.join(&file_name);
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        let is_symlink = metadata.file_type().is_symlink();
        let readable = !is_symlink && metadata.is_file() && File::open(&path).is_ok();
        let modified_at =
            metadata.modified().map(DateTime::<Utc>::from).unwrap_or_else(|_| Utc::now());
        items.push(ModuleLogFileResp {
            module: module.to_string(),
            file_name,
            date: date.to_string(),
            size_bytes: if readable { metadata.len() } else { 0 },
            modified_at,
            readable,
            active: date == today,
        });
    }
    items.sort_by(|left, right| {
        left.module.cmp(&right.module).then_with(|| right.date.cmp(&left.date))
    });
    Ok(items)
}

pub(super) fn tail_in(
    log_dir: &Path,
    selector: ParsedSelector,
    cursor: Option<&str>,
) -> Result<ModuleLogTailResp, ServiceError> {
    let checked = checked_candidate(log_dir, &selector)?;
    let end = match cursor {
        Some(cursor) => decode_cursor(cursor, &selector, checked.signature)?,
        None => checked.signature.size,
    };
    let start = end.saturating_sub((MAX_TAIL_BYTES + MAX_LINE_BYTES) as u64);
    let bytes = checked.file.read_range(start, end.saturating_sub(start))?;
    if !same_signature(checked.file.signature()?, checked.signature) {
        return Err(ServiceError::InvalidOperation("Module log changed while it was read".into()));
    }

    if !bytes.is_empty() && !bytes.contains(&b'\n') {
        return tail_single_line_chunk(&selector, checked.signature, start, end, &bytes);
    }

    let mut truncated = start > 0;
    let mut content_start = start;
    let mut content_bytes = bytes.as_slice();
    if start > 0 {
        if let Some(index) = bytes.iter().position(|byte| *byte == b'\n') {
            content_start = start + index as u64 + 1;
            content_bytes = &bytes[index + 1..];
        } else {
            content_start = end;
            content_bytes = &[];
        }
    }

    let mut lines = Vec::new();
    let mut line_offset = content_start;
    for chunk in content_bytes.split_inclusive(|byte| *byte == b'\n') {
        let line_bytes = chunk.strip_suffix(b"\n").unwrap_or(chunk);
        lines.push((line_offset, String::from_utf8_lossy(line_bytes).into_owned()));
        line_offset += chunk.len() as u64;
    }
    if lines.len() > MAX_TAIL_LINES {
        let drop_count = lines.len() - MAX_TAIL_LINES;
        lines.drain(0..drop_count);
        truncated = true;
    }
    let mut bounded = Vec::with_capacity(lines.len());
    for (offset, line) in lines {
        let (line, line_truncated) = truncate_utf8(&line, MAX_LINE_BYTES);
        truncated |= line_truncated;
        bounded.push((offset, line));
    }
    while bounded.iter().map(|(_, line)| line.len()).sum::<usize>()
        + bounded.len().saturating_sub(1)
        > MAX_TAIL_BYTES
    {
        if bounded.is_empty() {
            break;
        }
        bounded.remove(0);
        truncated = true;
    }
    let next_offset = bounded.first().map(|(offset, _)| *offset).filter(|offset| *offset > 0);
    let content = bounded.iter().map(|(_, line)| line.as_str()).collect::<Vec<_>>().join("\n");
    let next_cursor = next_offset
        .map(|offset| encode_cursor(&selector, checked.signature, offset))
        .transpose()?;
    Ok(ModuleLogTailResp {
        module: selector.module,
        date: selector.date.to_string(),
        line_count: bounded.len(),
        byte_count: content.len(),
        content,
        next_cursor,
        truncated,
    })
}

pub(super) fn tail_single_line_chunk(
    selector: &ParsedSelector,
    signature: FileSignature,
    start: u64,
    end: u64,
    bytes: &[u8],
) -> Result<ModuleLogTailResp, ServiceError> {
    let segment_start = start.max(end.saturating_sub(MAX_LINE_BYTES as u64));
    let relative_start = segment_start.saturating_sub(start) as usize;
    let segment = &bytes[relative_start..];
    let (content, line_truncated) =
        truncate_utf8(&String::from_utf8_lossy(segment), MAX_LINE_BYTES);
    let next_cursor = (segment_start > 0)
        .then(|| encode_cursor(selector, signature, segment_start))
        .transpose()?;
    Ok(ModuleLogTailResp {
        module: selector.module.clone(),
        date: selector.date.to_string(),
        line_count: 1,
        byte_count: content.len(),
        content,
        next_cursor,
        truncated: line_truncated
            || start > 0
            || segment.len() < bytes.len()
            || end < signature.size,
    })
}
