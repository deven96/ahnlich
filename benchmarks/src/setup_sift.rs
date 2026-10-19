//! Loads the SIFT dataset into a running ahnlich-db and writes the ghz payloads.
//!
//! All stores hold the same vectors. `sift_linear` has no index, `sift_hnsw` has an
//! HNSW index, and `sift_linear_indexed` has predicate indexes for selective predicate
//! workloads. Payloads are generated rather than hand-written to keep the `algorithm`
//! field matched to its store.

mod sift;

use ahnlich_client_rs::db::DbClient;
use ahnlich_types::algorithm::nonlinear::{HnswConfig, NonLinearIndex, non_linear_index};
use ahnlich_types::db::query::{CreateStore, DropStore, GetSimN, Set};
use ahnlich_types::keyval::{DbStoreEntry, StoreKey, StoreValue};
use ahnlich_types::metadata::{MetadataValue, metadata_value};
use ahnlich_types::predicates::{
    AndCondition, Equals, In, OrCondition, Predicate, PredicateCondition, predicate,
    predicate_condition,
};
use anyhow::{Context, Result, bail};
use serde::{Serialize, Serializer};
use std::path::{Path, PathBuf};
use std::time::Instant;

fn string_value(s: &str) -> MetadataValue {
    MetadataValue {
        value: Some(metadata_value::Value::RawString(s.to_string())),
    }
}

fn assign_metadata(
    vector_index: usize,
    store_size: usize,
) -> std::collections::HashMap<String, MetadataValue> {
    let mut metadata = std::collections::HashMap::new();
    let one_percent = store_size / 100;
    let ten_percent = store_size / 10;
    let half = store_size / 2;

    if vector_index < one_percent {
        // 1% selectivity: all three conditions match
        metadata.insert("category".to_string(), string_value("electronics"));
        metadata.insert("price_range".to_string(), string_value("high"));
        metadata.insert("in_stock".to_string(), string_value("true"));
    } else if vector_index < ten_percent {
        // 10% selectivity: category + in_stock match
        metadata.insert("category".to_string(), string_value("electronics"));
        metadata.insert("price_range".to_string(), string_value("low"));
        metadata.insert("in_stock".to_string(), string_value("true"));
    } else if vector_index < half {
        // 50% selectivity: only category matches
        metadata.insert("category".to_string(), string_value("electronics"));
        metadata.insert("price_range".to_string(), string_value("mid"));
        metadata.insert("in_stock".to_string(), string_value("false"));
    } else {
        // No match: different category
        let categories = ["books", "clothing", "home", "toys", "sports"];
        let category = categories[(vector_index / 1000) % categories.len()];
        metadata.insert("category".to_string(), string_value(category));
        metadata.insert("price_range".to_string(), string_value("mid"));
        metadata.insert("in_stock".to_string(), string_value("true"));
    }

    metadata
}

fn equals_predicate(key: &str, value: &str) -> PredicateCondition {
    PredicateCondition {
        kind: Some(predicate_condition::Kind::Value(Predicate {
            kind: Some(predicate::Kind::Equals(Equals {
                key: key.to_string(),
                value: Some(string_value(value)),
            })),
        })),
    }
}

fn and_predicate(left: PredicateCondition, right: PredicateCondition) -> PredicateCondition {
    PredicateCondition {
        kind: Some(predicate_condition::Kind::And(Box::new(AndCondition {
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }))),
    }
}

fn or_predicate(left: PredicateCondition, right: PredicateCondition) -> PredicateCondition {
    PredicateCondition {
        kind: Some(predicate_condition::Kind::Or(Box::new(OrCondition {
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }))),
    }
}

fn in_predicate(key: &str, values: &[&str]) -> PredicateCondition {
    PredicateCondition {
        kind: Some(predicate_condition::Kind::Value(Predicate {
            kind: Some(predicate::Kind::In(In {
                key: key.to_owned(),
                values: values.iter().map(|value| string_value(value)).collect(),
            })),
        })),
    }
}

fn predicate_50_percent() -> PredicateCondition {
    equals_predicate("category", "electronics")
}

fn predicate_10_percent() -> PredicateCondition {
    and_predicate(
        equals_predicate("category", "electronics"),
        equals_predicate("in_stock", "true"),
    )
}

fn predicate_1_percent() -> PredicateCondition {
    and_predicate(
        and_predicate(
            equals_predicate("category", "electronics"),
            equals_predicate("price_range", "high"),
        ),
        equals_predicate("in_stock", "true"),
    )
}

const BENCHMARK_5K_FIELD: &str = "benchmark_5k";
const BENCHMARK_1K_FIELD: &str = "benchmark_1k";
const BENCHMARK_100_FIELD: &str = "benchmark_100";
const BENCHMARK_MATCH_VALUE: &str = "match";
const BENCHMARK_GROUP_5K_FIELD: &str = "benchmark_group_5k";
const BENCHMARK_GROUP_1K_FIELD: &str = "benchmark_group_1k";
const BENCHMARK_GROUP_100_FIELD: &str = "benchmark_group_100";
const BENCHMARK_GROUP_VALUES: [&str; 3] = ["a", "b", "c"];
// The suffixes are SIFT10k reference counts; divisors preserve selectivity at other sizes.
const INDEXED_SELECTIVITIES: [(&str, &str, &str, usize); 3] = [
    ("5k", BENCHMARK_5K_FIELD, BENCHMARK_GROUP_5K_FIELD, 2),
    ("1k", BENCHMARK_1K_FIELD, BENCHMARK_GROUP_1K_FIELD, 10),
    ("100", BENCHMARK_100_FIELD, BENCHMARK_GROUP_100_FIELD, 100),
];

fn indexed_metadata(
    vector_index: usize,
    store_size: usize,
) -> std::collections::HashMap<String, MetadataValue> {
    let mut metadata = assign_metadata(vector_index, store_size);
    for (_, equality_field, group_field, divisor) in INDEXED_SELECTIVITIES {
        let match_count = store_size / divisor;
        if vector_index < match_count {
            metadata.insert(
                equality_field.to_owned(),
                string_value(BENCHMARK_MATCH_VALUE),
            );
            let group = vector_index * BENCHMARK_GROUP_VALUES.len() / match_count;
            metadata.insert(
                group_field.to_owned(),
                string_value(BENCHMARK_GROUP_VALUES[group]),
            );
        }
    }
    metadata
}

fn indexed_equality_filters() -> [(&'static str, PredicateCondition); 4] {
    [
        (
            "5k",
            equals_predicate(BENCHMARK_5K_FIELD, BENCHMARK_MATCH_VALUE),
        ),
        (
            "1k",
            equals_predicate(BENCHMARK_1K_FIELD, BENCHMARK_MATCH_VALUE),
        ),
        (
            "100",
            equals_predicate(BENCHMARK_100_FIELD, BENCHMARK_MATCH_VALUE),
        ),
        ("miss", equals_predicate(BENCHMARK_100_FIELD, "absent")),
    ]
}

fn indexed_planner_filters() -> Vec<(String, PredicateCondition)> {
    let mut filters = Vec::new();
    for (suffix, equality_field, group_field, _) in INDEXED_SELECTIVITIES {
        let equality = || equals_predicate(equality_field, BENCHMARK_MATCH_VALUE);
        let group = |value| equals_predicate(group_field, value);
        let indexed_or = || or_predicate(group("a"), or_predicate(group("b"), group("c")));
        let unindexed_scope = || {
            let category = equals_predicate("category", "electronics");
            match suffix {
                "5k" => and_predicate(
                    category,
                    or_predicate(
                        equals_predicate("in_stock", "true"),
                        equals_predicate("in_stock", "false"),
                    ),
                ),
                "1k" => and_predicate(category, equals_predicate("in_stock", "true")),
                "100" => and_predicate(category, equals_predicate("price_range", "high")),
                _ => unreachable!("indexed selectivity suffix is fixed"),
            }
        };
        let unindexed_subset = || equals_predicate("price_range", "high");

        filters.push((
            format!("in_{suffix}"),
            in_predicate(group_field, &BENCHMARK_GROUP_VALUES),
        ));
        filters.push((
            format!("and_{suffix}"),
            and_predicate(equality(), indexed_or()),
        ));
        filters.push((
            format!("and_partial_{suffix}"),
            and_predicate(
                equality(),
                or_predicate(unindexed_scope(), unindexed_subset()),
            ),
        ));
        filters.push((format!("or_{suffix}"), indexed_or()));
        filters.push((
            format!("or_partial_{suffix}"),
            or_predicate(
                equality(),
                and_predicate(unindexed_scope(), unindexed_subset()),
            ),
        ));
    }
    filters
}

fn indexed_filters() -> Vec<(String, PredicateCondition)> {
    indexed_equality_filters()
        .into_iter()
        .map(|(suffix, condition)| (suffix.to_owned(), condition))
        .chain(indexed_planner_filters())
        .collect()
}

fn serialize_predicate_condition<S>(
    condition: &Option<PredicateCondition>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match condition {
        None => serializer.serialize_none(),
        Some(c) => {
            let json = predicate_condition_to_json(c);
            json.serialize(serializer)
        }
    }
}

fn predicate_condition_to_json(condition: &PredicateCondition) -> serde_json::Value {
    use serde_json::json;

    match &condition.kind {
        None => json!(null),
        Some(predicate_condition::Kind::Value(predicate)) => {
            json!({
                "value": predicate_to_json(predicate)
            })
        }
        Some(predicate_condition::Kind::And(and_cond)) => {
            json!({
                "and": {
                    "left": and_cond.left.as_ref().map(|l| predicate_condition_to_json(l)),
                    "right": and_cond.right.as_ref().map(|r| predicate_condition_to_json(r)),
                }
            })
        }
        Some(predicate_condition::Kind::Or(or_cond)) => {
            json!({
                "or": {
                    "left": or_cond.left.as_ref().map(|l| predicate_condition_to_json(l)),
                    "right": or_cond.right.as_ref().map(|r| predicate_condition_to_json(r)),
                }
            })
        }
    }
}

fn predicate_to_json(predicate: &Predicate) -> serde_json::Value {
    use serde_json::json;

    match &predicate.kind {
        None => json!(null),
        Some(predicate::Kind::Equals(equals)) => {
            json!({
                "equals": {
                    "key": equals.key,
                    "value": metadata_value_to_json(equals.value.as_ref()),
                }
            })
        }
        Some(predicate::Kind::NotEquals(not_equals)) => {
            json!({
                "notEquals": {
                    "key": not_equals.key,
                    "value": metadata_value_to_json(not_equals.value.as_ref()),
                }
            })
        }
        Some(predicate::Kind::In(in_pred)) => {
            json!({
                "in": {
                    "key": in_pred.key,
                    "values": in_pred.values.iter().map(|v| metadata_value_to_json(Some(v))).collect::<Vec<_>>(),
                }
            })
        }
        Some(predicate::Kind::NotIn(not_in)) => {
            json!({
                "notIn": {
                    "key": not_in.key,
                    "values": not_in.values.iter().map(|v| metadata_value_to_json(Some(v))).collect::<Vec<_>>(),
                }
            })
        }
    }
}

fn metadata_value_to_json(value: Option<&MetadataValue>) -> serde_json::Value {
    use serde_json::json;

    match value.and_then(|v| v.value.as_ref()) {
        None => json!(null),
        Some(metadata_value::Value::RawString(s)) => json!({ "rawString": s }),
        Some(metadata_value::Value::Image(img)) => json!({ "image": img }),
        Some(metadata_value::Value::Audio(audio)) => json!({ "audio": audio }),
    }
}

const LINEAR_STORE: &str = "sift_linear";
const INDEXED_LINEAR_STORE: &str = "sift_linear_indexed";
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
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_predicate_condition"
    )]
    condition: Option<PredicateCondition>,
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
            dataset_dir: sift::dataset_dir()?,
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
    if dataset.base.len() / 100 < config.closest_n as usize {
        bail!(
            "1% of STORE_SIZE={} yields fewer than CLOSEST_N={} matches",
            dataset.base.len(),
            config.closest_n
        );
    }

    println!("Connecting to ahnlich-db at {}", config.db_addr);
    let client = DbClient::new(config.db_addr.clone())
        .await
        .with_context(|| format!("could not connect to ahnlich-db at {}", config.db_addr))?;

    let entries: Vec<DbStoreEntry> = dataset
        .base
        .iter()
        .enumerate()
        .map(|(i, vector)| DbStoreEntry {
            key: Some(StoreKey {
                key: vector.clone(),
            }),
            value: Some(StoreValue {
                value: assign_metadata(i, dataset.base.len()),
            }),
        })
        .collect();

    let indexed_entries: Vec<DbStoreEntry> = dataset
        .base
        .iter()
        .enumerate()
        .map(|(i, vector)| DbStoreEntry {
            key: Some(StoreKey {
                key: vector.clone(),
            }),
            value: Some(StoreValue {
                value: indexed_metadata(i, dataset.base.len()),
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

    populate(
        &client,
        LINEAR_STORE,
        dimension,
        vec![],
        vec![],
        &entries,
        &config,
    )
    .await?;
    populate(
        &client,
        INDEXED_LINEAR_STORE,
        dimension,
        vec![
            BENCHMARK_5K_FIELD.to_owned(),
            BENCHMARK_1K_FIELD.to_owned(),
            BENCHMARK_100_FIELD.to_owned(),
            BENCHMARK_GROUP_5K_FIELD.to_owned(),
            BENCHMARK_GROUP_1K_FIELD.to_owned(),
            BENCHMARK_GROUP_100_FIELD.to_owned(),
        ],
        vec![],
        &indexed_entries,
        &config,
    )
    .await?;
    populate(
        &client,
        HNSW_STORE,
        dimension,
        vec![],
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
    let indexed_filters = indexed_filters();
    for (suffix, condition) in &indexed_filters {
        if suffix == "miss" {
            continue;
        }
        verify_answers_with_condition(
            &client,
            INDEXED_LINEAR_STORE,
            config.metric.linear_algorithm,
            probe,
            config.closest_n,
            condition.clone(),
        )
        .await?;
    }
    verify_answers(&client, HNSW_STORE, "HNSW", probe, config.closest_n).await?;

    // Generate base payloads (no filter)
    let linear_payload = config.payload_dir.join("getsimn_linear.json");
    write_payload_with_condition(
        &linear_payload,
        LINEAR_STORE,
        config.metric.linear_algorithm,
        &dataset.queries,
        config.closest_n,
        None,
    )?;

    let hnsw_payload = config.payload_dir.join("getsimn_hnsw.json");
    write_payload_with_condition(
        &hnsw_payload,
        HNSW_STORE,
        "HNSW",
        &dataset.queries,
        config.closest_n,
        None,
    )?;

    // Generate filtered payloads
    let filters = [
        ("5k", predicate_50_percent()),
        ("1k", predicate_10_percent()),
        ("100", predicate_1_percent()),
    ];

    for (suffix, predicate) in &filters {
        let linear_path = config
            .payload_dir
            .join(format!("getsimn_linear_{}.json", suffix));
        write_payload_with_condition(
            &linear_path,
            LINEAR_STORE,
            config.metric.linear_algorithm,
            &dataset.queries,
            config.closest_n,
            Some(predicate.clone()),
        )?;

        let hnsw_path = config
            .payload_dir
            .join(format!("getsimn_hnsw_{}.json", suffix));
        write_payload_with_condition(
            &hnsw_path,
            HNSW_STORE,
            "HNSW",
            &dataset.queries,
            config.closest_n,
            Some(predicate.clone()),
        )?;
    }

    for (suffix, predicate) in indexed_filters {
        let path = config
            .payload_dir
            .join(format!("getsimn_linear_indexed_{suffix}.json"));
        write_payload_with_condition(
            &path,
            INDEXED_LINEAR_STORE,
            config.metric.linear_algorithm,
            &dataset.queries,
            config.closest_n,
            Some(predicate),
        )?;
    }

    let ping_path = config.payload_dir.join("ping.json");
    std::fs::write(&ping_path, b"{}")
        .with_context(|| format!("could not write {}", ping_path.display()))?;
    println!("Wrote Ping payload to {}", ping_path.display());

    println!(
        "\nSetup complete. All stores hold {} vectors.",
        entries.len()
    );
    Ok(())
}

/// Drop, recreate and fill a store.
async fn populate(
    client: &DbClient,
    store: &str,
    dimension: usize,
    create_predicates: Vec<String>,
    non_linear_indices: Vec<NonLinearIndex>,
    entries: &[DbStoreEntry],
    config: &Config,
) -> Result<()> {
    let index_description = if !non_linear_indices.is_empty() {
        "HNSW index"
    } else if !create_predicates.is_empty() {
        "predicate indexes"
    } else {
        "no index"
    };
    println!("\nPreparing store {store:?} ({})", index_description);

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
                create_predicates,
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

async fn verify_answers_with_condition(
    client: &DbClient,
    store: &str,
    algorithm: &str,
    query: &[f32],
    closest_n: u64,
    condition: PredicateCondition,
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
                condition: Some(condition),
                schema: None,
            },
            None,
        )
        .await
        .with_context(|| format!("{store}: filtered {algorithm} query failed"))?;

    if response.entries.len() as u64 != closest_n {
        bail!(
            "{store}: filtered {algorithm} returned {} entries, expected {closest_n}",
            response.entries.len()
        );
    }
    println!("Verified {store} answers filtered {algorithm} queries");
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
fn write_payload_with_condition(
    path: &Path,
    store: &str,
    algorithm: &'static str,
    queries: &[Vec<f32>],
    closest_n: u64,
    condition: Option<PredicateCondition>,
) -> Result<()> {
    let payloads: Vec<GetSimNPayload> = queries
        .iter()
        .map(|q| GetSimNPayload {
            store: store.to_string(),
            search_input: SearchInput { key: q.clone() },
            closest_n,
            algorithm,
            condition: condition.clone(),
        })
        .collect();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("could not create {}", parent.display()))?;
    }

    let json = serde_json::to_string_pretty(&payloads).context("serializing payloads")?;
    std::fs::write(path, json).with_context(|| format!("writing {}", path.display()))?;

    println!(
        "Wrote {} {} payloads to {}",
        queries.len(),
        if condition.is_some() {
            "filtered"
        } else {
            "unfiltered"
        },
        path.display()
    );
    Ok(())
}
