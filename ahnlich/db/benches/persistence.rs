use ahnlich_db::engine::store::StoreHandler;
use ahnlich_types::algorithm::algorithms::Algorithm;
use ahnlich_types::algorithm::nonlinear::{HnswConfig, NonLinearAlgorithm};
use ahnlich_types::keyval::{StoreKey, StoreName, StoreValue};
use ahnlich_types::metadata::MetadataValue;
use criterion::{criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tempfile::TempDir;

fn criterion_config(seconds: u64, sample_size: usize) -> Criterion {
    Criterion::default()
        .measurement_time(std::time::Duration::new(seconds, 0))
        .sample_size(sample_size)
}

fn bench_persistence(_c: &mut Criterion) {
    // TODO: implement benchmarks
}

criterion_group! {
    name = persistence;
    config = criterion_config(30, 10);
    targets = bench_persistence
}

criterion_main!(persistence);
