use std::time::Duration;

pub const ALLOWED_HOST: &str = "hf.co";
pub const MAX_PARALLEL_CHUNKS: u64 = 24;

pub fn make_client(token: Option<&str>) -> Result<reqwest::Client, String> {
    let mut headers = reqwest::header::HeaderMap::new();

    if let Some(t) = token {
        let value = format!("Bearer {}", t)
            .parse()
            .map_err(|_| "HuggingFace token contains invalid characters".to_string())?;
        headers.insert(reqwest::header::AUTHORIZATION, value);
    }

    reqwest::Client::builder()
        .default_headers(headers)
        .pool_max_idle_per_host(32)
        .tcp_keepalive(Duration::from_secs(60))
        .timeout(Duration::from_secs(300))
        .connect_timeout(Duration::from_secs(15))
        // Compression disabled — model files are already-compressed binaries
        .build()
        .map_err(|e| e.to_string())
}

/// Sends a HEAD-style range probe to resolve the final URL, file size,
/// and whether the server supports byte-range requests.
pub async fn probe(client: &reqwest::Client, url: &str) -> Result<(String, u64, bool), String> {
    let resp = client
        .get(url)
        .header("Range", "bytes=0-0")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() && resp.status().as_u16() != 206 {
        return Err(format!("Probe failed: HTTP {}", resp.status()));
    }

    // Validate redirect hasn't left the allowed host
    if let Some(host) = resp.url().host_str() {
        if !host.ends_with(ALLOWED_HOST) {
            return Err(format!("Redirected to unexpected host: {}", host));
        }
    }

    let resolved = resp.url().to_string();
    let accepts_ranges = resp.status().as_u16() == 206;

    let total = resp
        .headers()
        .get("content-range")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split('/').last())
        .and_then(|s| s.parse::<u64>().ok())
        .or_else(|| resp.content_length())
        .unwrap_or(0);

    Ok((resolved, total, accepts_ranges))
}

/// Scales parallel chunk count to file size.
pub fn choose_chunks(total: u64) -> u64 {
    let gb = total as f64 / 1_000_000_000.0;
    if gb < 1.0 {
        8
    } else if gb < 5.0 {
        16
    } else {
        MAX_PARALLEL_CHUNKS
    }
}