use ahnlich_db::engine::store::{ParallelismConfig, StoreHandler};
use ahnlich_types::{
    algorithm::algorithms::Algorithm,
    keyval::{StoreKey, StoreName, StoreValue},
    metadata::{MetadataValue, metadata_value},
    predicates::{
        Equals, In, NotEquals, Predicate, PredicateCondition, predicate, predicate_condition,
    },
    schema::Schema,
};
use criterion::{BenchmarkId, Criterion, black_box};
use std::{
    collections::{HashMap, HashSet},
    num::NonZeroUsize,
    sync::{Arc, atomic::AtomicBool},
};

pub fn value(s: &str) -> MetadataValue {
    MetadataValue {
        value: Some(metadata_value::Value::RawString(s.into())),
    }
}

pub fn condition() -> PredicateCondition {
    PredicateCondition {
        kind: Some(predicate_condition::Kind::Value(Predicate {
            kind: Some(predicate::Kind::Equals(Equals {
                key: "category".into(),
                value: Some(value("match")),
            })),
        })),
    }
}

#[allow(dead_code)] // This module is also imported by correctness tests and one control per bench.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ControlPath {
    Current,
    Historical,
}

#[allow(dead_code)]
impl ControlPath {
    fn name(self) -> &'static str {
        match self {
            Self::Current => "predicate_linear_search_current",
            Self::Historical => "predicate_linear_search_historical",
        }
    }

    fn search(
        self,
        fixture: &Fixture,
        predicate: PredicateCondition,
    ) -> Vec<(
        ahnlich_similarity::EmbeddingKey,
        Arc<StoreValue>,
        ahnlich_types::similarity::Similarity,
    )> {
        match self {
            Self::Current => fixture.search(false, Algorithm::EuclideanDistance, Some(predicate)),
            Self::Historical => {
                fixture.search_indexed_matches(Algorithm::EuclideanDistance, predicate)
            }
        }
    }
}

#[allow(dead_code)]
pub fn benchmark_comparison(c: &mut Criterion, control: ControlPath) {
    for count in [10_000, 100_000] {
        for dimension in [128, 1536] {
            for permille in [0, 10, 100, 500] {
                for indexed in [true, false] {
                    if !indexed && control == ControlPath::Historical {
                        continue;
                    }
                    let fixture = Fixture::new(count, dimension, count * permille / 1000, indexed);
                    let predicate = condition();
                    let expected = control.search(&fixture, predicate.clone());
                    assert_eq!(
                        expected,
                        fixture
                            .search(true, Algorithm::EuclideanDistance, Some(predicate.clone()),),
                    );

                    let mut group = c.benchmark_group(format!(
                        "{}/n{count}_d{dimension}_permille{permille}_indexed{indexed}",
                        control.name(),
                    ));
                    group.bench_function(BenchmarkId::new("control", "top10"), |b| {
                        b.iter(|| black_box(control.search(&fixture, predicate.clone())))
                    });
                    group.bench_function(BenchmarkId::new("candidate", "top10"), |b| {
                        b.iter(|| {
                            black_box(fixture.search(
                                true,
                                Algorithm::EuclideanDistance,
                                Some(predicate.clone()),
                            ))
                        })
                    });
                    group.finish();
                }
            }
        }
    }

    for shape in PlannerShape::ALL {
        for dimension in [128, 1536] {
            for candidate_permille in [10, 100, 500] {
                let case = PlannerFixture::new(100_000, dimension, candidate_permille, shape);
                let predicate = case.condition.clone();
                let expected = control.search(&case.fixture, predicate.clone());
                assert_eq!(
                    expected,
                    case.fixture.search(
                        true,
                        Algorithm::EuclideanDistance,
                        Some(predicate.clone()),
                    ),
                );

                let mut group = c.benchmark_group(format!(
                    "{}/planner/{}/d{dimension}_candidate_permille{candidate_permille}",
                    control.name(),
                    shape.name(),
                ));
                group.bench_function(BenchmarkId::new("control", "top10"), |b| {
                    b.iter(|| black_box(control.search(&case.fixture, predicate.clone())))
                });
                group.bench_function(BenchmarkId::new("candidate", "top10"), |b| {
                    b.iter(|| {
                        black_box(case.fixture.search(
                            true,
                            Algorithm::EuclideanDistance,
                            Some(predicate.clone()),
                        ))
                    })
                });
                group.finish();
            }
        }
    }
}

pub struct Fixture {
    pub handler: StoreHandler,
    pub name: StoreName,
    pub query: StoreKey,
}

#[derive(Clone, Copy, Debug)]
pub enum PlannerShape {
    Equals,
    In,
    AndIndexed,
    AndPartial,
    OrIndexed,
    OrPartial,
    Negative,
}

impl PlannerShape {
    pub const ALL: [Self; 7] = [
        Self::Equals,
        Self::In,
        Self::AndIndexed,
        Self::AndPartial,
        Self::OrIndexed,
        Self::OrPartial,
        Self::Negative,
    ];

    #[allow(dead_code)]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Equals => "equals",
            Self::In => "in",
            Self::AndIndexed => "and_indexed",
            Self::AndPartial => "and_partial",
            Self::OrIndexed => "or_indexed",
            Self::OrPartial => "or_partial",
            Self::Negative => "negative",
        }
    }

    fn condition(self) -> PredicateCondition {
        let equals = |key: &str, expected: &str| PredicateCondition {
            kind: Some(predicate_condition::Kind::Value(Predicate {
                kind: Some(predicate::Kind::Equals(Equals {
                    key: key.into(),
                    value: Some(value(expected)),
                })),
            })),
        };
        let negative = || PredicateCondition {
            kind: Some(predicate_condition::Kind::Value(Predicate {
                kind: Some(predicate::Kind::NotEquals(NotEquals {
                    key: "status".into(),
                    value: Some(value("blocked")),
                })),
            })),
        };

        match self {
            Self::Equals => equals("category", "a"),
            Self::In => PredicateCondition {
                kind: Some(predicate_condition::Kind::Value(Predicate {
                    kind: Some(predicate::Kind::In(In {
                        key: "category".into(),
                        values: vec![value("a"), value("b")],
                    })),
                })),
            },
            Self::AndIndexed => equals("category", "a").and(equals("region", "x")),
            Self::AndPartial => equals("category", "a").and(negative()),
            Self::OrIndexed => equals("category", "a").or(equals("region", "x")),
            Self::OrPartial => equals("category", "a").or(negative()),
            Self::Negative => negative(),
        }
    }
}

pub struct PlannerFixture {
    pub fixture: Fixture,
    pub condition: PredicateCondition,
}

impl PlannerFixture {
    pub fn new(
        count: usize,
        dimension: usize,
        candidate_permille: usize,
        shape: PlannerShape,
    ) -> Self {
        let handler = StoreHandler::new(
            Arc::new(AtomicBool::new(false)),
            ParallelismConfig::from_cli(16, None, 10_000),
        );
        let name = StoreName {
            value: "predicate-linear-planner".into(),
        };
        handler
            .create_store(
                name.clone(),
                &Schema::default(),
                NonZeroUsize::new(dimension).unwrap(),
                vec!["category".into(), "region".into(), "status".into()],
                HashSet::new(),
                true,
            )
            .unwrap();

        let candidate_count = count * candidate_permille / 1000;
        let first_half = candidate_count / 2;
        let second_half_end = candidate_count;
        let intersection = candidate_count / 2;
        let entries = (0..count)
            .map(|id| {
                let mut key: Vec<_> = (0..dimension)
                    .map(|d| ((id * 17 + d * 31) % 101) as f32 / 101.0)
                    .collect();
                key[0] = id as f32;

                let (category, region, status) = match shape {
                    PlannerShape::Equals => (
                        if id < candidate_count { "a" } else { "other" },
                        "other",
                        "blocked",
                    ),
                    PlannerShape::In => (
                        if id < first_half {
                            "a"
                        } else if id < second_half_end {
                            "b"
                        } else {
                            "other"
                        },
                        "other",
                        "blocked",
                    ),
                    PlannerShape::AndIndexed => (
                        if id < candidate_count { "a" } else { "other" },
                        if id < intersection
                            || (id >= candidate_count
                                && id < candidate_count + (candidate_count - intersection))
                        {
                            "x"
                        } else {
                            "other"
                        },
                        "blocked",
                    ),
                    PlannerShape::AndPartial => (
                        if id < candidate_count { "a" } else { "other" },
                        "other",
                        if id % 2 == 0 { "allowed" } else { "blocked" },
                    ),
                    PlannerShape::OrIndexed => (
                        if id < first_half { "a" } else { "other" },
                        if id >= first_half && id < second_half_end {
                            "x"
                        } else {
                            "other"
                        },
                        "blocked",
                    ),
                    PlannerShape::OrPartial => (
                        if id < first_half { "a" } else { "other" },
                        "other",
                        if id >= first_half && id < second_half_end {
                            "allowed"
                        } else {
                            "blocked"
                        },
                    ),
                    PlannerShape::Negative => (
                        "other",
                        "other",
                        if id < candidate_count {
                            "allowed"
                        } else {
                            "blocked"
                        },
                    ),
                };
                let metadata = HashMap::from([
                    ("category".into(), value(category)),
                    ("region".into(), value(region)),
                    ("status".into(), value(status)),
                ]);
                (StoreKey { key }, StoreValue { value: metadata })
            })
            .collect();
        handler
            .set_in_store(&name, &Schema::default(), entries)
            .unwrap();

        Self {
            fixture: Fixture {
                handler,
                name,
                query: StoreKey {
                    key: vec![0.25; dimension],
                },
            },
            condition: shape.condition(),
        }
    }
}

impl Fixture {
    pub fn new(count: usize, dimension: usize, matches: usize, indexed: bool) -> Self {
        let handler = StoreHandler::new(
            Arc::new(AtomicBool::new(false)),
            ParallelismConfig::from_cli(16, None, 10_000),
        );
        let name = StoreName {
            value: "predicate-linear".into(),
        };
        handler
            .create_store(
                name.clone(),
                &Schema::default(),
                NonZeroUsize::new(dimension).unwrap(),
                if indexed {
                    vec!["category".into()]
                } else {
                    vec![]
                },
                HashSet::new(),
                true,
            )
            .unwrap();
        let entries = (0..count)
            .map(|id| {
                let mut key: Vec<_> = (0..dimension)
                    .map(|d| ((id * 17 + d * 31) % 101) as f32 / 101.0)
                    .collect();
                key[0] = id as f32;
                let metadata = if id < matches {
                    HashMap::from([("category".into(), value("match"))])
                } else if id % 2 == 0 {
                    HashMap::from([("category".into(), value("other"))])
                } else {
                    HashMap::new()
                };
                (StoreKey { key }, StoreValue { value: metadata })
            })
            .collect();
        handler
            .set_in_store(&name, &Schema::default(), entries)
            .unwrap();
        Self {
            handler,
            name,
            query: StoreKey {
                key: vec![0.25; dimension],
            },
        }
    }

    pub fn search(
        &self,
        candidate: bool,
        metric: Algorithm,
        cond: Option<PredicateCondition>,
    ) -> Vec<(
        ahnlich_similarity::EmbeddingKey,
        Arc<StoreValue>,
        ahnlich_types::similarity::Similarity,
    )> {
        let search = if candidate {
            StoreHandler::get_sim_in_store_with_bounded_index_filtering
        } else {
            StoreHandler::get_sim_in_store
        };
        search(
            &self.handler,
            &self.name,
            &Schema::default(),
            self.query.clone(),
            NonZeroUsize::new(10).unwrap(),
            metric,
            cond,
        )
        .unwrap()
    }

    #[allow(dead_code)]
    pub fn search_indexed_matches(
        &self,
        metric: Algorithm,
        cond: PredicateCondition,
    ) -> Vec<(
        ahnlich_similarity::EmbeddingKey,
        Arc<StoreValue>,
        ahnlich_types::similarity::Similarity,
    )> {
        self.handler
            .get_sim_in_store_with_indexed_matches(
                &self.name,
                &Schema::default(),
                self.query.clone(),
                NonZeroUsize::new(10).unwrap(),
                metric,
                cond,
            )
            .unwrap()
    }
}
