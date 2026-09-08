//! Deterministic Set fixtures for run_set_ab.py. No benchmark timing is done here.
use ahnlich_client_rs::db::DbClient;
use ahnlich_types::db::query::{CreateStore, GetKey, GetPred, Set};
use ahnlich_types::keyval::{DbStoreEntry, StoreKey, StoreValue};
use ahnlich_types::metadata::{MetadataValue, metadata_value::Value};
use ahnlich_types::predicates::{
    Equals, Predicate, PredicateCondition, predicate, predicate_condition,
};
use anyhow::{Context, Result, bail, ensure};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::path::Path;

#[derive(Deserialize)]
struct Spec {
    store: String,
    workload: String,
    batch: usize,
    dimension: usize,
    indexed_fields: usize,
    cardinality: usize,
    pool_requests: usize,
    total_requests: usize,
    payload_file: String,
}

impl Spec {
    fn requests(&self) -> usize {
        if self.workload == "update" {
            self.pool_requests
        } else {
            self.total_requests
        }
    }

    fn preload_rows(&self) -> usize {
        match self.workload.as_str() {
            "update" => self.batch,
            "mixed" => self.batch / 2,
            _ => 0,
        }
    }

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
}

fn string_value(value: String) -> MetadataValue {
    MetadataValue {
        value: Some(Value::RawString(value)),
    }
}

fn write_payload(s: &Spec) -> Result<()> {
    let mut out = BufWriter::new(std::fs::File::create(&s.payload_file)?);
    out.write_all(b"[")?;
    for request in 0..s.requests() {
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
                non_linear_indices: vec![],
                error_if_exists: true,
                schema: None,
            },
            None,
        )
        .await?;
    insert(client, s, vec![s.seed()]).await?;
    if s.preload_rows() > 0 {
        for request in 0..s.requests() {
            insert(
                client,
                s,
                (0..s.preload_rows())
                    .map(|row| s.entry(request, row))
                    .collect(),
            )
            .await?;
        }
    }
    check_catalog(client, s, 1 + s.requests() * s.preload_rows()).await
}

async fn verify(client: &DbClient, s: &Spec) -> Result<()> {
    check_catalog(client, s, 1 + s.requests() * s.batch).await?;
    // Inspect first and last requests, including both sides of the mixed split.
    for request in [0, s.requests() - 1] {
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

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    ensure!(
        args.len() == 3,
        "usage: setup_set generate|prepare|verify SPEC.json"
    );
    let s: Spec = serde_json::from_slice(&std::fs::read(Path::new(&args[2]))?)?;
    ensure!(
        matches!(s.workload.as_str(), "insert" | "update" | "mixed"),
        "invalid workload"
    );
    ensure!(
        s.batch > 0 && s.batch < (1 << 24) && s.dimension >= 2 && s.dimension <= 4096,
        "invalid shape"
    );
    ensure!(
        s.indexed_fields <= 4 && s.cardinality > 0,
        "invalid metadata configuration"
    );
    ensure!(
        s.requests() > 0 && s.requests() < (1 << 24),
        "invalid request count"
    );
    if args[1] == "generate" {
        return write_payload(&s);
    }
    let client = DbClient::new(std::env::var("AHNLICH_DB_ADDR")?).await?;
    match args[1].as_str() {
        "prepare" => prepare(&client, &s).await?,
        "verify" => verify(&client, &s).await?,
        _ => bail!("unknown action"),
    }
    println!("{} passed for {}", args[1], s.store);
    Ok(())
}
