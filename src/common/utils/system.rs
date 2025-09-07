use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;
use sysinfo::{Disk, Disks, System};

#[derive(Debug, Serialize)]
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

/// 系统工具函数集合
pub struct SystemUtils;

impl SystemUtils {
    /// 获取系统信息
    pub fn get_system_info() -> SystemInfo {
        let mut sys = System::new_all();
        sys.refresh_all();

        let (disk_total, disk_used, disk_free) = Self::get_disk_info();

        SystemInfo {
            memory_total: sys.total_memory(),
            memory_used: sys.used_memory(),
            memory_free: sys.free_memory(),
            cpu_total: sys.cpus().len(),
            cpu_used: sys.global_cpu_usage(),
            disk_total,
            disk_used,
            disk_free,
        }
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
