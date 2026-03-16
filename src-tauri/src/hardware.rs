use sysinfo::System;
use wgpu::Instance;

// ── Structs ───────────────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct CpuInfo {
    pub name: String,
    pub cores: usize,
}

#[derive(serde::Serialize)]
pub struct MemoryInfo {
    pub total_gb: f64,
}

#[derive(serde::Serialize)]
pub struct GpuInfo {
    pub name: String,
    pub vram_gb: Option<f64>,
}

#[derive(serde::Serialize)]
pub struct HardwareInfo {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub gpu: Option<GpuInfo>,
}

// ── Name helpers ──────────────────────────────────────────────────────────────

/// "Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz" → "Intel i9-10900K"
/// "AMD Ryzen 9 5900X 12-Core Processor"         → "AMD Ryzen 9 5900X"
fn shorten_cpu_name(raw: &str) -> String {
    let s = raw
        .replace("(R)", "")
        .replace("(TM)", "");

    // Strip ordinal generation prefix: "12th Gen", "13th Gen", "3rd Gen", etc.
    let words: Vec<&str> = s.split_whitespace().collect();
    let s = match words.as_slice() {
        [gen, tag, rest @ ..] if
            tag.eq_ignore_ascii_case("gen") &&
            (gen.ends_with("th") || gen.ends_with("st") || gen.ends_with("nd") || gen.ends_with("rd")) =>
            rest.join(" "),
        _ => words.join(" "),
    };

    let s = s
        .find(" CPU @")
        .or_else(|| s.find(" @"))
        .map(|i| s[..i].to_string())
        .unwrap_or(s);

    let noise = ["Processor", "CPU"];
    let words: Vec<&str> = s
        .split_whitespace()
        .filter(|w| !noise.contains(w) && !w.ends_with("-Core") && !w.ends_with("-core"))
        .collect();

    let words = if words.first().copied() == Some("Intel") {
        words.into_iter().filter(|w| *w != "Core").collect::<Vec<_>>()
    } else {
        words
    };

    let result = words.join(" ");
    if result.is_empty() { raw.trim().to_string() } else { result }
}

/// "NVIDIA GeForce RTX 4090" → "NVIDIA RTX 4090"
/// "AMD Radeon RX 7900 XTX"  → "AMD RX 7900 XTX"
fn shorten_gpu_name(raw: &str) -> String {
    let s = raw
        .replace("(R)", "")
        .replace("(TM)", "");
    // Strip trailing laptop qualifiers first, before any early returns
    let mut s = s.trim().to_string();
    for phrase in &["Laptop GPU", "Laptop"] {
        if let Some(stripped) = s.trim_end().strip_suffix(phrase) {
            s = stripped.trim_end().to_string();
        }
    }
    let s = s.as_str();

    let sub_brands = [
        ("NVIDIA GeForce ", "NVIDIA "),
        ("AMD Radeon ",     "AMD "),
    ];
    for (full, short) in &sub_brands {
        if let Some(rest) = s.strip_prefix(full) {
            return format!("{}{}", short, rest.trim());
        }
    }

    s.to_string()
}

// ── VRAM total (Windows only) ─────────────────────────────────────────────────

#[cfg(windows)]
fn query_vram_total_gb() -> Option<f64> {
    use winreg::{enums::*, RegKey};
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let class = hklm
        .open_subkey(
            r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}",
        )
        .ok()?;
    class
        .enum_keys()
        .filter_map(|k| k.ok())
        .filter_map(|name| class.open_subkey(&name).ok())
        .find_map(|subkey| {
            let raw = subkey
                .get_raw_value("HardwareInformation.qwMemorySize")
                .ok()?;
            let bytes: [u8; 8] = raw.bytes.as_ref().try_into().ok()?;
            let vram_bytes = u64::from_le_bytes(bytes);
            (vram_bytes > 0).then(|| vram_bytes as f64 / 1_073_741_824.0)
        })
}

#[cfg(not(windows))]
fn query_vram_total_gb() -> Option<f64> {
    None
}

// ── Command ───────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_hardware_info() -> HardwareInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpus = sys.cpus();
    let raw_cpu_name = cpus
        .first()
        .map(|c| c.brand().trim().to_string())
        .unwrap_or_else(|| "Unknown CPU".to_string());

    let total_mem = sys.total_memory() as f64 / 1_073_741_824.0;

    let gpu = Instance::default()
        .enumerate_adapters(wgpu::Backends::all())
        .await
        .into_iter()
        .next()
        .map(|adapter| {
            let info = adapter.get_info();
            GpuInfo {
                name: shorten_gpu_name(&info.name),
                vram_gb: query_vram_total_gb(),
            }
        });

    HardwareInfo {
        cpu: CpuInfo {
            name: shorten_cpu_name(&raw_cpu_name),
            cores: cpus.len(),
        },
        memory: MemoryInfo {
            total_gb: total_mem,
        },
        gpu,
    }
}