use crate::{ProgressKind, ProgressReporter};
use anyhow::Result;
use sha2::Digest;
use std::path::PathBuf;
use std::time::Instant;
use tracing::info;

// TODO: retrive the list of models to be able to update it OTA?
// TODO: add a progressbar for model download
// TODO: use a real hosting service

struct ModelSource {
    url: &'static str,
    sha256: &'static str,
}

static MODEL_SOURCE: ModelSource =
    // model_augment_all.onnx
    ModelSource {
        url: "https://github.com/DCNick3/mangai-models/releases/download/v0.0.0/model_augment_all.onnx",
        sha256: "23e9b3e030eae4b614ad477dd726ee7605e8e0c211b500bcf6363fba2db3f3d4",
    }
    // model_augment_all_2.onnx
    // ""
    // model_augment_all_more_train.onnx
    // ModelSource {
    //     url: "https://github.com/Dinislam36/Auto_manga_cleaner/releases/download/v0.1.1/model_augment_all_more_train.onnx",
    //     sha256: "6df90cb764092f574f4c4916aef84e45f773339dcdc6c7274a0c1b7096e0ac65",
    // }
;

fn get_cache_dir() -> Result<PathBuf> {
    let dirs = directories::BaseDirs::new()
        .ok_or_else(|| anyhow::anyhow!("failed to get base directories"))?;
    let cache_dir = dirs.cache_dir();
    let cache_dir = cache_dir.join("mangai-clean");
    std::fs::create_dir_all(&cache_dir)?;

    Ok(cache_dir)
}

fn get_cache_path(source: &ModelSource) -> Result<PathBuf> {
    let cache_dir = get_cache_dir()?;
    let cache_path = cache_dir.join(format!("{}.onnx", source.sha256));
    Ok(cache_path)
}

fn get_from_cache(source: &ModelSource) -> Result<Option<Vec<u8>>> {
    let cache_path = get_cache_path(source)?;
    if cache_path.exists() {
        let bytes = std::fs::read(&cache_path)?;
        let mut hasher = sha2::Sha256::new();
        hasher.update(&bytes);
        let hash = hasher.finalize();
        let hash = hex::encode(hash);
        if hash == source.sha256 {
            return Ok(Some(bytes));
        } else {
            std::fs::remove_file(&cache_path)?;
        }
    }
    Ok(None)
}

fn write_to_cache(source: &ModelSource, bytes: &[u8]) -> Result<()> {
    let cache_path = get_cache_path(source)?;
    std::fs::write(&cache_path, bytes)?;
    Ok(())
}

pub fn get_model(progress: &mut dyn ProgressReporter) -> Result<Vec<u8>> {
    info!("Looking for model {}.onnx...", MODEL_SOURCE.sha256);
    info!("Cache dir: {:?}", get_cache_dir()?);

    if let Some(data) = get_from_cache(&MODEL_SOURCE)? {
        info!("found model in cache");
        return Ok(data);
    }
    info!("model not found in cache, downloading");

    let req = ureq::get(MODEL_SOURCE.url);
    let resp = req.call()?;

    let len = resp
        .header("Content-Length")
        .and_then(|len| len.parse::<usize>().ok())
        .unwrap_or(120038965); // this... works ig

    progress.init(ProgressKind::Bytes, "Downloading model", len);

    let mut bytes = Vec::new();
    let mut reader = resp.into_reader();
    let mut buffer = [0; 1024 * 1024];
    let mut total_read = 0;
    let mut prev_progress = Instant::now();
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        total_read += read;
        bytes.extend_from_slice(&buffer[..read]);

        if prev_progress.elapsed().as_millis() >= 250 {
            progress.progress(total_read);
            prev_progress = Instant::now();
        }
    }
    progress.finish();

    let hash = sha2::Sha256::digest(&bytes);
    let hash = hex::encode(hash);

    if hash != MODEL_SOURCE.sha256 {
        anyhow::bail!("Download hash mismatch");
    }

    info!("model downloaded, writing to cache");

    write_to_cache(&MODEL_SOURCE, &bytes)?;

    Ok(bytes)
}
