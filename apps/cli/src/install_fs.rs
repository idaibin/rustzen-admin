use std::{
    ffi::CString,
    io,
    os::fd::{AsRawFd, FromRawFd},
    os::unix::fs::OpenOptionsExt,
    path::Path,
};

/// Creates every component below an already-private root through directory FDs.
/// No caller-controlled component is ever reopened by pathname after validation.
pub(super) fn write_relative(
    root: &Path,
    relative: &str,
    bytes: &[u8],
    mode: u32,
) -> Result<(), String> {
    if relative.is_empty()
        || relative.split('/').any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err("payload path invalid".into());
    }
    let base = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW)
        .open(root)
        .map_err(io_text)?;
    let mut fd = base.as_raw_fd();
    let parts = relative.split('/').collect::<Vec<_>>();
    let mut owned = Vec::new();
    for part in &parts[..parts.len() - 1] {
        let name = CString::new(*part).map_err(|_| "payload path invalid")?;
        if unsafe { libc::mkdirat(fd, name.as_ptr(), 0o750) } != 0
            && io::Error::last_os_error().raw_os_error() != Some(libc::EEXIST)
        {
            return Err(io::Error::last_os_error().to_string());
        }
        let next = unsafe {
            libc::openat(
                fd,
                name.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if next < 0 {
            return Err(io::Error::last_os_error().to_string());
        }
        owned.push(unsafe { std::fs::File::from_raw_fd(next) });
        set_mode(owned.last().ok_or("opened directory missing")?, 0o750)?;
        fd = owned.last().expect("opened directory").as_raw_fd();
    }
    let name = CString::new(parts.last().expect("validated path").as_bytes())
        .map_err(|_| "payload path invalid")?;
    let raw = unsafe {
        libc::openat(
            fd,
            name.as_ptr(),
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            mode,
        )
    };
    if raw < 0 {
        return Err(io::Error::last_os_error().to_string());
    }
    let mut file = unsafe { std::fs::File::from_raw_fd(raw) };
    set_mode(&file, mode)?;
    use std::io::Write;
    file.write_all(bytes).map_err(io_text)?;
    file.sync_all().map_err(io_text)
}

pub(super) fn mkdir_private(path: impl AsRef<Path>, mode: u32) -> Result<(), String> {
    let path = path.as_ref();
    std::fs::create_dir_all(path).map_err(io_text)?;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(io_text)?;
    set_mode(&file, mode)
}

pub(super) fn write_secure(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(io_text)?;
    set_mode(&file, 0o640)?;
    use std::io::Write;
    file.write_all(bytes).map_err(io_text)?;
    file.sync_all().map_err(io_text)
}

/// Synchronizes a verified private tree from child directories through its root.
pub(super) fn fsync_tree(path: &Path) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path).map_err(io_text)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("published root is not a directory".into());
    }
    for entry in std::fs::read_dir(path).map_err(io_text)? {
        let entry = entry.map_err(io_text)?;
        let metadata = std::fs::symlink_metadata(entry.path()).map_err(io_text)?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            fsync_tree(&entry.path())?;
        } else if !metadata.is_file() {
            return Err("published root contains a non-file".into());
        }
    }
    std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(io_text)?
        .sync_all()
        .map_err(io_text)
}

fn set_mode(file: &std::fs::File, mode: u32) -> Result<(), String> {
    if unsafe { libc::fchmod(file.as_raw_fd(), mode as libc::mode_t) } != 0 {
        return Err(io::Error::last_os_error().to_string());
    }
    Ok(())
}
fn io_text(error: io::Error) -> String {
    error.to_string()
}

pub(super) fn canonical_ustar_header(h: &[u8]) -> Result<(String, u64, String), String> {
    if h.len() != 512
        || &h[257..263] != b"ustar\0"
        || &h[263..265] != b"00"
        || h[156] != b'0'
        || &h[108..116] != b"0000000\0"
        || &h[116..124] != b"0000000\0"
        || &h[136..148] != b"00000000000\0"
        || h[157..257].iter().any(|byte| *byte != 0)
        || h[265..345].iter().any(|byte| *byte != 0)
        || h[500..512].iter().any(|byte| *byte != 0)
    {
        return Err("archive header is not canonical ustar".into());
    }
    let sum = h
        .iter()
        .enumerate()
        .map(|(i, b)| if (148..156).contains(&i) { b' ' as u32 } else { *b as u32 })
        .sum::<u32>();
    if !h[148..154].iter().all(|byte| matches!(byte, b'0'..=b'7'))
        || h[154] != 0
        || h[155] != b' '
        || u64::from_str_radix(
            std::str::from_utf8(&h[148..154]).map_err(|_| "archive checksum is invalid")?,
            8,
        )
        .map_err(|_| "archive checksum is invalid")?
            != sum as u64
    {
        return Err("archive checksum is invalid".into());
    }
    let mode = String::from_utf8_lossy(cstr(&h[100..108])?).into_owned();
    if mode != "0000644" && mode != "0000755" {
        return Err("archive mode invalid".into());
    }
    let name = String::from_utf8(cstr(&h[..100])?.to_vec()).map_err(|_| "archive path utf8")?;
    let prefix =
        String::from_utf8(cstr(&h[345..500])?.to_vec()).map_err(|_| "archive path utf8")?;
    let path = if prefix.is_empty() { name } else { format!("{prefix}/{name}") };
    if path.is_empty()
        || path.starts_with('/')
        || path.split('/').any(|x| x.is_empty() || x == "." || x == "..")
    {
        return Err("archive path invalid".into());
    }
    let size_text = std::str::from_utf8(&h[124..136]).map_err(|_| "archive size invalid")?;
    if size_text.len() != 12
        || !size_text.as_bytes()[..11].iter().all(|x| matches!(x, b'0'..=b'7'))
        || size_text.as_bytes()[11] != 0
    {
        return Err("archive size field is invalid".into());
    }
    let canonical = split_ustar_path(&path)?;
    if canonical.0 != cstr(&h[..100])? || canonical.1 != cstr(&h[345..500])? {
        return Err("archive path is not canonical".into());
    }
    Ok((
        path,
        u64::from_str_radix(&size_text[..11], 8).map_err(|_| "archive size invalid")?,
        mode[3..].to_string(),
    ))
}
fn split_ustar_path(path: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\0')
        || path.split('/').any(|x| x.is_empty() || x == "." || x == "..")
    {
        return Err("archive path invalid".into());
    }
    let bytes = path.as_bytes();
    if bytes.len() <= 100 {
        return Ok((bytes.to_vec(), Vec::new()));
    }
    for index in path.match_indices('/').map(|(i, _)| i).rev() {
        let prefix = &path.as_bytes()[..index];
        let name = &path.as_bytes()[index + 1..];
        if prefix.len() <= 155 && name.len() <= 100 {
            return Ok((name.to_vec(), prefix.to_vec()));
        }
    }
    Err("archive path exceeds ustar limits".into())
}
fn cstr(bytes: &[u8]) -> Result<&[u8], String> {
    let n = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
    if bytes[n..].iter().any(|b| *b != 0) {
        return Err("archive field not canonical".into());
    }
    Ok(&bytes[..n])
}
