use sysinfo::System;

// ── Structs ───────────────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct CpuInfo {
    pub name: String,
    pub cores: usize,
    pub usage: f32,
}

#[derive(serde::Serialize)]
pub struct MemoryInfo {
    pub total_gb: f64,
    pub used_gb: f64,
    pub usage: f32,
}

#[derive(serde::Serialize)]
pub struct GpuInfo {
    pub name: String,
    pub vram_total_gb: f64,
    pub vram_used_gb: Option<f64>,
}

#[derive(serde::Serialize)]
pub struct HardwareInfo {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub gpu: Option<GpuInfo>,
}

// ── WMI + Registry GPU info (Windows only) ───────────────────────────────────

#[cfg(windows)]
fn query_gpu_info() -> Option<GpuInfo> {
    use serde::Deserialize;
    use winreg::{enums::*, RegKey};
    use wmi::WMIConnection;

    // GPU name via WMI — AdapterRAM intentionally ignored (uint32 caps at ~4 GB)
    #[derive(Deserialize)]
    #[serde(rename = "Win32_VideoController")]
    struct VideoController {
        #[serde(rename = "Name")]
        name: String,
    }

    // Live VRAM usage
    #[derive(Deserialize)]
    #[serde(rename = "Win32_PerfFormattedData_GPUPerformanceCounters_GPULocalAdapterMemory")]
    struct GpuMemPerf {
        #[serde(rename = "LocalAdapterMemoryUsage")]
        local_adapter_memory_usage: u64, // KB
    }

    let wmi = WMIConnection::new().ok()?;
    let gb = 1_073_741_824.0_f64;

    let name = wmi
        .query::<VideoController>()
        .ok()?
        .into_iter()
        .next()?
        .name;

    // Read total VRAM from the registry — HardwareInformation.qwMemorySize is a
    // 64-bit QWORD so it correctly reports 8 GB, 16 GB, 24 GB, etc.
    let vram_total_gb = {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let class = hklm
            .open_subkey(
                r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}",
            )
            .ok()?;

        class
            .enum_keys()
            .filter_map(|k| k.ok())
            .filter_map(|subkey_name| class.open_subkey(&subkey_name).ok())
            .find_map(|subkey| {
                // Stored as REG_BINARY — 8 bytes, little-endian u64
                let raw = subkey
                    .get_raw_value("HardwareInformation.qwMemorySize")
                    .ok()?;
                let bytes: [u8; 8] = raw.bytes.try_into().ok()?;
                let vram_bytes = u64::from_le_bytes(bytes);
                (vram_bytes > 0).then(|| vram_bytes as f64 / gb)
            })?
    };

    let used_kb: u64 = wmi
        .query::<GpuMemPerf>()
        .ok()
        .map(|rows| rows.iter().map(|r| r.local_adapter_memory_usage).sum())
        .unwrap_or(0);

    Some(GpuInfo {
        name,
        vram_total_gb,
        vram_used_gb: (used_kb > 0).then(|| used_kb as f64 / (1024.0 * 1024.0)),
    })
}

#[cfg(not(windows))]
fn query_gpu_info() -> Option<GpuInfo> {
    None
}

// ── Tauri command ─────────────────────────────────────────────────────────────

#[tauri::command]
async fn get_hardware_info() -> HardwareInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    // CPU
    let cpus = sys.cpus();
    let cpu_name = cpus
        .first()
        .map(|c| c.brand().trim().to_string())
        .unwrap_or_else(|| "Unknown CPU".to_string());

    // Memory
    let total_mem = sys.total_memory() as f64;
    let used_mem = sys.used_memory() as f64;
    let gb = 1_073_741_824.0_f64;
    let mem_usage = if total_mem > 0.0 {
        (used_mem / total_mem * 100.0) as f32
    } else {
        0.0
    };

    HardwareInfo {
        cpu: CpuInfo {
            name: cpu_name,
            cores: cpus.len(),
            usage: sys.global_cpu_usage(),
        },
        memory: MemoryInfo {
            total_gb: total_mem / gb,
            used_gb: used_mem / gb,
            usage: mem_usage,
        },
        gpu: query_gpu_info(),
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_hardware_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}