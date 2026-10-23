#![allow(dead_code)]

use ahnlich_similarity::hnsw::HNSWConfig;
use ahnlich_similarity::hnsw::index::experiments::ControlHNSW as HNSW;
use ahnlich_similarity::{EmbeddingKey, LinearAlgorithm};
use rand::{Rng, SeedableRng, rngs::StdRng};

// Match DB's allocator for both the performance and memory experiments.
#[global_allocator]
static ALLOCATOR: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[derive(Clone, Copy)]
pub enum Variant {
    Control,
    Candidate,
}

impl Variant {
    pub const ALL: [Self; 2] = [Self::Control, Self::Candidate];
    pub const fn label(self) -> &'static str {
        match self {
            Self::Control => "control",
            Self::Candidate => "candidate",
        }
    }
    pub fn from_label(label: &str) -> Self {
        match label {
            "control" => Self::Control,
            "candidate" => Self::Candidate,
            _ => panic!("unknown variant: {label}"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Scenario {
    pub label: &'static str,
    pub entries: usize,
    pub dimension: usize,
    pub m: usize,
    pub m_zero: usize,
    pub ef_construction: usize,
    pub ef_search: usize,
}

pub const PERFORMANCE_SCENARIOS: [Scenario; 2] = [
    Scenario {
        label: "1k_128d_defaults",
        entries: 1_000,
        dimension: 128,
        m: 16,
        m_zero: 32,
        ef_construction: 200,
        ef_search: 100,
    },
    Scenario {
        label: "1k_768d_zvec_1m",
        entries: 1_000,
        dimension: 768,
        m: 15,
        m_zero: 30,
        ef_construction: 500,
        ef_search: 180,
    },
];
pub const MEMORY_SCENARIOS: [Scenario; 3] = [
    PERFORMANCE_SCENARIOS[0],
    PERFORMANCE_SCENARIOS[1],
    Scenario {
        label: "10k_768d_zvec_1m",
        entries: 10_000,
        ..PERFORMANCE_SCENARIOS[1]
    },
];

pub fn vectors(start: usize, count: usize, dimension: usize) -> Vec<EmbeddingKey> {
    (start..start + count)
        .map(|id| {
            let mut rng = StdRng::seed_from_u64(id as u64 + 424);
            EmbeddingKey::new((0..dimension).map(|_| rng.gen_range(-1.0..1.0)).collect())
        })
        .collect()
}

pub fn index(variant: Variant, scenario: Scenario) -> HNSW<LinearAlgorithm> {
    let config = HNSWConfig {
        ef_construction: scenario.ef_construction,
        maximum_connections: scenario.m,
        maximum_connections_zero: scenario.m_zero,
        extend_candidates: false,
        keep_pruned_connections: false,
    };
    match variant {
        Variant::Control => {
            HNSW::new_control_with_config(config, LinearAlgorithm::CosineSimilarity)
        }
        Variant::Candidate => HNSW::new_control_with_config_and_shared_collector(
            config,
            LinearAlgorithm::CosineSimilarity,
        ),
    }
}

pub fn populated(
    variant: Variant,
    scenario: Scenario,
    data: &[EmbeddingKey],
) -> HNSW<LinearAlgorithm> {
    let index = index(variant, scenario);
    index.insert(data).expect("fixture insertion failed");
    assert_eq!(index.collector_sharing_bench_counts().0, data.len());
    index
}
