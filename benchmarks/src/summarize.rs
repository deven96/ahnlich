//! Collapses the per-run ghz JSON reports in a directory into SUMMARY.md.
//!
//! Each configuration is reported as the median of its repeats, with the spread, since
//! single runs vary by ~20% at high concurrency.
//!
//! Usage: `summarize <results-dir>`

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Subset of the ghz report. Durations are nanoseconds.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Report {
    count: u64,
    rps: f64,
    #[serde(default)]
    latency_distribution: Vec<LatencyBucket>,
    #[serde(default)]
    status_code_distribution: BTreeMap<String, u64>,
}

#[derive(Deserialize)]
struct LatencyBucket {
    percentage: u32,
    latency: f64,
}

impl Report {
    fn percentile(&self, percentage: u32) -> Option<f64> {
        self.latency_distribution
            .iter()
            .find(|bucket| bucket.percentage == percentage)
            .map(|bucket| bucket.latency)
    }

    fn failed(&self) -> u64 {
        self.status_code_distribution
            .iter()
            .filter(|(code, _)| code.as_str() != "OK")
            .map(|(_, count)| count)
            .sum()
    }
}

struct Group {
    label: String,
    concurrency: u32,
    reports: Vec<Report>,
    /// Server CPU microseconds per request, one entry per repeat. `None` when the
    /// sidecar `.cpu` file was missing or unreadable.
    server_cpu: Vec<Option<f64>>,
}

impl Group {
    /// Median of each metric independently, not of one representative run.
    fn median_of(&self, metric: impl Fn(&Report) -> Option<f64>) -> Option<f64> {
        let mut values: Vec<f64> = self.reports.iter().filter_map(&metric).collect();
        median(&mut values)
    }

    fn rps_range(&self) -> (f64, f64) {
        let rps: Vec<f64> = self.reports.iter().map(|report| report.rps).collect();
        let min = rps.iter().copied().fold(f64::INFINITY, f64::min);
        let max = rps.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        (min, max)
    }

    fn failed(&self) -> u64 {
        self.reports.iter().map(Report::failed).sum()
    }

    /// Renders `- (n/total)` unless every repeat contributed a reading.
    fn server_cpu_cell(&self) -> String {
        let samples: Vec<f64> = self.server_cpu.iter().flatten().copied().collect();
        if samples.len() != self.reports.len() {
            return format!("- ({}/{})", samples.len(), self.reports.len());
        }
        median(&mut samples.clone()).map_or("-".to_owned(), |cpu| format!("{cpu:.0}"))
    }
}

fn median(values: &mut [f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    Some(if values.len().is_multiple_of(2) {
        (values[middle - 1] + values[middle]) / 2.0
    } else {
        values[middle]
    })
}

fn main() -> Result<()> {
    let results_dir = match std::env::args().nth(1) {
        Some(dir) => PathBuf::from(dir),
        None => bail!("usage: summarize <results-dir>"),
    };

    let groups = collect_groups(&results_dir)?;
    if groups.is_empty() {
        bail!("no ghz result files found in {}", results_dir.display());
    }

    let summary = render(&groups)?;
    let summary_path = results_dir.join("SUMMARY.md");
    std::fs::write(&summary_path, &summary)
        .with_context(|| format!("could not write {}", summary_path.display()))?;

    println!("Wrote {}", summary_path.display());
    print!("{summary}");
    Ok(())
}

/// Reports and their server-CPU sidecars, keyed by (label, concurrency).
type Grouped = BTreeMap<(String, u32), (Vec<Report>, Vec<Option<f64>>)>;

fn collect_groups(dir: &Path) -> Result<Vec<Group>> {
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("could not read results dir {}", dir.display()))?;

    // BTreeMap for stable ordering regardless of readdir order.
    let mut grouped: Grouped = BTreeMap::new();
    for entry in entries {
        let path = entry?.path();
        if path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }

        let Some((label, concurrency)) = parse_run_name(&path) else {
            continue;
        };

        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("could not read {}", path.display()))?;
        let report: Report = serde_json::from_str(&contents)
            .with_context(|| format!("could not parse ghz report {}", path.display()))?;

        let cpu_path = path.with_extension("cpu");
        let server_cpu = match std::fs::read_to_string(&cpu_path) {
            Ok(raw) => match raw.trim().parse::<f64>() {
                Ok(micros) => Some(micros),
                Err(err) => {
                    eprintln!("warning: {} is not a number: {err}", cpu_path.display());
                    None
                }
            },
            Err(_) => None,
        };

        let entry = grouped.entry((label, concurrency)).or_default();
        entry.0.push(report);
        entry.1.push(server_cpu);
    }

    Ok(grouped
        .into_iter()
        .map(|((label, concurrency), (reports, server_cpu))| Group {
            label,
            concurrency,
            reports,
            server_cpu,
        })
        .collect())
}

/// `linear_c50_r2.json` -> ("linear", 50). Repeats are summarised together.
fn parse_run_name(path: &Path) -> Option<(String, u32)> {
    let stem = path.file_stem()?.to_str()?;
    let stem = match stem.rsplit_once("_r") {
        Some((head, repeat)) if repeat.parse::<u32>().is_ok() => head,
        _ => stem,
    };
    let (label, concurrency) = stem.rsplit_once("_c")?;
    Some((label.to_owned(), concurrency.parse().ok()?))
}

fn render(groups: &[Group]) -> Result<String> {
    let repeats = groups
        .iter()
        .map(|group| group.reports.len())
        .max()
        .unwrap_or(0);

    let mut out = String::new();
    writeln!(out, "# Baseline Benchmark Results\n")?;
    writeln!(out, "Median of {repeats} run(s) per configuration.\n")?;
    writeln!(
        out,
        "`server us/req` is CPU measured from the server process; RPS also includes client \
         cost, which at concurrency 1 is roughly half the round trip.\n"
    )?;
    writeln!(
        out,
        "| Run | Concurrency | Runs | Requests | Failed | RPS (median) | RPS range | server us/req | p50 (ms) | p95 (ms) | p99 (ms) |"
    )?;
    writeln!(out, "|---|---|---|---|---|---|---|---|---|---|---|")?;

    for group in groups {
        let (min, max) = group.rps_range();
        writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {:.0}–{:.0} | {} | {} | {} | {} |",
            group.label,
            group.concurrency,
            group.reports.len(),
            group.reports.first().map_or(0, |report| report.count),
            group.failed(),
            group
                .median_of(|report| Some(report.rps))
                .map_or("-".to_owned(), |rps| format!("{rps:.1}")),
            min,
            max,
            group.server_cpu_cell(),
            millis(group.median_of(|report| report.percentile(50))),
            millis(group.median_of(|report| report.percentile(95))),
            millis(group.median_of(|report| report.percentile(99))),
        )?;
    }

    let failed: u64 = groups.iter().map(Group::failed).sum();
    if failed > 0 {
        writeln!(
            out,
            "\n> {failed} request(s) did not return OK. Treat these numbers as invalid \
             until that is explained."
        )?;
    }

    if let Some(noisiest) = groups
        .iter()
        .filter(|group| group.reports.len() > 1)
        .max_by(|a, b| spread(a).total_cmp(&spread(b)))
        && spread(noisiest) > 0.1
    {
        writeln!(
            out,
            "\n> Widest spread: {} at concurrency {} varied {:.0}% between repeats. \
             Improvements smaller than that are not measurable here.",
            noisiest.label,
            noisiest.concurrency,
            spread(noisiest) * 100.0
        )?;
    }

    Ok(out)
}

fn spread(group: &Group) -> f64 {
    let (min, max) = group.rps_range();
    if min <= 0.0 { 0.0 } else { (max - min) / min }
}

fn millis(nanos: Option<f64>) -> String {
    match nanos {
        Some(nanos) => format!("{:.2}", nanos / 1e6),
        None => "-".to_owned(),
    }
}
