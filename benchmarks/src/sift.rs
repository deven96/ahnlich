//! Reader for the SIFT dataset in `.fvecs` format: a sequence of records, each a
//! little-endian `i32` dimension followed by that many 4-byte floats.
//!
//! Duplicates `ahnlich/similarity/src/tests/fixtures/sift.rs`, which is behind
//! `#[cfg(test)]` and cannot be imported from another crate.

use anyhow::{Context, Result, bail};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

/// Relative to the manifest, so the binary works from any directory.
const SIFT_10K_DATASET_DIR: &str = "../ahnlich/similarity/sift_10k";
const SIFT_1M_DATASET_DIR: &str = "../ahnlich/similarity/sift_1m";

/// Rejects a corrupt header before it becomes a multi-gigabyte allocation.
const MAX_DIMENSION: usize = 4096;

pub struct Dataset {
    pub base: Vec<Vec<f32>>,
    pub queries: Vec<Vec<f32>>,
}

impl Dataset {
    pub fn dimension(&self) -> usize {
        self.base[0].len()
    }
}

pub fn dataset_dir() -> Result<PathBuf> {
    if let Some(dir) = std::env::var_os("SIFT_DIR") {
        return Ok(PathBuf::from(dir));
    }

    let relative_dir = match std::env::var("SIFT_DATASET") {
        Err(std::env::VarError::NotPresent) => SIFT_10K_DATASET_DIR,
        Ok(dataset) if dataset == "sift_10k" => SIFT_10K_DATASET_DIR,
        Ok(dataset) if dataset == "sift_1m" => SIFT_1M_DATASET_DIR,
        Err(err) => return Err(err).context("invalid SIFT_DATASET"),
        Ok(dataset) => bail!("unknown SIFT_DATASET={dataset:?}; expected sift_10k or sift_1m"),
    };

    Ok(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_dir))
}

/// Truncate the base set to `size`. Errors if the dataset holds fewer vectors.
pub fn truncate(mut base: Vec<Vec<f32>>, size: usize) -> Result<Vec<Vec<f32>>> {
    if size == 0 {
        bail!("STORE_SIZE must be greater than zero");
    }
    if size > base.len() {
        bail!(
            "STORE_SIZE={size} exceeds the {} vectors available; point SIFT_DIR at a \
             larger dataset",
            base.len()
        );
    }
    base.truncate(size);
    Ok(base)
}

/// Find a dataset file by suffix, matching both `siftsmall_base.fvecs` and
/// `sift_base.fvecs`.
fn find(dir: &Path, suffix: &str) -> Result<PathBuf> {
    let mut matches: Vec<PathBuf> = std::fs::read_dir(dir)
        .with_context(|| format!("could not read dataset dir {}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(suffix))
        })
        .collect();
    matches.sort();

    match matches.len() {
        0 => bail!("no *{suffix} in {}", dir.display()),
        1 => Ok(matches.remove(0)),
        _ => bail!(
            "several *{suffix} in {}: {}",
            dir.display(),
            matches
                .iter()
                .filter_map(|path| path.file_name().and_then(|name| name.to_str()))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

pub fn load(dir: &Path) -> Result<Dataset> {
    let base = read_fvecs(&find(dir, "base.fvecs")?)?;
    let queries = read_fvecs(&find(dir, "query.fvecs")?)?;

    if base.is_empty() {
        bail!("no base vectors found in {}", dir.display());
    }
    if queries.is_empty() {
        bail!("no query vectors found in {}", dir.display());
    }

    let dimension = base[0].len();
    if let Some(bad) = queries.iter().find(|q| q.len() != dimension) {
        bail!(
            "query dimension {} does not match base dimension {dimension}",
            bad.len()
        );
    }

    Ok(Dataset { base, queries })
}

pub fn read_fvecs(path: &Path) -> Result<Vec<Vec<f32>>> {
    let file =
        File::open(path).with_context(|| format!("failed to open dataset {}", path.display()))?;
    let mut reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut header = [0u8; 4];
    let mut buffer = Vec::new();

    // A short read on the header is end of file; on the body it is corruption.
    while reader.read_exact(&mut header).is_ok() {
        let dimension = i32::from_le_bytes(header);
        let dimension = usize::try_from(dimension)
            .with_context(|| format!("negative dimension {dimension} in {}", path.display()))?;

        if dimension == 0 || dimension > MAX_DIMENSION {
            bail!(
                "implausible dimension {dimension} in {}, is this a .fvecs file?",
                path.display()
            );
        }

        buffer.resize(dimension * 4, 0);
        reader
            .read_exact(&mut buffer)
            .with_context(|| format!("truncated record in {}", path.display()))?;

        records.push(
            buffer
                .chunks_exact(4)
                .map(|chunk| {
                    f32::from_le_bytes(chunk.try_into().expect("chunks_exact(4) yields 4 bytes"))
                })
                .collect(),
        );
    }

    Ok(records)
}
