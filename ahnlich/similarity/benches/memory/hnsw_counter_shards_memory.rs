#[path = "../support/hnsw_collector_sharing.rs"]
mod support;

use ahnlich_similarity::{
    EmbeddingKey, LinearAlgorithm,
    hnsw::{HNSWConfig, index::HNSW},
};
use std::fmt::Write as _;
use std::hint::black_box;
use std::num::NonZeroUsize;
use std::path::Path;
use std::process::Command;
use support::{MEMORY_SCENARIOS, Scenario, vectors};

#[derive(Clone, Copy)]
enum Variant {
    Control,
    One,
    Two,
}

impl Variant {
    const ALL: [Self; 3] = [Self::Control, Self::One, Self::Two];
    fn label(self) -> &'static str {
        match self {
            Self::Control => "control",
            Self::One => "candidate_1",
            Self::Two => "candidate_2",
        }
    }
    fn shards(self) -> Option<NonZeroUsize> {
        match self {
            Self::Control => None,
            Self::One => NonZeroUsize::new(1),
            Self::Two => NonZeroUsize::new(2),
        }
    }
    fn from_label(label: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|variant| variant.label() == label)
            .unwrap_or_else(|| panic!("unknown variant: {label}"))
    }
}

fn populated(variant: Variant, scenario: Scenario, data: &[EmbeddingKey]) -> HNSW<LinearAlgorithm> {
    let config = HNSWConfig {
        ef_construction: scenario.ef_construction,
        maximum_connections: scenario.m,
        maximum_connections_zero: scenario.m_zero,
        extend_candidates: false,
        keep_pruned_connections: false,
    };
    let index = match variant.shards() {
        None => {
            HNSW::new_with_config_and_shared_collector(config, LinearAlgorithm::CosineSimilarity)
        }
        Some(shards) => HNSW::new_with_config_and_shared_collector_and_counter_shards(
            config,
            LinearAlgorithm::CosineSimilarity,
            shards,
        ),
    };
    index.insert(data).expect("fixture insertion failed");
    index
}
use tikv_jemalloc_ctl::{epoch, stats};

const SAMPLES: usize = 3;

#[derive(Clone, Copy)]
struct MemoryStats {
    allocated: usize,
    active: usize,
    resident: usize,
}

impl MemoryStats {
    fn read() -> Self {
        epoch::mib().unwrap().advance().unwrap();
        Self {
            allocated: stats::allocated::mib().unwrap().read().unwrap(),
            active: stats::active::mib().unwrap().read().unwrap(),
            resident: stats::resident::mib().unwrap().read().unwrap(),
        }
    }

    fn delta(self, baseline: Self) -> MemoryUsage {
        MemoryUsage {
            allocated: self.allocated as isize - baseline.allocated as isize,
            active: self.active as isize - baseline.active as isize,
            resident: self.resident as isize - baseline.resident as isize,
        }
    }
}

#[derive(Clone, Copy)]
struct MemoryUsage {
    allocated: isize,
    active: isize,
    resident: isize,
}

#[derive(Clone, Copy)]
struct MemoryResult {
    populated: MemoryUsage,
    after_delete: MemoryUsage,
    after_drop: MemoryUsage,
    nodes: usize,
    collections: usize,
}

impl MemoryResult {
    fn values(self) -> [isize; 11] {
        [
            self.populated.allocated,
            self.populated.active,
            self.populated.resident,
            self.after_delete.allocated,
            self.after_delete.active,
            self.after_delete.resident,
            self.after_drop.allocated,
            self.after_drop.active,
            self.after_drop.resident,
            self.nodes as isize,
            self.collections as isize,
        ]
    }

    fn from_values(values: [isize; 11]) -> Self {
        Self {
            populated: MemoryUsage {
                allocated: values[0],
                active: values[1],
                resident: values[2],
            },
            after_delete: MemoryUsage {
                allocated: values[3],
                active: values[4],
                resident: values[5],
            },
            after_drop: MemoryUsage {
                allocated: values[6],
                active: values[7],
                resident: values[8],
            },
            nodes: values[9].try_into().unwrap(),
            collections: values[10].try_into().unwrap(),
        }
    }

    fn encode(self) -> String {
        self.values().map(|value| value.to_string()).join("\t")
    }

    fn decode(encoded: &str) -> Self {
        let values = encoded
            .split_whitespace()
            .map(|value| value.parse::<isize>().unwrap())
            .collect::<Vec<_>>();
        Self::from_values(
            values
                .try_into()
                .expect("expected eleven measurement values"),
        )
    }
}

fn scenario(label: &str) -> Scenario {
    MEMORY_SCENARIOS
        .into_iter()
        .find(|scenario| scenario.label == label)
        .unwrap_or_else(|| panic!("unknown memory scenario: {label}"))
}

fn measure_one(variant: Variant, scenario: Scenario) -> MemoryResult {
    // Keep fixture vectors alive in every phase and exclude them from the baseline delta.
    let data = vectors(0, scenario.entries, scenario.dimension);
    let baseline = MemoryStats::read();

    let store = populated(variant, scenario, &data);
    let (nodes, collections) = store.counter_shards_bench_counts();
    assert_eq!(nodes, data.len());
    black_box(&store);
    let populated = MemoryStats::read().delta(baseline);

    assert_eq!(store.delete(&data).unwrap(), data.len());
    assert_eq!(store.counter_shards_bench_counts().0, 0);
    black_box(&store);
    let after_delete = MemoryStats::read().delta(baseline);

    drop(store);
    let after_drop = MemoryStats::read().delta(baseline);

    MemoryResult {
        populated,
        after_delete,
        after_drop,
        nodes,
        collections,
    }
}

fn run_isolated(variant: Variant, scenario: Scenario) -> MemoryResult {
    let executable = std::env::current_exe().unwrap();
    let samples = (0..SAMPLES)
        .map(|_| {
            let output = Command::new(&executable)
                .args(["--measure-one", scenario.label, variant.label()])
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "isolated memory measurement failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            MemoryResult::decode(String::from_utf8_lossy(&output.stdout).trim())
        })
        .collect::<Vec<_>>();

    for sample in &samples {
        assert_eq!(
            (sample.nodes, sample.collections),
            (samples[0].nodes, samples[0].collections),
            "fixture structure changed across isolated samples"
        );
    }
    let mut medians = [0; 11];
    for (index, median) in medians.iter_mut().enumerate() {
        let mut values = samples
            .iter()
            .map(|sample| sample.values()[index])
            .collect::<Vec<_>>();
        values.sort_unstable();
        *median = values[values.len() / 2];
    }
    MemoryResult::from_values(medians)
}

fn mib(value: isize) -> String {
    format!("{:.3}", value as f64 / 1_048_576.0)
}

fn triple(a: isize, b: isize, c: isize) -> String {
    format!("{} / {} / {}", mib(a), mib(b), mib(c))
}

fn saving(control: isize, candidate: isize) -> String {
    if control <= 0 {
        return "n/a".to_owned();
    }
    format!(
        "{:.1}%",
        100.0 * (control - candidate) as f64 / control as f64
    )
}

fn expected_saving(collections: usize, default_shards: usize, candidate_shards: usize) -> String {
    // Match the pinned Papaya padding on the benchmark's supported comparison targets.
    if !cfg!(any(target_arch = "x86_64", target_arch = "aarch64")) {
        return "n/a".to_owned();
    }
    let bytes = collections as i128 * (default_shards as i128 - candidate_shards as i128) * 128;
    format!("{:.3}", bytes as f64 / 1_048_576.0)
}

fn render_report() -> String {
    let cpus = std::thread::available_parallelism().unwrap().get();
    let default_shards = cpus.next_power_of_two();
    let mut report = format!(
        "### similarity/hnsw_counter_shards_memory\n\nDetected CPUs: {cpus}; default counter shards: {default_shards}; architecture: {}.\n\nMedian of {SAMPLES} fresh processes per scenario/variant. All variants share one collector. Control uses default sharding; candidates use one/two shards only for per-node collections. Index-wide collections retain default sharding.\n\nValues are jemalloc deltas from a baseline retaining the input vectors. Active/resident columns are control / one shard / two shards. Resident is jemalloc resident, not process RSS. Fixtures use cosine distance and serial insertion.\n\n",
        std::env::consts::ARCH
    );
    let mut structure = String::from(
        "| Scenario | Nodes | Per-node collections | Expected counter saving MiB: one shard | Expected counter saving MiB: two shards |\n|---|---:|---:|---:|---:|\n",
    );
    let mut measurements = String::from(
        "| Scenario | Phase | Control allocated MiB | One shard allocated MiB | Saving | Two shards allocated MiB | Saving | Active MiB | Resident MiB |\n|---|---|---:|---:|---:|---:|---:|---:|---:|\n",
    );
    for scenario in MEMORY_SCENARIOS {
        let results = Variant::ALL.map(|variant| run_isolated(variant, scenario));
        for candidate in &results[1..] {
            assert_eq!(
                (candidate.nodes, candidate.collections),
                (results[0].nodes, results[0].collections),
                "control/candidate fixture structure differs"
            );
        }
        writeln!(
            structure,
            "| `{}` | {} | {} | {} | {} |",
            scenario.label,
            results[0].nodes,
            results[0].collections,
            expected_saving(results[0].collections, default_shards, 1),
            expected_saving(results[0].collections, default_shards, 2)
        )
        .unwrap();
        for (phase, values, compare_saving) in [
            ("populated", results.map(|r| r.populated), true),
            (
                "after deleting all nodes",
                results.map(|r| r.after_delete),
                true,
            ),
            ("after dropping index", results.map(|r| r.after_drop), false),
        ] {
            writeln!(
                measurements,
                "| `{}` | {} | {} | {} | {} | {} | {} | {} | {} |",
                scenario.label,
                phase,
                mib(values[0].allocated),
                mib(values[1].allocated),
                if compare_saving {
                    saving(values[0].allocated, values[1].allocated)
                } else {
                    "n/a".to_owned()
                },
                mib(values[2].allocated),
                if compare_saving {
                    saving(values[0].allocated, values[2].allocated)
                } else {
                    "n/a".to_owned()
                },
                triple(values[0].active, values[1].active, values[2].active),
                triple(values[0].resident, values[1].resident, values[2].resident)
            )
            .unwrap();
        }
    }
    report.push_str(&structure);
    report.push_str("\nExpected savings cover live per-node counters only, using 128-byte padding on x86_64/ARM64. Retired collections, allocator size classes, and reclamation timing can change the measured delta; this prediction is not whole-index accounting. A negative expectation on a small-CPU machine means the candidate uses more shards than the default.\n\n");
    report.push_str(&measurements);
    report
}

fn write_report(path: &Path) {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).expect("failed to create memory report directory");
    }
    std::fs::write(path, render_report()).expect("failed to write memory report");
}

fn main() {
    let args = std::env::args()
        .skip(1)
        .filter(|arg| arg != "--bench")
        .collect::<Vec<_>>();
    match args.as_slice() {
        [] => print!("{}", render_report()),
        [flag, path] if flag == "--output" => write_report(Path::new(path)),
        [flag, scenario_label, variant_label] if flag == "--measure-one" => {
            let result = measure_one(Variant::from_label(variant_label), scenario(scenario_label));
            println!("{}", result.encode());
        }
        _ => {
            panic!(
                "usage: hnsw_counter_shards_memory [--output PATH | --measure-one SCENARIO VARIANT]"
            )
        }
    }
}
