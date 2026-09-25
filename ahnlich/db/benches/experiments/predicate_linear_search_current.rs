//! Current scan path versus bounded predicate-index candidates.
#[path = "../support/predicate_linear_search.rs"]
mod support;

use criterion::{Criterion, criterion_group, criterion_main};
use std::time::Duration;

fn compare(c: &mut Criterion) {
    support::benchmark_comparison(c, support::ControlPath::Current);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10).warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(3));
    targets = compare
}
criterion_main!(benches);
