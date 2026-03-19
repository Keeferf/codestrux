use std::time::Duration;

pub const ALLOWED_HOST: &str = "hf.co";
pub const MAX_PARALLEL_CHUNKS: u64 = 64;
pub const TARGET_CHUNK_SIZE: u64 = 50 * 1024 * 1024; // 50MB chunks

pub fn make_client(token: Option<&str>) -> Result<reqwest::Client, String> {
    let mut headers = reqwest::header::HeaderMap::new();

    // Explicitly opt out of content encoding for all requests made by this
    // client. Without gzip/brotli features reqwest won't advertise them, but
    // setting this header makes the intent explicit and guards against any
    // future middleware or feature re-enabling transparent decompression, which
    // would corrupt binary file transfers and break Content-Range accounting.
    headers.insert(
        reqwest::header::ACCEPT_ENCODING,
        "identity".parse().unwrap(),
    );

    if let Some(t) = token {
        let value = format!("Bearer {}", t)
            .parse()
            .map_err(|_| "HuggingFace token contains invalid characters".to_string())?;
        headers.insert(reqwest::header::AUTHORIZATION, value);
    }

    reqwest::Client::builder()
        .default_headers(headers)
        .pool_max_idle_per_host(MAX_PARALLEL_CHUNKS as usize)
        .tcp_keepalive(Duration::from_secs(60))
        .timeout(Duration::from_secs(300))
        .connect_timeout(Duration::from_secs(15))
        .http2_adaptive_window(true)
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

/// Scales parallel chunk count based on an optimal target size.
///
/// Minimum is 1 (not 4) so that files smaller than TARGET_CHUNK_SIZE are
/// routed through download_stream rather than paying parallel overhead for
/// a single logical chunk split artificially across 4 connections.
pub fn choose_chunks(total: u64) -> u64 {
    if total == 0 {
        return 1;
    }

    let chunks = (total + TARGET_CHUNK_SIZE - 1) / TARGET_CHUNK_SIZE;
    // Fix #5: was clamp(4, MAX_PARALLEL_CHUNKS) — the floor of 4 forced small
    // files into parallel mode unnecessarily. clamp(1, ...) lets the natural
    // math produce 1 for files under TARGET_CHUNK_SIZE, and commands.rs's
    // `if chunks > 1` guard will correctly route them to download_stream.
    chunks.clamp(1, MAX_PARALLEL_CHUNKS)
}