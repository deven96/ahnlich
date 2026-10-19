//! Deterministic Set fixtures for the Set benchmark runners. No timing is done here.
use ahnlich_client_rs::db::DbClient;
use ahnlich_types::algorithm::algorithms::Algorithm;
use ahnlich_types::algorithm::nonlinear::{HnswConfig, NonLinearIndex, non_linear_index};
use ahnlich_types::db::query::{CreateStore, DropStore, GetKey, GetPred, GetSimN, Set};
use ahnlich_types::keyval::{DbStoreEntry, StoreKey, StoreValue};
use ahnlich_types::metadata::{MetadataValue, metadata_value::Value};
use ahnlich_types::predicates::{
    Equals, Predicate, PredicateCondition, predicate, predicate_condition,
};
use anyhow::{Context, Result, bail, ensure};
use serde::Deserialize;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::io::{BufWriter, Write};
use std::path::Path;

const HNSW_VERIFICATION_SHARD_SIZE: usize = 64;
const HNSW_VERIFICATION_FIELD: &str = "benchmark_request_shard";

#[derive(Deserialize)]
struct Spec {
    store: String,
    batch: usize,
    dimension: usize,
    indexed_fields: usize,
    cardinality: usize,
    pool_requests: usize,
    payload_file: String,
    #[serde(default)]
    workload: Workload,
    #[serde(default)]
    hnsw: bool,
    #[serde(default)]
    expected_requests: usize,
}

#[derive(Default, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Workload {
    #[default]
    Update,
    Insert,
}

impl Spec {
    fn entry(&self, request: usize, row: usize) -> DbStoreEntry {
        // First two coordinates provide exact, collision-free f32 fixture identity.
        let mut key: Vec<f32> = (0..self.dimension)
            .map(|d| ((row * 17 + d * 31) % 101) as f32)
            .collect();
        key[0] = (request + 1) as f32;
        key[1] = (row + 1) as f32;
        let bucket = (request * self.batch + row) % self.cardinality;
        let mut value: HashMap<_, _> = (0..4)
            .map(|field| (format!("field-{field}"), string_value(format!("v{bucket}"))))
            .collect();
        value.insert("payload".into(), string_value("x".repeat(64)));
        DbStoreEntry {
            key: Some(StoreKey { key }),
            value: Some(StoreValue { value }),
        }
    }

    fn seed(&self) -> DbStoreEntry {
        DbStoreEntry {
            key: Some(StoreKey {
                key: vec![-1.0; self.dimension],
            }),
            value: Some(StoreValue {
                value: (0..4)
                    .map(|i| (format!("field-{i}"), string_value("seed".into())))
                    .collect(),
            }),
        }
    }

    fn insert_key(&self, request: usize, row: usize) -> StoreKey {
        StoreKey {
            key: (0..self.dimension)
                .map(|dimension| match dimension {
                    0 => request as f32,
                    1 => (row + 1) as f32,
                    _ => ((row * 17 + dimension * 31) % 101) as f32,
                })
                .collect(),
        }
    }
}

fn string_value(value: String) -> MetadataValue {
    MetadataValue {
        value: Some(Value::RawString(value)),
    }
}

fn write_payload(s: &Spec) -> Result<()> {
    if s.workload == Workload::Insert {
        return write_insert_payload(s);
    }

    let mut out = BufWriter::new(std::fs::File::create(&s.payload_file)?);
    out.write_all(b"[")?;
    for request in 0..s.pool_requests {
        if request != 0 {
            out.write_all(b",")?;
        }
        let inputs: Vec<_> = (0..s.batch)
            .map(|row| {
                let entry = s.entry(request, row);
                let values: HashMap<_, _> = entry
                    .value
                    .unwrap()
                    .value
                    .into_iter()
                    .map(|(key, value)| {
                        let Some(Value::RawString(value)) = value.value else {
                            unreachable!()
                        };
                        (key, json!({"rawString": value}))
                    })
                    .collect();
                json!({"key": {"key": entry.key.unwrap().key}, "value": {"value": values}})
            })
            .collect();
        serde_json::to_writer(&mut out, &json!({"store": s.store, "inputs": inputs}))?;
    }
    out.write_all(b"]")?;
    out.flush()?;
    Ok(())
}

fn write_insert_payload(s: &Spec) -> Result<()> {
    const REQUEST_NUMBER: &str = "__GHZ_REQUEST_NUMBER__";

    let inputs: Vec<_> = (0..s.batch)
        .map(|row| {
            let key: Vec<_> = (0..s.dimension)
                .map(|dimension| match dimension {
                    0 => json!(REQUEST_NUMBER),
                    1 => json!(row + 1),
                    _ => json!(((row * 17 + dimension * 31) % 101) as f32),
                })
                .collect();
            let bucket = row % s.cardinality;
            let mut values: HashMap<_, _> = (0..4)
                .map(|field| {
                    (
                        format!("field-{field}"),
                        json!({"rawString": format!("v{bucket}")}),
                    )
                })
                .collect();
            values.insert("payload".into(), json!({"rawString": "x".repeat(64)}));
            values.insert(
                HNSW_VERIFICATION_FIELD.into(),
                json!({"rawString": format!("{REQUEST_NUMBER}:{}", row / HNSW_VERIFICATION_SHARD_SIZE)}),
            );
            json!({"key": {"key": key}, "value": {"value": values}})
        })
        .collect();

    let payload = serde_json::to_string(&json!({"store": s.store, "inputs": inputs}))?
        .replace(&format!("\"{REQUEST_NUMBER}\""), "{{.RequestNumber}}")
        .replace(REQUEST_NUMBER, "{{.RequestNumber}}");
    std::fs::write(&s.payload_file, payload)?;
    Ok(())
}

async fn insert(client: &DbClient, s: &Spec, inputs: Vec<DbStoreEntry>) -> Result<()> {
    let count = inputs.len() as u64;
    let result = client
        .set(
            Set {
                store: s.store.clone(),
                inputs,
                schema: None,
            },
            None,
        )
        .await?;
    let result = result.upsert.context("Set response missing counts")?;
    ensure!(
        result.inserted == count && result.updated == 0,
        "fixture IDs overlap"
    );
    Ok(())
}

async fn check_catalog(client: &DbClient, s: &Spec, expected_len: usize) -> Result<()> {
    let info = client.get_store(s.store.clone(), None).await?;
    ensure!(
        info.len == expected_len as u64,
        "store length: expected {expected_len}, got {}",
        info.len
    );
    let mut actual = info.predicate_indices;
    actual.sort();
    let expected: Vec<_> = (0..s.indexed_fields)
        .map(|i| format!("field-{i}"))
        .collect();
    ensure!(actual == expected, "predicate configuration differs");
    let has_hnsw = info
        .non_linear_indices
        .iter()
        .any(|index| matches!(index.index, Some(non_linear_index::Index::Hnsw(_))));
    ensure!(has_hnsw == s.hnsw, "HNSW configuration differs");
    // Check memberships created by the sentinel write before measured Set calls.
    for field in expected {
        let result = client
            .get_pred(
                GetPred {
                    store: s.store.clone(),
                    schema: None,
                    condition: Some(PredicateCondition {
                        kind: Some(predicate_condition::Kind::Value(Predicate {
                            kind: Some(predicate::Kind::Equals(Equals {
                                key: field,
                                value: Some(string_value("seed".into())),
                            })),
                        })),
                    }),
                },
                None,
            )
            .await?;
        ensure!(
            result.entries == vec![s.seed()],
            "seed predicate membership changed"
        );
    }
    Ok(())
}

fn compact_ranges(values: &[usize]) -> String {
    let mut ranges = Vec::new();
    let mut start = values[0];
    let mut end = start;
    for &value in &values[1..] {
        if value == end + 1 {
            end = value;
            continue;
        }
        ranges.push(if start == end {
            start.to_string()
        } else {
            format!("{start}-{end}")
        });
        start = value;
        end = value;
    }
    ranges.push(if start == end {
        start.to_string()
    } else {
        format!("{start}-{end}")
    });
    ranges.join(", ")
}

async fn diagnose_insert_length(client: &DbClient, s: &Spec, actual_len: usize) -> Result<String> {
    // ghz assigns RequestNumber globally from zero. Probe one key from every request in
    // bounded messages so a failed run tells us whether complete Set calls disappeared.
    let mut present = HashSet::new();
    for first_request in (0..s.expected_requests).step_by(64) {
        let last_request = (first_request + 64).min(s.expected_requests);
        let keys = (first_request..last_request)
            .map(|request| s.insert_key(request, 0))
            .collect();
        let result = client
            .get_key(
                GetKey {
                    store: s.store.clone(),
                    keys,
                    schema: None,
                },
                None,
            )
            .await?;
        for entry in result.entries {
            let request = entry
                .key
                .context("diagnostic GetKey entry missing key")?
                .key
                .first()
                .copied()
                .context("diagnostic GetKey entry has an empty key")?;
            present.insert(request as usize);
        }
    }

    let missing: Vec<_> = (0..s.expected_requests)
        .filter(|request| !present.contains(request))
        .collect();
    let expected_len = 1 + s.expected_requests * s.batch;
    let missing_entries = expected_len.saturating_sub(actual_len);
    if missing.is_empty() {
        return Ok(format!(
            "every request number 0-{} has its first row; the {missing_entries}-entry gap is within one or more partial batches",
            s.expected_requests - 1
        ));
    }

    let whole_batch_entries = missing.len() * s.batch;
    let qualification = if whole_batch_entries == missing_entries {
        "complete"
    } else {
        "at least partially missing"
    };
    Ok(format!(
        "request-number batches {} are {qualification} ({} absent first rows account for {whole_batch_entries} of {missing_entries} missing entries)",
        compact_ranges(&missing),
        missing.len()
    ))
}

async fn verify_hnsw_indexed_entries(client: &DbClient, s: &Spec) -> Result<()> {
    // An unindexed predicate scans the primary store to obtain the expected IDs. For an
    // accept list of at most 64 IDs, HNSW scans its own node map directly. Comparing the
    // returned keys therefore detects primary entries missing from the HNSW node map.
    for request in 0..s.expected_requests {
        for first_row in (0..s.batch).step_by(HNSW_VERIFICATION_SHARD_SIZE) {
            let last_row = (first_row + HNSW_VERIFICATION_SHARD_SIZE).min(s.batch);
            let shard = first_row / HNSW_VERIFICATION_SHARD_SIZE;
            let result = client
                .get_sim_n(
                    GetSimN {
                        store: s.store.clone(),
                        search_input: Some(s.insert_key(request, first_row)),
                        closest_n: (last_row - first_row) as u64,
                        algorithm: Algorithm::Hnsw as i32,
                        condition: Some(PredicateCondition {
                            kind: Some(predicate_condition::Kind::Value(Predicate {
                                kind: Some(predicate::Kind::Equals(Equals {
                                    key: HNSW_VERIFICATION_FIELD.into(),
                                    value: Some(string_value(format!("{request}:{shard}"))),
                                })),
                            })),
                        }),
                        schema: None,
                        ef_search: None,
                    },
                    None,
                )
                .await?;

            let mut seen = vec![false; last_row - first_row];
            for entry in result.entries {
                let key = entry.key.context("HNSW verification result missing key")?;
                let row = key
                    .key
                    .get(1)
                    .copied()
                    .context("HNSW verification result has no row coordinate")?;
                ensure!(
                    row >= (first_row + 1) as f32 && row <= last_row as f32,
                    "HNSW returned a row outside request {request}, shard {shard}"
                );
                let row = row as usize - 1;
                ensure!(
                    key == s.insert_key(request, row),
                    "HNSW returned an unexpected key for request {request}, row {row}"
                );
                ensure!(
                    !std::mem::replace(&mut seen[row - first_row], true),
                    "HNSW returned a duplicate key for request {request}, row {row}"
                );
            }

            let missing: Vec<_> = (first_row..last_row)
                .filter(|row| !seen[row - first_row])
                .collect();
            ensure!(
                missing.is_empty(),
                "HNSW did not return request {request}, shard {shard}, rows {}",
                compact_ranges(&missing)
            );
        }
    }
    Ok(())
}

async fn prepare(client: &DbClient, s: &Spec) -> Result<()> {
    // Never drop an existing user store; the runner starts a fresh private server.
    client
        .create_store(
            CreateStore {
                store: s.store.clone(),
                dimension: s.dimension as u32,
                create_predicates: (0..s.indexed_fields)
                    .map(|i| format!("field-{i}"))
                    .collect(),
                non_linear_indices: if s.hnsw {
                    vec![NonLinearIndex {
                        index: Some(non_linear_index::Index::Hnsw(HnswConfig {
                            ef_construction: Some(100),
                            ..Default::default()
                        })),
                    }]
                } else {
                    vec![]
                },
                error_if_exists: true,
                schema: None,
            },
            None,
        )
        .await?;
    insert(client, s, vec![s.seed()]).await?;
    if s.workload == Workload::Update {
        for request in 0..s.pool_requests {
            insert(
                client,
                s,
                (0..s.batch).map(|row| s.entry(request, row)).collect(),
            )
            .await?;
        }
    }
    let expected_len = match s.workload {
        Workload::Update => 1 + s.pool_requests * s.batch,
        Workload::Insert => 1,
    };
    check_catalog(client, s, expected_len).await
}

async fn verify(client: &DbClient, s: &Spec) -> Result<()> {
    if s.workload == Workload::Insert {
        ensure!(s.expected_requests > 0, "missing expected request count");
        let expected_len = 1 + s.expected_requests * s.batch;
        let info = client.get_store(s.store.clone(), None).await?;
        if info.len != expected_len as u64 {
            let diagnostic = diagnose_insert_length(client, s, info.len as usize).await?;
            bail!(
                "store length: expected {expected_len}, got {}; {diagnostic}",
                info.len
            );
        }
        check_catalog(client, s, expected_len).await?;
        if s.hnsw {
            verify_hnsw_indexed_entries(client, s).await?;
        }
        return Ok(());
    }

    check_catalog(client, s, 1 + s.pool_requests * s.batch).await?;
    // Inspect the first and last requests across each batch.
    for request in [0, s.pool_requests - 1] {
        for row in [0, s.batch / 2, s.batch - 1] {
            let expected = s.entry(request, row);
            let result = client
                .get_key(
                    GetKey {
                        store: s.store.clone(),
                        keys: vec![expected.key.clone().unwrap()],
                        schema: None,
                    },
                    None,
                )
                .await?;
            ensure!(
                result.entries == vec![expected],
                "sampled entry missing or metadata differs"
            );
        }
    }
    Ok(())
}

async fn cleanup(client: &DbClient, s: &Spec) -> Result<()> {
    let deleted = client
        .drop_store(
            DropStore {
                store: s.store.clone(),
                error_if_not_exists: true,
                schema: None,
            },
            None,
        )
        .await?;
    ensure!(deleted.deleted_count == 1, "store was not deleted");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    ensure!(
        args.len() == 3,
        "usage: setup_set generate|prepare|verify|cleanup SPEC.json"
    );
    let s: Spec = serde_json::from_slice(&std::fs::read(Path::new(&args[2]))?)?;
    ensure!(
        s.batch > 0 && s.batch < (1 << 24) && s.dimension >= 2 && s.dimension <= 4096,
        "invalid shape"
    );
    ensure!(
        s.indexed_fields <= 4 && s.cardinality > 0,
        "invalid metadata configuration"
    );
    ensure!(
        s.pool_requests > 0 && s.pool_requests < (1 << 24),
        "invalid request count"
    );
    if args[1] == "generate" {
        return write_payload(&s);
    }
    let client = DbClient::new(std::env::var("AHNLICH_DB_ADDR")?).await?;
    match args[1].as_str() {
        "prepare" => prepare(&client, &s).await?,
        "verify" => verify(&client, &s).await?,
        "cleanup" => cleanup(&client, &s).await?,
        _ => bail!("unknown action"),
    }
    println!("{} passed for {}", args[1], s.store);
    Ok(())
}
