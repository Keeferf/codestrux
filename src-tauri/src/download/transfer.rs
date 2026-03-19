use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use futures_util::StreamExt;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncSeekExt, AsyncWriteExt, BufWriter};
use tokio::task::JoinSet;

use super::types::DownloadProgress;

const MAX_CONNECT_RETRIES: u32 = 3;
const MAX_STREAM_RETRIES: u32 = 3;
const PROGRESS_INTERVAL: Duration = Duration::from_millis(250);

/// Spawns a background task that emits `download-progress` events on a fixed
/// interval until the download completes, is cancelled, or the handle is aborted.
fn spawn_progress_reporter(
    app: AppHandle,
    downloaded: Arc<AtomicU64>,
    cancel: Arc<AtomicBool>,
    model_id: String,
    filename: String,
    total: u64,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(PROGRESS_INTERVAL);
        loop {
            interval.tick().await;
            if cancel.load(Ordering::Relaxed) {
                break;
            }

            let so_far = downloaded.load(Ordering::Relaxed);
            let percent = if total > 0 {
                (so_far as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            let _ = app.emit(
                "download-progress",
                DownloadProgress {
                    model_id: model_id.clone(),
                    filename: filename.clone(),
                    downloaded: so_far,
                    total,
                    percent,
                },
            );

            if total > 0 && so_far >= total {
                break;
            }
        }
    })
}

pub async fn download_parallel(
    app: AppHandle,
    client: Arc<reqwest::Client>,
    url: String,
    dest: PathBuf,
    total: u64,
    downloaded: Arc<AtomicU64>,
    cancel: Arc<AtomicBool>,
    model_id: String,
    filename: String,
    chunks: u64,
) -> Result<(), String> {
    {
        // Pre-allocate the file to prevent fragmentation and make concurrent
        // writes to non-overlapping regions safe from the outset.
        let f = tokio::fs::File::create(&dest)
            .await
            .map_err(|e| e.to_string())?;
        f.set_len(total).await.map_err(|e| e.to_string())?;
    }

    let progress_task = spawn_progress_reporter(
        app.clone(),
        downloaded.clone(),
        cancel.clone(),
        model_id.clone(),
        filename.clone(),
        total,
    );

    let chunk_size = (total + chunks - 1) / chunks;
    let mut join_set = JoinSet::new();

    for i in 0..chunks {
        let start = i * chunk_size;
        let end = ((i + 1) * chunk_size - 1).min(total - 1);

        if start >= total {
            break;
        }

        let client = Arc::clone(&client);
        let url = url.clone();
        let downloaded = Arc::clone(&downloaded);
        let cancel = Arc::clone(&cancel);
        let dest = dest.clone();

        join_set.spawn(async move {
            // Each chunk opens the pre-allocated file independently and seeks
            // once to its own start offset. From that point all writes within
            // this chunk are sequential, so BufWriter's coalescing is effective
            // and there are no cross-chunk seeks forcing premature flushes.
            //
            // This replaces the previous central writer task + MPSC channel,
            // which caused BufWriter to flush on every received frame because
            // interleaved chunk offsets triggered a seek before each write.
            let file = tokio::fs::OpenOptions::new()
                .write(true)
                .open(&dest)
                .await
                .map_err(|e| e.to_string())?;
            let mut file = BufWriter::new(file);
            file.seek(std::io::SeekFrom::Start(start))
                .await
                .map_err(|e| e.to_string())?;

            let mut current_start = start;
            let mut connect_attempts: u32 = 0;
            let mut stream_attempts: u32 = 0;

            loop {
                let total_attempts = connect_attempts + stream_attempts;
                if total_attempts > 0 {
                    let delay = 250 * (1u64 << total_attempts.min(4));
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                }

                // ── Connect phase ────────────────────────────────────────────
                let resp = client
                    .get(&url)
                    .header("Range", format!("bytes={}-{}", current_start, end))
                    .send()
                    .await;

                let resp = match resp {
                    Ok(r) if r.status().is_success() || r.status().as_u16() == 206 => r,
                    Ok(r) => {
                        return Err(format!(
                            "Chunk {} got unexpected status {}",
                            i,
                            r.status()
                        ));
                    }
                    Err(_) => {
                        connect_attempts += 1;
                        if connect_attempts >= MAX_CONNECT_RETRIES {
                            return Err(format!(
                                "Chunk {} failed to connect after {} attempts",
                                i, connect_attempts
                            ));
                        }
                        continue;
                    }
                };

                // ── Stream phase ─────────────────────────────────────────────
                //
                // On stream resume after a partial failure, re-seek to
                // current_start so the file handle is at the right position.
                // (It may have advanced during the failed attempt.)
                if stream_attempts > 0 {
                    file.seek(std::io::SeekFrom::Start(current_start))
                        .await
                        .map_err(|e| e.to_string())?;
                }

                let mut stream = resp.bytes_stream();
                let mut stream_error = false;

                while let Some(chunk_res) = stream.next().await {
                    if cancel.load(Ordering::Relaxed) {
                        return Ok(());
                    }

                    let chunk = match chunk_res {
                        Ok(c) => c,
                        Err(_) => {
                            stream_error = true;
                            break;
                        }
                    };

                    let len = chunk.len() as u64;
                    file.write_all(&chunk).await.map_err(|e| e.to_string())?;
                    current_start += len;
                    downloaded.fetch_add(len, Ordering::Relaxed);
                }

                if stream_error {
                    stream_attempts += 1;
                    if stream_attempts >= MAX_STREAM_RETRIES {
                        return Err(format!(
                            "Chunk {} stream failed after {} retries (resumed from byte {})",
                            i, stream_attempts, current_start
                        ));
                    }
                    continue;
                }

                // Flush before the task exits so userspace buffers are
                // committed. Each chunk flushes independently.
                file.flush().await.map_err(|e| e.to_string())?;
                return Ok::<(), String>(());
            }
        });
    }

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                cancel.store(true, Ordering::SeqCst);
                join_set.abort_all();
                progress_task.abort();
                return Err(e);
            }
            Err(join_err) => {
                cancel.store(true, Ordering::SeqCst);
                join_set.abort_all();
                progress_task.abort();
                return Err(join_err.to_string());
            }
        }
    }

    progress_task.abort();
    Ok(())
}

pub async fn download_stream(
    app: AppHandle,
    client: Arc<reqwest::Client>,
    url: String,
    dest: PathBuf,
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

    let progress_task = spawn_progress_reporter(
        app.clone(),
        downloaded.clone(),
        cancel.clone(),
        model_id,
        filename,
        total,
    );

    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        if cancel.load(Ordering::Relaxed) {
            progress_task.abort();
            return Ok(());
        }

        let chunk = chunk.map_err(|e| e.to_string())?;
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        downloaded.fetch_add(chunk.len() as u64, Ordering::Relaxed);
    }

    file.flush().await.map_err(|e| e.to_string())?;
    progress_task.abort();

    Ok(())
}