use once_cell::sync::Lazy;
use serde::Serialize;
use std::collections::HashSet;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use sysinfo::{CpuRefreshKind, Disk, Disks, MemoryRefreshKind, RefreshKind, System};

#[cfg(target_os = "macos")]
use std::path::Path;

static SYSTEM: Lazy<RwLock<System>> = Lazy::new(|| {
    RwLock::new(System::new_with_specifics(
        RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    ))
});

const SYSTEM_INFO_CACHE_TTL: Duration = Duration::from_secs(10);

static CACHED_INFO: Lazy<RwLock<Option<CachedSystemInfo>>> = Lazy::new(|| RwLock::new(None));

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub memory_total: u64,
    pub memory_used: u64,
    pub memory_free: u64,
    pub cpu_total: usize,
    pub cpu_used: f32,
    pub disk_total: u64,
    pub disk_used: u64,
    pub disk_free: u64,
}

#[derive(Debug, Clone)]
struct CachedSystemInfo {
    info: SystemInfo,
    fetched_at: Instant,
}

/// 系统信息工具（CPU/内存/磁盘等）
pub struct SystemUtils;

impl SystemUtils {
    /// 获取系统信息
    pub fn get_system_info() -> SystemInfo {
        if let Some(info) = Self::get_cached_system_info() {
            return info;
        }

        let mut sys = SYSTEM.write().expect("system info lock poisoned");
        sys.refresh_memory();
        sys.refresh_cpu_usage();

        let (disk_total, disk_used, disk_free) = Self::get_disk_info();

        let info = SystemInfo {
            memory_total: sys.total_memory(),
            memory_used: sys.used_memory(),
            memory_free: sys.free_memory(),
            cpu_total: sys.cpus().len(),
            cpu_used: sys.global_cpu_usage(),
            disk_total,
            disk_used,
            disk_free,
        };

        Self::cache_system_info(&info);

        info
    }

    /// 获取磁盘信息
    fn get_disk_info() -> (u64, u64, u64) {
        let disks = Disks::new_with_refreshed_list();
        let mut seen_devices = HashSet::new();
        let mut total_space = 0;
        let mut used_space = 0;
        let mut free_space = 0;

        for disk in disks.list() {
            if !is_main_disk(disk) {
                continue;
            }
            // 避免 Linux 重复设备计数
            let dev = disk.name().to_string_lossy().to_string();
            if seen_devices.contains(&dev) {
                continue;
            }
            seen_devices.insert(dev);

            let total = disk.total_space();
            let free = disk.available_space();
            let used = total - free;

            total_space += total;
            used_space += used;
            free_space += free;
        }

        (total_space, used_space, free_space)
    }

    fn get_cached_system_info() -> Option<SystemInfo> {
        let cache = CACHED_INFO.read().expect("system info cache lock poisoned");
        let cached = cache.as_ref()?;

        if cached.fetched_at.elapsed() >= SYSTEM_INFO_CACHE_TTL {
            return None;
        }

        Some(cached.info.clone())
    }

    fn cache_system_info(info: &SystemInfo) {
        let mut cache = CACHED_INFO.write().expect("system info cache lock poisoned");
        *cache = Some(CachedSystemInfo { info: info.clone(), fetched_at: Instant::now() });
    }
}

#[cfg(target_os = "macos")]
fn is_main_disk(disk: &Disk) -> bool {
    // macOS APFS: 只取根挂载点
    disk.mount_point() == Path::new("/")
}

#[cfg(target_os = "linux")]
fn is_main_disk(disk: &Disk) -> bool {
    // Linux: 去重，只算一次物理设备
    // 过滤掉 tmpfs, overlay 等虚拟文件系统
    let name = disk.name().to_string_lossy();
    !(name.starts_with("tmpfs") || name.starts_with("overlay"))
}

#[cfg(target_os = "windows")]
fn is_main_disk(_disk: &Disk) -> bool {
    // Windows: 每个盘符都是真实磁盘，直接返回 true
    true
}

#[cfg(test)]
mod tests {
    use super::{CachedSystemInfo, SYSTEM_INFO_CACHE_TTL, SystemInfo};
    use std::time::Instant;

    fn sample_info() -> SystemInfo {
        SystemInfo {
            memory_total: 1,
            memory_used: 1,
            memory_free: 0,
            cpu_total: 2,
            cpu_used: 10.0,
            disk_total: 3,
            disk_used: 1,
            disk_free: 2,
        }
    }

    #[test]
    fn cached_system_info_is_fresh_within_ttl() {
        let cached = CachedSystemInfo { info: sample_info(), fetched_at: Instant::now() };

        assert!(cached.fetched_at.elapsed() < SYSTEM_INFO_CACHE_TTL);
    }
}
