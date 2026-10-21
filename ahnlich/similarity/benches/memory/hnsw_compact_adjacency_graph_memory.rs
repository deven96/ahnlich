//! Structural-memory comparison for the whole compact-adjacency HNSW candidate.
use std::{fmt::Write as _, path::Path, process::Command};

use ahnlich_similarity::{
    LinearAlgorithm,
    hnsw::{
        HNSWConfig,
        index::{HNSW, experiments::CompactAdjacencyHNSW},
    },
};
use tikv_jemalloc_ctl::{epoch, stats};

#[path = "../support/hnsw_collector_sharing.rs"]
mod support;

const SAMPLES: usize = 3;
const CONFIG: HNSWConfig = HNSWConfig {
    ef_construction: 200,
    maximum_connections: 16,
    maximum_connections_zero: 32,
    extend_candidates: false,
    keep_pruned_connections: false,
};

fn memory() -> [i64; 3] {
    epoch::mib().unwrap().advance().unwrap();
    [
        stats::allocated::read().unwrap() as i64,
        stats::active::read().unwrap() as i64,
        stats::resident::read().unwrap() as i64,
    ]
}

fn measure(variant: &str, entries: usize) -> [i64; 5] {
    // Accounting-only: production keeps per-thread caches enabled.
    unsafe { tikv_jemalloc_ctl::raw::write(b"thread.tcache.enabled\0", false).unwrap() };
    let data = support::vectors(0, entries, 768);
    let _ = memory();
    let baseline = memory();
    let (counts, populated) = match variant {
        "control" => {
            let index = HNSW::new_with_config_and_shared_collector_and_counter_shards(
                CONFIG,
                LinearAlgorithm::CosineSimilarity,
                std::num::NonZeroUsize::new(1).unwrap(),
            );
            index.insert(&data).unwrap();
            (index.compact_adjacency_bench_counts(), memory())
        }
        "candidate" => {
            let index = CompactAdjacencyHNSW::new(CONFIG, LinearAlgorithm::CosineSimilarity);
            index.insert(&data).unwrap();
            let counts = index.edge_counts();
            (counts, memory())
        }
        _ => panic!("unknown variant: {variant}"),
    };
    // The index is dropped at the end of the match arm. The source vectors remain
    // in the baseline so unchanged embedding payload is excluded from the delta.
    let dropped = memory();
    assert!(
        (dropped[0] - baseline[0]).abs() <= 65_536,
        "post-drop allocation drift exceeds 64 KiB; discard this sample"
    );
    [
        counts.0 as i64,
        counts.1 as i64,
        populated[0] - baseline[0],
        populated[1] - baseline[1],
        populated[2] - baseline[2],
    ]
}

fn isolated(variant: &str, entries: usize) -> [i64; 5] {
    let mut samples = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--measure-one", variant, &entries.to_string()])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "measurement failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let values = String::from_utf8(output.stdout)
            .unwrap()
            .split_whitespace()
            .map(|value| value.parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(values.len(), 5);
        samples.push(values);
    }
    std::array::from_fn(|column| {
        let mut values = samples
            .iter()
            .map(|sample| sample[column])
            .collect::<Vec<_>>();
        values.sort_unstable();
        values[SAMPLES / 2]
    })
}

fn main() {
    let args = std::env::args()
        .skip(1)
        .filter(|arg| arg != "--bench")
        .collect::<Vec<_>>();
    if args.first().is_some_and(|arg| arg == "--measure-one") {
        assert_eq!(args.len(), 3);
        let values = measure(&args[1], args[2].parse().unwrap());
        println!(
            "{}",
            values
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        );
        return;
    }
    let mut output = None;
    let mut entries = 10_000;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--output" => {
                index += 1;
                output = Some(args[index].clone());
            }
            "--entries" => {
                index += 1;
                entries = args[index].parse::<usize>().unwrap();
            }
            other => panic!("unknown argument: {other}"),
        }
        index += 1;
    }

    let control = isolated("control", entries);
    let candidate = isolated("candidate", entries);
    assert_eq!(control[..2], candidate[..2]);
    let mut report = format!(
        "### similarity/hnsw_compact_adjacency_graph_memory\n\nWhole HNSW structural-memory candidate, `{entries}` deterministic 768-dimensional vectors, cosine, serial insertion, M=16/M0=32/efConstruction=200, shared collectors, and one per-node counter shard. Median of {SAMPLES} fresh processes. Both variants build the same topology; vectors are generated before the baseline and remain alive through the measurement, so unchanged float32 payload is excluded.\n\n"
    );
    report.push_str("The candidate replaces only each per-layer outgoing Papaya `HashSet<NodeId>` with an immutable `ArcSwap<Vec<NodeId>>`; node outer layer maps, backlinks, graph layer sets, IDs, and construction heuristic remain. Child thread cache is disabled for accounting only. MiB values are allocated / active / jemalloc resident; resident is not process RSS.\n\n");
    writeln!(report, "| Nodes | Layer lists | Outgoing edges | Control MiB | Candidate MiB | Allocated saving MiB | Saving/list bytes | Saving/edge bytes |\n|---:|---:|---:|---:|---:|---:|---:|---:|\n| {} | {} | {} | {:.3} / {:.3} / {:.3} | {:.3} / {:.3} / {:.3} | {:.3} | {:.1} | {:.1} |",
        entries, control[0], control[1],
        control[2] as f64 / 1_048_576.0, control[3] as f64 / 1_048_576.0, control[4] as f64 / 1_048_576.0,
        candidate[2] as f64 / 1_048_576.0, candidate[3] as f64 / 1_048_576.0, candidate[4] as f64 / 1_048_576.0,
        (control[2] - candidate[2]) as f64 / 1_048_576.0,
        (control[2] - candidate[2]) as f64 / control[0] as f64,
        (control[2] - candidate[2]) as f64 / control[1] as f64,
    ).unwrap();
    report.push_str("\nThis compares steady, post-build structural memory. It excludes vector payload and does not measure concurrent peak memory, persistence, deletion, or whole-DB overhead. Do not extrapolate directly to Cohere 10M.\n");
    if let Some(output) = output {
        let path = Path::new(&output);
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, &report).unwrap();
    }
    print!("{report}");
}
