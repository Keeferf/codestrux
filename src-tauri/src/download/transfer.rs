use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use futures_util::StreamExt;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncSeekExt, AsyncWriteExt, BufWriter};

use super::types::DownloadProgress;

const MAX_RETRIES: usize = 3;
const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);

pub async fn download_parallel(
    app: AppHandle,
    client: Arc<reqwest::Client>,
    url: String,
    dest: std::path::PathBuf,
    total: u64,
    downloaded: Arc<AtomicU64>,
    cancel: Arc<AtomicBool>,
    model_id: String,
    filename: String,
    chunks: u64,
) -> Result<(), String> {
    {
        let f = tokio::fs::File::create(&dest)
            .await
            .map_err(|e| e.to_string())?;
        f.set_len(total).await.map_err(|e| e.to_string())?;
    }

    let chunk_size = (total + chunks - 1) / chunks;
    let mut tasks = Vec::new();

    for i in 0..chunks {
        let start = i * chunk_size;
        let end = ((i + 1) * chunk_size - 1).min(total - 1);

        if start >= total {
            break;
        }

        let client = Arc::clone(&client);
        let url = url.clone();
        let dest = dest.clone();
        let downloaded = Arc::clone(&downloaded);
        let cancel = Arc::clone(&cancel);
        let app = app.clone();
        let model_id = model_id.clone();
        let filename = filename.clone();

        tasks.push(tokio::spawn(async move {
            let mut attempt = 0;

            loop {
                // Exponential backoff between retries
                if attempt > 0 {
                    tokio::time::sleep(Duration::from_millis(250 * (1 << attempt))).await;
                }

                let resp = client
                    .get(&url)
                    .header("Range", format!("bytes={}-{}", start, end))
                    .send()
                    .await;

                let resp = match resp {
                    Ok(r) => r,
                    Err(e) => {
                        attempt += 1;
                        if attempt >= MAX_RETRIES {
                            return Err(e.to_string());
                        }
                        continue;
                    }
                };

                if !resp.status().is_success() && resp.status().as_u16() != 206 {
                    attempt += 1;
                    if attempt >= MAX_RETRIES {
                        return Err(format!("Chunk {} failed: HTTP {}", i, resp.status()));
                    }
                    continue;
                }

                let mut file = BufWriter::new(
                    tokio::fs::OpenOptions::new()
                        .write(true)
                        .open(&dest)
                        .await
                        .map_err(|e| e.to_string())?,
                );

                file.seek(std::io::SeekFrom::Start(start))
                    .await
                    .map_err(|e| e.to_string())?;

                let mut stream = resp.bytes_stream();
                let mut last_emit = Instant::now();

                while let Some(chunk) = stream.next().await {
                    if cancel.load(Ordering::Relaxed) {
                        return Ok(());
                    }

                    let chunk = chunk.map_err(|e| e.to_string())?;
                    file.write_all(&chunk).await.map_err(|e| e.to_string())?;

                    let so_far = downloaded.fetch_add(chunk.len() as u64, Ordering::Relaxed)
                        + chunk.len() as u64;

                    if last_emit.elapsed() >= PROGRESS_INTERVAL {
                        last_emit = Instant::now();
                        let _ = app.emit(
                            "download-progress",
                            DownloadProgress {
                                model_id: model_id.clone(),
                                filename: filename.clone(),
                                downloaded: so_far,
                                total,
                                percent: (so_far as f64 / total as f64) * 100.0,
                            },
                        );
                    }
                }

                file.flush().await.map_err(|e| e.to_string())?;
                return Ok::<(), String>(());
            }
        }));
    }

    for task in tasks {
        task.await.map_err(|e| e.to_string())??;
    }

    Ok(())
}

pub async fn download_stream(
    app: AppHandle,
    client: Arc<reqwest::Client>,
    url: String,
    dest: std::path::PathBuf,
    total: u64,
    downloaded: Arc<AtomicU64>,
    cancel: Arc<AtomicBool>,
    model_id: String,
    filename: String,
) -> Result<(), String> {
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("Download failed: HTTP {}", resp.status()));
    }

    let mut file = BufWriter::new(
        tokio::fs::File::create(&dest)
            .await
            .map_err(|e| e.to_string())?,
    );

    let mut stream = resp.bytes_stream();
    let mut last_emit = Instant::now();

    while let Some(chunk) = stream.next().await {
        if cancel.load(Ordering::Relaxed) {
            return Ok(());
        }

        let chunk = chunk.map_err(|e| e.to_string())?;
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;

        let so_far = downloaded.fetch_add(chunk.len() as u64, Ordering::Relaxed)
            + chunk.len() as u64;

        if last_emit.elapsed() >= PROGRESS_INTERVAL {
            last_emit = Instant::now();
            let _ = app.emit(
                "download-progress",
                DownloadProgress {
                    model_id: model_id.clone(),
                    filename: filename.clone(),
                    downloaded: so_far,
                    total,
                    percent: if total > 0 {
                        (so_far as f64 / total as f64) * 100.0
                    } else {
                        0.0
                    },
                },
            );
        }
    }

    file.flush().await.map_err(|e| e.to_string())?;

    Ok(())
}