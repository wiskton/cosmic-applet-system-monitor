use std::collections::HashSet;
use std::ffi::CString;
use std::fs::File;
use std::io::{BufRead, BufReader};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiskInfo {
    pub device: String,
    pub mount_point: String,
    pub filesystem: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub avail_bytes: u64,
    pub usage_percent: f64,
}

impl DiskInfo {
    pub fn is_critical(&self, crit_percent: f64) -> bool {
        self.usage_percent >= crit_percent
    }

    pub fn is_warning(&self, warn_percent: f64) -> bool {
        self.usage_percent >= warn_percent
    }
}

pub fn format_bytes(bytes: u64, decimals: usize) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let b = bytes as f64;
    if b >= TB {
        if decimals == 0 {
            format!("{:.0}T", (b / TB).round())
        } else {
            format!("{:.1$}T", b / TB, decimals)
        }
    } else if b >= GB {
        if decimals == 0 {
            format!("{:.0}G", (b / GB).round())
        } else {
            format!("{:.1$}G", b / GB, decimals)
        }
    } else if b >= MB {
        if decimals == 0 {
            format!("{:.0}M", (b / MB).round())
        } else {
            format!("{:.1$}M", b / MB, decimals)
        }
    } else if b >= KB {
        if decimals == 0 {
            format!("{:.0}K", (b / KB).round())
        } else {
            format!("{:.1$}K", b / KB, decimals)
        }
    } else {
        format!("{}B", bytes)
    }
}

pub fn get_disks() -> Vec<DiskInfo> {
    let mut disks = Vec::new();
    let mut seen_mounts = HashSet::new();

    let file = match File::open("/proc/mounts") {
        Ok(f) => f,
        Err(_) => return disks,
    };

    let reader = BufReader::new(file);

    for line in reader.lines().flatten() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let device = parts[0];
        let mount_point = parts[1];
        let fstype = parts[2];

        // Filtra sistemas de arquivos reais/físicos
        let is_real_dev = (device.starts_with("/dev/")
            && !device.starts_with("/dev/loop")
            && !device.contains("/pts")
            && !mount_point.starts_with("/dev"))
            || fstype == "zfs"
            || fstype == "btrfs";

        if !is_real_dev {
            continue;
        }

        if seen_mounts.contains(mount_point) {
            continue;
        }
        seen_mounts.insert(mount_point.to_string());

        if let Ok(c_mount) = CString::new(mount_point) {
            unsafe {
                let mut stat: libc::statvfs = std::mem::zeroed();
                if libc::statvfs(c_mount.as_ptr(), &mut stat) == 0 {
                    let frsize = stat.f_frsize as u64;
                    let total = (stat.f_blocks as u64).saturating_mul(frsize);
                    if total == 0 {
                        continue;
                    }
                    let free = (stat.f_bfree as u64).saturating_mul(frsize);
                    let avail = (stat.f_bavail as u64).saturating_mul(frsize);
                    let used = total.saturating_sub(free);

                    let denom = (used + avail) as f64;
                    let usage_percent = if denom > 0.0 {
                        ((used as f64) / denom) * 100.0
                    } else {
                        ((used as f64) / (total as f64)) * 100.0
                    };

                    disks.push(DiskInfo {
                        device: device.to_string(),
                        mount_point: mount_point.to_string(),
                        filesystem: fstype.to_string(),
                        total_bytes: total,
                        used_bytes: used,
                        free_bytes: free,
                        avail_bytes: avail,
                        usage_percent,
                    });
                }
            }
        }
    }

    // Ordena colocando a raiz "/" em primeiro lugar
    disks.sort_by(|a, b| {
        if a.mount_point == "/" {
            std::cmp::Ordering::Less
        } else if b.mount_point == "/" {
            std::cmp::Ordering::Greater
        } else {
            a.mount_point.cmp(&b.mount_point)
        }
    });

    disks
}
