#![cfg(feature = "bench-experiments")]
#[path = "../benches/support/predicate_linear_search.rs"]
mod support;
use ahnlich_types::{
    algorithm::algorithms::Algorithm,
    predicates::{NotEquals, Predicate, PredicateCondition, predicate, predicate_condition},
};
use support::{Fixture, PlannerFixture, PlannerShape, condition, value};

#[test]
fn recursive_planner_shapes_match_scan_and_historical_results() {
    for shape in PlannerShape::ALL {
        for candidate_permille in [10, 100, 500] {
            let case = PlannerFixture::new(1_000, 16, candidate_permille, shape);
            let predicate = case.condition.clone();
            let scan =
                case.fixture
                    .search(false, Algorithm::EuclideanDistance, Some(predicate.clone()));
            assert_eq!(
                scan,
                case.fixture
                    .search_indexed_matches(Algorithm::EuclideanDistance, predicate.clone(),),
                "historical mismatch for {shape:?} at {candidate_permille} permille",
            );
            assert_eq!(
                scan,
                case.fixture
                    .search(true, Algorithm::EuclideanDistance, Some(predicate),),
                "bounded mismatch for {shape:?} at {candidate_permille} permille",
            );
        }
    }
}

#[test]
fn indexed_equality_matches_scan_and_fallbacks() {
    for indexed in [false, true] {
        for matches in [0, 1, 10, 100, 500, 1000] {
            let fixture = Fixture::new(1000, 16, matches, indexed);
            let negative = PredicateCondition {
                kind: Some(predicate_condition::Kind::Value(Predicate {
                    kind: Some(predicate::Kind::NotEquals(NotEquals {
                        key: "category".into(),
                        value: Some(value("match")),
                    })),
                })),
            };
            for metric in [
                Algorithm::EuclideanDistance,
                Algorithm::CosineSimilarity,
                Algorithm::DotProductSimilarity,
            ] {
                for cond in [Some(condition()), Some(negative.clone()), None] {
                    assert_eq!(
                        fixture.search(false, metric, cond.clone()),
                        fixture.search(true, metric, cond)
                    );
                }
            }
        }
    }
}

#[test]
fn parallel_candidates_and_completed_updates_match_scan() {
    use ahnlich_types::{
        keyval::{StoreKey, StoreValue},
        schema::Schema,
    };
    use std::collections::HashMap;
    // 600 candidates exceeds the 1536d parallel threshold (500).
    let fixture = Fixture::new(6_000, 1536, 600, true);
    let metric = Algorithm::EuclideanDistance;
    let before = fixture.search(false, metric, Some(condition()));
    assert_eq!(before, fixture.search(true, metric, Some(condition())));
    let key = StoreKey {
        key: before[0].0.as_slice().to_vec(),
    };
    for metadata in [
        HashMap::new(),
        HashMap::from([("category".into(), value("match"))]),
    ] {
        fixture
            .handler
            .set_in_store(
                &fixture.name,
                &Schema::default(),
                vec![(key.clone(), StoreValue { value: metadata })],
            )
            .unwrap();
        assert_eq!(
            fixture.search(false, metric, Some(condition())),
            fixture.search(true, metric, Some(condition()))
        );
    }
}

#[test]
fn recursive_and_unsupported_conditions_preserve_scan_semantics() {
    use ahnlich_types::predicates::{Equals, In, NotIn};
    let fixture = Fixture::new(1000, 16, 10, true);
    let in_condition = PredicateCondition {
        kind: Some(predicate_condition::Kind::Value(Predicate {
            kind: Some(predicate::Kind::In(In {
                key: "category".into(),
                values: vec![value("match"), value("other")],
            })),
        })),
    };
    let negative = PredicateCondition {
        kind: Some(predicate_condition::Kind::Value(Predicate {
            kind: Some(predicate::Kind::NotIn(NotIn {
                key: "category".into(),
                values: vec![value("match")],
            })),
        })),
    };
    let conditions = [
        PredicateCondition { kind: None },
        PredicateCondition {
            kind: Some(predicate_condition::Kind::Value(Predicate {
                kind: Some(predicate::Kind::Equals(Equals {
                    key: "category".into(),
                    value: None,
                })),
            })),
        },
        negative.clone(),
        condition().and(in_condition.clone()),
        condition().or(in_condition),
        condition().and(negative.clone()),
        condition().or(negative),
    ];
    for cond in conditions {
        assert_eq!(
            fixture.search(false, Algorithm::EuclideanDistance, Some(cond.clone())),
            fixture.search(true, Algorithm::EuclideanDistance, Some(cond))
        );
    }
}
