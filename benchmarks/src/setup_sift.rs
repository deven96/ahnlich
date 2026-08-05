//! Loads the SIFT dataset into a running ahnlich-db and writes the ghz payloads.
//!
//! Both stores hold the same vectors, so the only difference between runs is the search
//! path: `sift_linear` has no index, `sift_hnsw` does. Payloads are generated rather
//! than hand-written to keep the `algorithm` field matched to its store.

mod sift;

use ahnlich_client_rs::db::DbClient;
use ahnlich_types::algorithm::nonlinear::{HnswConfig, NonLinearIndex, non_linear_index};
use ahnlich_types::db::query::{CreateStore, DropStore, GetSimN, Set};
use ahnlich_types::keyval::{DbStoreEntry, StoreKey, StoreValue};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

const LINEAR_STORE: &str = "sift_linear";
const HNSW_STORE: &str = "sift_hnsw";

/// Vectors per `Set` request. The server caps messages at 10MB.
const DEFAULT_BATCH_SIZE: usize = 2_000;
const DEFAULT_CLOSEST_N: u64 = 10;
/// Matches the server default in `HNSWConfig::default()`.
const DEFAULT_EF_CONSTRUCTION: u32 = 100;

#[derive(Clone, Copy)]
struct Metric {
    index: ahnlich_types::algorithm::algorithms::DistanceMetric,
    linear_algorithm: &'static str,
}

impl Metric {
    fn parse(name: &str) -> Result<Self> {
        use ahnlich_types::algorithm::algorithms::DistanceMetric;
        Ok(match name.to_ascii_lowercase().as_str() {
            "euclidean" => Metric {
                index: DistanceMetric::Euclidean,
                linear_algorithm: "EuclideanDistance",
            },
            "cosine" => Metric {
                index: DistanceMetric::Cosine,
                linear_algorithm: "CosineSimilarity",
            },
            "dotproduct" => Metric {
                index: DistanceMetric::DotProduct,
                linear_algorithm: "DotProductSimilarity",
            },
            other => bail!("unknown distance metric {other:?} (euclidean|cosine|dotproduct)"),
        })
    }
}

/// `algorithm` is written as the enum name, which fails at parse time if wrong. A wrong
/// number is a valid request that selects a different search path.
#[derive(Serialize)]
struct GetSimNPayload {
    store: String,
    search_input: SearchInput,
    closest_n: u64,
    algorithm: &'static str,
}

#[derive(Serialize)]
struct SearchInput {
    key: Vec<f32>,
}

struct Config {
    db_addr: String,
    dataset_dir: PathBuf,
    payload_dir: PathBuf,
    metric: Metric,
    closest_n: u64,
    batch_size: usize,
    ef_construction: u32,
    store_size: Option<usize>,
}

impl Config {
    fn from_env() -> Result<Self> {
        Ok(Self {
            db_addr: env_or("AHNLICH_DB_ADDR", "127.0.0.1:1369"),
            dataset_dir: sift::dataset_dir(),
            payload_dir: PathBuf::from(env_or("PAYLOAD_DIR", ".")),
            metric: Metric::parse(&env_or("DISTANCE_METRIC", "euclidean"))?,
            closest_n: parse_env("CLOSEST_N", DEFAULT_CLOSEST_N)?,
            batch_size: parse_env("BATCH_SIZE", DEFAULT_BATCH_SIZE)?,
            ef_construction: parse_env("EF_CONSTRUCTION", DEFAULT_EF_CONSTRUCTION)?,
            store_size: match std::env::var("STORE_SIZE") {
                Err(_) => None,
                Ok(raw) => Some(
                    raw.parse()
                        .map_err(|err| anyhow::anyhow!("invalid STORE_SIZE={raw:?}: {err}"))?,
                ),
            },
        })
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_owned())
}

fn parse_env<T: std::str::FromStr>(key: &str, default: T) -> Result<T>
where
    T::Err: std::fmt::Display,
{
    match std::env::var(key) {
        Err(_) => Ok(default),
        Ok(raw) => raw
            .parse()
            .map_err(|err| anyhow::anyhow!("invalid {key}={raw:?}: {err}")),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env()?;

    println!("Loading SIFT dataset from {}", config.dataset_dir.display());
    let mut dataset = sift::load(&config.dataset_dir)?;
    if let Some(size) = config.store_size {
        dataset.base = sift::truncate(dataset.base, size)?;
    }
    let dimension = dataset.dimension();
    println!(
        "  {} base vectors, {} query vectors, dimension {dimension}",
        dataset.base.len(),
        dataset.queries.len()
    );

    println!("Connecting to ahnlich-db at {}", config.db_addr);
    let client = DbClient::new(config.db_addr.clone())
        .await
        .with_context(|| format!("could not connect to ahnlich-db at {}", config.db_addr))?;

    let entries: Vec<DbStoreEntry> = dataset
        .base
        .iter()
        .map(|vector| DbStoreEntry {
            key: Some(StoreKey {
                key: vector.clone(),
            }),
            value: Some(StoreValue {
                value: HashMap::new(),
            }),
        })
        .collect();

    let hnsw_index = NonLinearIndex {
        index: Some(non_linear_index::Index::Hnsw(HnswConfig {
            // Unset falls back to Euclidean.
            distance: Some(config.metric.index.into()),
            ef_construction: Some(config.ef_construction),
            ..Default::default()
        })),
    };

    populate(&client, LINEAR_STORE, dimension, vec![], &entries, &config).await?;
    populate(
        &client,
        HNSW_STORE,
        dimension,
        vec![hnsw_index],
        &entries,
        &config,
    )
    .await?;

    let probe = &dataset.queries[0];
    verify_answers(
        &client,
        LINEAR_STORE,
        config.metric.linear_algorithm,
        probe,
        config.closest_n,
    )
    .await?;
    verify_answers(&client, HNSW_STORE, "HNSW", probe, config.closest_n).await?;

    write_payload(
        &config.payload_dir.join("getsimn_linear.json"),
        LINEAR_STORE,
        config.metric.linear_algorithm,
        &dataset.queries,
        config.closest_n,
    )?;
    write_payload(
        &config.payload_dir.join("getsimn_hnsw.json"),
        HNSW_STORE,
        "HNSW",
        &dataset.queries,
        config.closest_n,
    )?;

    let ping_path = config.payload_dir.join("ping.json");
    std::fs::write(&ping_path, b"{}")
        .with_context(|| format!("could not write {}", ping_path.display()))?;
    println!("Wrote Ping payload to {}", ping_path.display());

    println!(
        "\nSetup complete. Both stores hold {} vectors.",
        entries.len()
    );
    Ok(())
}

/// Drop, recreate and fill a store.
async fn populate(
    client: &DbClient,
    store: &str,
    dimension: usize,
    non_linear_indices: Vec<NonLinearIndex>,
    entries: &[DbStoreEntry],
    config: &Config,
) -> Result<()> {
    let indexed = !non_linear_indices.is_empty();
    println!(
        "\nPreparing store {store:?} ({})",
        if indexed { "HNSW index" } else { "no index" }
    );

    client
        .drop_store(
            DropStore {
                store: store.to_owned(),
                error_if_not_exists: false,
                schema: None,
            },
            None,
        )
        .await
        .with_context(|| format!("failed to drop store {store}"))?;

    client
        .create_store(
            CreateStore {
                store: store.to_owned(),
                dimension: dimension as u32,
                create_predicates: vec![],
                non_linear_indices,
                error_if_exists: true,
                schema: None,
            },
            None,
        )
        .await
        .with_context(|| format!("failed to create store {store}"))?;

    let start = Instant::now();
    let mut inserted = 0;
    for (batch_index, batch) in entries.chunks(config.batch_size).enumerate() {
        client
            .set(
                Set {
                    store: store.to_owned(),
                    inputs: batch.to_vec(),
                    schema: None,
                },
                None,
            )
            .await
            .with_context(|| format!("failed to insert batch {batch_index} into {store}"))?;

        inserted += batch.len();
        println!("  inserted {inserted}/{}", entries.len());
    }
    println!("  filled in {:?}", start.elapsed());

    Ok(())
}

/// Run one query per store and fail unless it returns `closest_n` results.
async fn verify_answers(
    client: &DbClient,
    store: &str,
    algorithm: &str,
    query: &[f32],
    closest_n: u64,
) -> Result<()> {
    let response = client
        .get_sim_n(
            GetSimN {
                store: store.to_owned(),
                search_input: Some(StoreKey {
                    key: query.to_vec(),
                }),
                closest_n,
                algorithm: algorithm_number(algorithm),
                condition: None,
                schema: None,
            },
            None,
        )
        .await
        .with_context(|| format!("{store}: {algorithm} query failed"))?;

    if response.entries.len() as u64 != closest_n {
        bail!(
            "{store}: {algorithm} returned {} entries, expected {closest_n}",
            response.entries.len()
        );
    }
    println!("Verified {store} answers {algorithm} queries");
    Ok(())
}

fn algorithm_number(name: &str) -> i32 {
    use ahnlich_types::algorithm::algorithms::Algorithm;
    match name {
        "EuclideanDistance" => Algorithm::EuclideanDistance,
        "CosineSimilarity" => Algorithm::CosineSimilarity,
        "DotProductSimilarity" => Algorithm::DotProductSimilarity,
        "HNSW" => Algorithm::Hnsw,
        other => unreachable!("unknown algorithm {other}"),
    }
    .into()
}

/// One message per query vector. ghz cycles the array.
fn write_payload(
    path: &Path,
    store: &str,
    algorithm: &'static str,
    queries: &[Vec<f32>],
    closest_n: u64,
) -> Result<()> {
    let payloads: Vec<GetSimNPayload> = queries
        .iter()
        .map(|query| GetSimNPayload {
            store: store.to_owned(),
            search_input: SearchInput { key: query.clone() },
            closest_n,
            algorithm,
        })
        .collect();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("could not create {}", parent.display()))?;
    }
    std::fs::write(path, serde_json::to_vec(&payloads)?)
        .with_context(|| format!("could not write {}", path.display()))?;

    println!(
        "Wrote {} {algorithm} payloads to {}",
        payloads.len(),
        path.display()
    );
    Ok(())
}
