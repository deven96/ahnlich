#[path = "../support/papaya_collector_sharing.rs"]
mod support;

use std::fmt::Write as _;
use std::hint::black_box;
use std::path::Path;
use std::process::Command;
use support::{MEMORY_SCENARIOS, Scenario, SharingScope, populated_store};
use tikv_jemalloc_ctl::{epoch, stats};

const SAMPLES: usize = 3;

#[derive(Clone, Copy)]
enum Variant {
    Control,
    Candidate,
}

impl Variant {
    const ALL: [Self; 2] = [Self::Control, Self::Candidate];

    const fn label(self) -> &'static str {
        match self {
            Self::Control => "control",
            Self::Candidate => "candidate",
        }
    }

    const fn scope(self) -> SharingScope {
        match self {
            Self::Control => SharingScope::Independent,
            Self::Candidate => SharingScope::PerIndex,
        }
    }

    fn from_label(label: &str) -> Self {
        match label {
            "control" => Self::Control,
            "candidate" => Self::Candidate,
            _ => panic!("unknown variant: {label}"),
        }
    }
}

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
}

impl MemoryResult {
    fn values(self) -> [isize; 9] {
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
        ]
    }

    fn from_values(values: [isize; 9]) -> Self {
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
        Self::from_values(values.try_into().expect("expected nine memory values"))
    }
}

fn scenario(label: &str) -> Scenario {
    MEMORY_SCENARIOS
        .into_iter()
        .find(|scenario| scenario.label == label)
        .unwrap_or_else(|| panic!("unknown memory scenario: {label}"))
}

fn measure_one(scope: SharingScope, scenario: Scenario) -> MemoryResult {
    let baseline = MemoryStats::read();

    let store = populated_store(scope, scenario);
    black_box(&store);
    let populated = MemoryStats::read().delta(baseline);

    store.clear_store_keys();
    black_box(&store);
    let after_delete = MemoryStats::read().delta(baseline);

    drop(store);
    let after_drop = MemoryStats::read().delta(baseline);

    MemoryResult {
        populated,
        after_delete,
        after_drop,
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

    let mut medians = [0; 9];
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

fn mib_pair(control: isize, candidate: isize) -> String {
    format!(
        "{:.3} / {:.3}",
        control as f64 / 1_048_576.0,
        candidate as f64 / 1_048_576.0
    )
}

fn saving(control: isize, candidate: isize) -> String {
    if control <= 0 {
        return "n/a".to_string();
    }
    format!(
        "{:.1}%",
        (control - candidate) as f64 / control as f64 * 100.0
    )
}

fn render_report() -> String {
    let mut report = String::new();
    writeln!(report, "### db/papaya_collector_memory").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "_Median of {SAMPLES} isolated processes per case. Values are control / candidate._"
    )
    .unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| Scenario | Phase | Collectors | Allocated MiB | Saving | Active MiB | Resident MiB |"
    )
    .unwrap();
    writeln!(report, "|---|---|---:|---:|---:|---:|---:|").unwrap();

    for scenario in MEMORY_SCENARIOS {
        let results = Variant::ALL.map(|variant| run_isolated(variant, scenario));
        let control_collectors = SharingScope::Independent
            .expected_collectors(scenario.predicates, scenario.expected_buckets());
        let candidate_collectors = SharingScope::PerIndex
            .expected_collectors(scenario.predicates, scenario.expected_buckets());

        for (phase, control, candidate, collectors, compare_saving) in [
            (
                "populated",
                results[0].populated,
                results[1].populated,
                format!("{control_collectors} / {candidate_collectors}"),
                true,
            ),
            (
                "after deleting store keys",
                results[0].after_delete,
                results[1].after_delete,
                format!("{control_collectors} / {candidate_collectors}"),
                true,
            ),
            (
                "after dropping store",
                results[0].after_drop,
                results[1].after_drop,
                "0 / 0".to_string(),
                false,
            ),
        ] {
            writeln!(
                report,
                "| `{}` | {} | {} | {} | {} | {} | {} |",
                scenario.label,
                phase,
                collectors,
                mib_pair(control.allocated, candidate.allocated),
                if compare_saving {
                    saving(control.allocated, candidate.allocated)
                } else {
                    "n/a".to_string()
                },
                mib_pair(control.active, candidate.active),
                mib_pair(control.resident, candidate.resident),
            )
            .unwrap();
        }
    }

    report
}

fn write_report(path: &Path) {
    std::fs::write(path, render_report()).unwrap();
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
            let result = measure_one(
                Variant::from_label(variant_label).scope(),
                scenario(scenario_label),
            );
            println!("{}", result.encode());
        }
        _ => panic!(
            "usage: papaya_collector_memory [--output PATH | --measure-one SCENARIO VARIANT]"
        ),
    }
}
