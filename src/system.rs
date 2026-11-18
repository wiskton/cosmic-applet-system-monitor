use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default)]
pub struct CpuMonitor {
    prev_total: u64,
    prev_idle: u64,
    pub usage_percent: f64,
}

impl CpuMonitor {
    pub fn new() -> Self {
        let mut monitor = Self::default();
        let (total, idle) = monitor.read_raw();
        monitor.prev_total = total;
        monitor.prev_idle = idle;
        monitor
    }

    fn read_raw(&self) -> (u64, u64) {
        if let Ok(file) = File::open("/proc/stat") {
            let mut reader = BufReader::new(file);
            let mut line = String::new();
            if reader.read_line(&mut line).is_ok() && line.starts_with("cpu ") {
                let parts: Vec<u64> = line
                    .split_whitespace()
                    .skip(1)
                    .filter_map(|s| s.parse::<u64>().ok())
                    .collect();

                if parts.len() >= 4 {
                    let user = parts[0];
                    let nice = parts[1];
                    let system = parts[2];
                    let idle = parts[3];
                    let iowait = parts.get(4).copied().unwrap_or(0);
                    let irq = parts.get(5).copied().unwrap_or(0);
                    let softirq = parts.get(6).copied().unwrap_or(0);
                    let steal = parts.get(7).copied().unwrap_or(0);

                    let total_idle = idle + iowait;
                    let total_time = user + nice + system + total_idle + irq + softirq + steal;
                    return (total_time, total_idle);
                }
            }
        }
        (0, 0)
    }

    pub fn update(&mut self) {
        let (total, idle) = self.read_raw();
        let diff_total = total.saturating_sub(self.prev_total);
        let diff_idle = idle.saturating_sub(self.prev_idle);

        if diff_total > 0 {
            let non_idle = diff_total.saturating_sub(diff_idle);
            self.usage_percent = ((non_idle as f64) / (diff_total as f64)) * 100.0;
        }

        self.prev_total = total;
        self.prev_idle = idle;
    }
}

#[derive(Debug, Clone, Default)]
pub struct MemInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub avail_bytes: u64,
    pub usage_percent: f64,
}

pub fn read_memory() -> MemInfo {
    let mut mem_total_kb = 0u64;
    let mut mem_avail_kb = 0u64;

    if let Ok(file) = File::open("/proc/meminfo") {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            if line.starts_with("MemTotal:") {
                mem_total_kb = parse_kb(&line);
            } else if line.starts_with("MemAvailable:") {
                mem_avail_kb = parse_kb(&line);
            }
        }
    }

    let total = mem_total_kb * 1024;
    let avail = mem_avail_kb * 1024;
    let used = total.saturating_sub(avail);
    let usage_percent = if total > 0 {
        ((used as f64) / (total as f64)) * 100.0
    } else {
        0.0
    };

    MemInfo {
        total_bytes: total,
        used_bytes: used,
        avail_bytes: avail,
        usage_percent,
    }
}

fn parse_kb(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0)
}

#[derive(Debug, Clone)]
pub struct NetMonitor {
    prev_rx: u64,
    prev_tx: u64,
    last_time: Instant,
    pub rx_speed: f64, // bytes por segundo
    pub tx_speed: f64, // bytes por segundo
    pub total_rx: u64,
    pub total_tx: u64,
}

impl Default for NetMonitor {
    fn default() -> Self {
        let (rx, tx) = read_net_totals();
        Self {
            prev_rx: rx,
            prev_tx: tx,
            last_time: Instant::now(),
            rx_speed: 0.0,
            tx_speed: 0.0,
            total_rx: rx,
            total_tx: tx,
        }
    }
}

impl NetMonitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self) {
        let (rx, tx) = read_net_totals();
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_time).as_secs_f64();

        if elapsed > 0.0 {
            let diff_rx = rx.saturating_sub(self.prev_rx);
            let diff_tx = tx.saturating_sub(self.prev_tx);

            self.rx_speed = (diff_rx as f64) / elapsed;
            self.tx_speed = (diff_tx as f64) / elapsed;
        }

        self.prev_rx = rx;
        self.prev_tx = tx;
        self.total_rx = rx;
        self.total_tx = tx;
        self.last_time = now;
    }
}

fn read_net_totals() -> (u64, u64) {
    let mut total_rx = 0u64;
    let mut total_tx = 0u64;

    if let Ok(file) = File::open("/proc/net/dev") {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            if let Some((iface, data)) = line.split_once(':') {
                let iface = iface.trim();
                if iface == "lo" {
                    continue;
                }
                let parts: Vec<&str> = data.split_whitespace().collect();
                if parts.len() >= 9 {
                    if let Ok(rx) = parts[0].parse::<u64>() {
                        total_rx += rx;
                    }
                    if let Ok(tx) = parts[8].parse::<u64>() {
                        total_tx += tx;
                    }
                }
            }
        }
    }

    (total_rx, total_tx)
}

pub fn format_net_speed(bytes_per_sec: f64) -> String {
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = MB * 1024.0;

    if bytes_per_sec >= GB {
        format!("{:.1}G/s", bytes_per_sec / GB)
    } else {
        format!("{:.1}M/s", bytes_per_sec / MB)
    }
}

// =================== GPU MONITOR ===================

#[derive(Debug, Clone, Default)]
pub struct GpuInfo {
    pub name: String,
    pub usage_percent: f64,
    pub vram_used_mb: u64,
    pub vram_total_mb: u64,
    pub temperature_c: u64,
}

#[derive(Clone)]
pub struct GpuMonitor {
    data: Arc<Mutex<Option<GpuInfo>>>,
}

impl Default for GpuMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuMonitor {
    pub fn new() -> Self {
        let initial = query_gpu_info();
        let data = Arc::new(Mutex::new(initial));
        let data_clone = Arc::clone(&data);

        // Atualiza a GPU em background para nunca travar a UI da barra
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_millis(1500));
                let info = query_gpu_info();
                if let Ok(mut lock) = data_clone.lock() {
                    *lock = info;
                }
            }
        });

        Self { data }
    }

    pub fn get(&self) -> Option<GpuInfo> {
        self.data.lock().ok().and_then(|guard| guard.clone())
    }
}

fn query_gpu_info() -> Option<GpuInfo> {
    // 1. Tenta consulta direta via nvidia-smi
    if let Ok(output) = Command::new("nvidia-smi")
        .args([
            "--query-gpu=utilization.gpu,memory.used,memory.total,temperature.gpu,name",
            "--format=csv,noheader,nounits",
        ])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = text.trim().split(',').map(|s| s.trim()).collect();
            if parts.len() >= 5 {
                let usage = parts[0].parse::<f64>().unwrap_or(0.0);
                let vram_used = parts[1].parse::<u64>().unwrap_or(0);
                let vram_total = parts[2].parse::<u64>().unwrap_or(0);
                let temp = parts[3].parse::<u64>().unwrap_or(0);
                let name = parts[4].to_string();

                return Some(GpuInfo {
                    name,
                    usage_percent: usage,
                    vram_used_mb: vram_used,
                    vram_total_mb: vram_total,
                    temperature_c: temp,
                });
            }
        }
    }

    // 2. Fallback para sysfs (AMD ou Intel DRM)
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let path = entry.path().join("device/gpu_busy_percent");
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(pct) = content.trim().parse::<f64>() {
                        return Some(GpuInfo {
                            name: "GPU Integrada".to_string(),
                            usage_percent: pct,
                            vram_used_mb: 0,
                            vram_total_mb: 0,
                            temperature_c: 0,
                        });
                    }
                }
            }
        }
    }

    None
}
