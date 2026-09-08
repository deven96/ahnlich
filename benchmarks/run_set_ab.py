#!/usr/bin/env python3
"""Opt-in ghz Set A/B runner. Uses prebuilt binaries; never builds or checks out code."""
import argparse
import hashlib
import itertools
import json
import math
import os
from pathlib import Path
import platform
import resource
import shutil
import signal
import socket
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
MARKER = "benchmark: existing-predicate-index candidate enabled"


def positive(text):
    value = int(text)
    if value <= 0:
        raise argparse.ArgumentTypeError("must be positive")
    return value


def sha256(path):
    with open(path, "rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def command(args, log, env=None):
    with log.open("a") as out:
        out.write(json.dumps([str(arg) for arg in args]) + "\n")
        out.flush()
        subprocess.run(args, env=env, stdout=out, stderr=subprocess.STDOUT, check=True)


def port_open(port):
    try:
        with socket.create_connection(("127.0.0.1", port), timeout=0.2):
            return True
    except OSError:
        return False


def server_cpu(pid):
    proc = Path(f"/proc/{pid}/stat")
    if proc.exists():
        fields = proc.read_text().rsplit(")", 1)[1].split()
        return (int(fields[11]) + int(fields[12])) / os.sysconf("SC_CLK_TCK")
    raw = subprocess.check_output(["ps", "-o", "time=", "-p", str(pid)], text=True).strip()
    days, raw = raw.split("-", 1) if "-" in raw else ("0", raw)
    parts = [float(part) for part in raw.split(":")]
    return int(days) * 86400 + sum(value * 60 ** i for i, value in enumerate(reversed(parts)))


def stop_server(server):
    if server.poll() is None:
        server.terminate()
        try:
            server.wait(timeout=10)
        except subprocess.TimeoutExpired:
            server.kill()
            server.wait()


def ghz(args, spec, output, concurrency, duration=None, total=None):
    cmd = [args.ghz, "--insecure", "--proto", str(ROOT / "protos/services/db_service.proto"),
           "--import-paths", str(ROOT / "protos"), "--call", "services.db_service.DBService/Set",
           "--data-file", spec["payload_file"], "--disable-template-data", "--count-errors",
           "--concurrency", str(concurrency), "--connections", str(min(args.connections, concurrency)),
           "--timeout", f"{args.request_timeout}s", "--format", "json", "--output", str(output)]
    if duration is not None:
        cmd += ["--duration", f"{duration}s", "--duration-stop", "wait"]
    else:
        cmd += ["--total", str(total)]
    cmd += [f"127.0.0.1:{args.port}"]
    command(cmd, output.with_suffix(".log"))
    report = json.loads(output.read_text())
    count = report.get("count", 0)
    codes = report.get("statusCodeDistribution", {})
    if count <= 0 or codes.get("OK") != count or any(v for k, v in codes.items() if k != "OK"):
        raise RuntimeError(f"incomplete/non-OK ghz run: {output}: count={count}, statuses={codes}")
    if report.get("errorDistribution"):
        raise RuntimeError(f"ghz errors: {output}")
    if total is not None and count != total:
        raise RuntimeError(f"expected exactly {total} requests, got {count}")
    if not math.isfinite(report["rps"]) or report["rps"] <= 0:
        raise RuntimeError(f"invalid throughput: {output}")
    return report


def helper(args, action, spec_file, log):
    env = dict(os.environ, AHNLICH_DB_ADDR=f"127.0.0.1:{args.port}")
    command([str(args.setup_bin), action, str(spec_file)], log, env)


def measure(args, variant, binary, spec, spec_file, warm, warm_file, concurrency, repeat):
    tag = f"{spec_file.stem}_c{concurrency}_r{repeat}_{variant}"
    folder = args.results / tag
    folder.mkdir()
    log = folder / "server.log"
    if port_open(args.port):
        raise RuntimeError(f"port {args.port} is occupied; refusing to use an existing server")
    server_args = [str(binary), "run", "--host", "127.0.0.1", "--port", str(args.port),
                   "--threadpool-size", str(args.threads),
                   "--parallel-batch-threshold", str(args.parallel_batch_threshold),
                   "--parallel-concurrency-threshold", str(args.threads),
                   "--size-calculation-interval", str(args.size_calculation_interval),
                   "--allocator-size", str(args.allocator_size), "--message-size", str(args.message_size)]
    write_json(folder / "server-command.json", server_args)
    with log.open("w") as out:
        server = subprocess.Popen(server_args, stdout=out, stderr=subprocess.STDOUT)
        try:
            for _ in range(120):
                if server.poll() is not None:
                    raise RuntimeError(f"server exited during startup: {log}")
                if port_open(args.port):
                    break
                time.sleep(0.25)
            else:
                raise RuntimeError(f"server startup timeout: {log}")
            enabled = MARKER in log.read_text()
            if enabled != (variant == "candidate"):
                raise RuntimeError(f"{variant} binary has wrong candidate feature; inspect {log}")
            helper(args, "prepare", warm_file, folder / "setup.log")
            ghz(args, warm, folder / "warmup.json", concurrency, duration=args.warmup_seconds)
            helper(args, "verify", warm_file, folder / "setup.log")
            helper(args, "prepare", spec_file, folder / "setup.log")

            cpu_before = server_cpu(server.pid)
            child_before = resource.getrusage(resource.RUSAGE_CHILDREN)
            started = time.monotonic()
            report = ghz(args, spec, folder / "measured.json", concurrency,
                         duration=args.seconds if spec["workload"] == "update" else None,
                         total=None if spec["workload"] == "update" else spec["total_requests"])
            wall = time.monotonic() - started
            child_after = resource.getrusage(resource.RUSAGE_CHILDREN)
            cpu_after = server_cpu(server.pid)
            if server.poll() is not None:
                raise RuntimeError(f"server exited during measurement: {log}")
            if spec["workload"] == "update" and report["count"] < spec["pool_requests"]:
                raise RuntimeError("update run did not cover the request pool; increase --seconds")
            helper(args, "verify", spec_file, folder / "verification.log")
            count = report["count"]
            inserted_per_request = {"insert": spec["batch"], "update": 0,
                                    "mixed": spec["batch"] - spec["batch"] // 2}[spec["workload"]]
            latency = {str(bucket["percentage"]): bucket["latency"] / 1e6
                       for bucket in report["latencyDistribution"]}
            for percentile in ("50", "95", "99"):
                if percentile not in latency:
                    raise RuntimeError(f"missing p{percentile} latency in {tag}")
            return {"scenario": spec_file.stem, "variant": variant, "repeat": repeat,
                    "concurrency": concurrency, "batch": spec["batch"], "count": count,
                    "rps": report["rps"], "entries_per_second": report["rps"] * spec["batch"],
                    "expected_inserted": inserted_per_request * count,
                    "expected_updated": (spec["batch"] - inserted_per_request) * count,
                    "server_us_per_request": (cpu_after - cpu_before) * 1e6 / count,
                    "ghz_cpu_seconds": child_after.ru_utime + child_after.ru_stime
                                       - child_before.ru_utime - child_before.ru_stime,
                    "wall_seconds": wall, "latency_ms": latency, "state_checks": "passed",
                    "short_run": wall < 10, "report": str(folder / "measured.json")}
        finally:
            stop_server(server)


def summary(args, records):
    lines = ["# Set A/B results", "", "Median of repeats. Positive RPS change favors candidate.", "",
             "| Scenario | c | Variant | Runs | RPS (min–max) | Entries/s | CPU µs/req | p50 ms | p95 ms | p99 ms | RPS change |",
             "|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|"]
    for scenario, concurrency in sorted({(r["scenario"], r["concurrency"]) for r in records}):
        rows = [r for r in records if (r["scenario"], r["concurrency"]) == (scenario, concurrency)]
        control = statistics.median(r["rps"] for r in rows if r["variant"] == "control")
        for variant in ("control", "candidate"):
            group = [r for r in rows if r["variant"] == variant]
            med = lambda key: statistics.median(r[key] for r in group)
            lat = [statistics.median(r["latency_ms"][p] for r in group) for p in ("50", "95", "99")]
            change = "—" if variant == "control" else f"{(med('rps') / control - 1) * 100:+.1f}%"
            lines.append(f"| {scenario} | {concurrency} | {variant} | {len(group)} | "
                         f"{med('rps'):.1f} ({min(r['rps'] for r in group):.1f}–{max(r['rps'] for r in group):.1f}) | "
                         f"{med('entries_per_second'):.0f} | {med('server_us_per_request'):.1f} | "
                         f"{lat[0]:.2f} | {lat[1]:.2f} | {lat[2]:.2f} | {change} |")
    lines += ["", "All included runs completed with only OK statuses and passed post-run state checks.",
              "CPU/request includes the ghz invocation window; fixture setup and verification are excluded.",
              "Counts labeled expected are derived from fixtures, not sums of individual RPC response bodies."]
    if any(r["short_run"] for r in records):
        lines += ["", "Some runs lasted under 10 seconds. Increase the request budget before drawing throughput conclusions."]
    (args.results / "SUMMARY.md").write_text("\n".join(lines) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--control-bin", type=Path, required=True)
    parser.add_argument("--candidate-bin", type=Path, required=True)
    parser.add_argument("--setup-bin", type=Path, default=ROOT / "benchmarks/target/release/setup_set")
    parser.add_argument("--control-ref", required=True, help="source revision used to build control")
    parser.add_argument("--candidate-ref", required=True, help="source revision/dirty-tree description used to build candidate")
    parser.add_argument("--ghz", default="ghz")
    parser.add_argument("--results", type=Path, default=ROOT / "benchmarks/results" / time.strftime("set_ab_%Y%m%d_%H%M%S"))
    parser.add_argument("--workloads", nargs="+", choices=["update", "insert", "mixed"], default=["update"])
    parser.add_argument("--batches", nargs="+", type=positive, default=[100, 1000])
    parser.add_argument("--concurrency", nargs="+", type=positive, default=[1, 4, 16])
    parser.add_argument("--cases", nargs="+", default=["0:100", "1:1", "1:100", "4:100"], help="indexed-fields:cardinality pairs")
    parser.add_argument("--repeats", type=positive, default=3)
    parser.add_argument("--seconds", type=positive, default=15, help="update duration per measured run")
    parser.add_argument("--warmup-seconds", type=positive, default=3)
    parser.add_argument("--total-requests", type=positive, default=1000, help="exact request count for insert/mixed")
    parser.add_argument("--max-fixture-entries", type=positive, default=200000)
    parser.add_argument("--pool-requests", type=positive, default=32)
    parser.add_argument("--dimension", type=positive, default=128)
    parser.add_argument("--threads", type=positive, default=16)
    parser.add_argument("--connections", type=positive, default=8)
    parser.add_argument("--port", type=positive, default=1397)
    parser.add_argument("--parallel-batch-threshold", type=positive, default=150000)
    parser.add_argument("--size-calculation-interval", type=positive, default=60000, help="milliseconds")
    parser.add_argument("--allocator-size", type=positive, default=10073741824)
    parser.add_argument("--message-size", type=positive, default=10048576)
    parser.add_argument("--request-timeout", type=positive, default=60)
    args = parser.parse_args()
    cases = []
    for raw in args.cases:
        try:
            fields, cardinality = map(int, raw.split(":"))
            assert 0 <= fields <= 4 and cardinality > 0
        except (ValueError, AssertionError):
            parser.error(f"invalid case: {raw}")
        cases.append((fields, cardinality))
    if not 2 <= args.dimension <= 4096 or args.port > 65535:
        parser.error("dimension must be 2..4096 and port must be <=65535")
    if "mixed" in args.workloads and min(args.batches) < 2:
        parser.error("mixed requires batch >=2")
    for values in (args.workloads, args.batches, args.concurrency, cases):
        if len(set(values)) != len(values):
            parser.error("duplicate scenarios are not supported")
    for workload, batch in itertools.product(args.workloads, args.batches):
        count = args.pool_requests if workload == "update" else args.total_requests
        if count >= 1 << 24 or batch >= 1 << 24 or batch * count > args.max_fixture_entries:
            parser.error("fixture exceeds identity/memory budget; reduce requests/batch or explicitly raise --max-fixture-entries")
    for name in ("control_bin", "candidate_bin", "setup_bin"):
        path = getattr(args, name).resolve()
        if not path.is_file() or not os.access(path, os.X_OK):
            parser.error(f"executable missing: {path}")
        setattr(args, name, path)
    if sha256(args.control_bin) == sha256(args.candidate_bin):
        parser.error("control and candidate binaries are identical")
    if shutil.which(args.ghz) is None:
        parser.error("ghz is not installed")
    args.results = args.results.resolve()
    args.results.mkdir(parents=True, exist_ok=False)
    specs_dir = args.results / "fixtures"
    specs_dir.mkdir()
    config = {k: str(v) if isinstance(v, Path) else v for k, v in vars(args).items()}
    config["host"] = platform.platform()
    config["logical_cpus"] = os.cpu_count()
    config["ghz_version"] = subprocess.check_output([args.ghz, "--version"], text=True, stderr=subprocess.STDOUT).strip()
    config["binary_sha256"] = {name: sha256(getattr(args, name)) for name in ("control_bin", "candidate_bin", "setup_bin")}
    write_json(args.results / "RUN.json", config)

    scenarios = []
    for workload, batch, (fields, cardinality) in itertools.product(args.workloads, args.batches, cases):
        name = f"{workload}_b{batch}_i{fields}_v{cardinality}"
        spec_file = specs_dir / f"{name}.json"
        spec = {"store": "set_bench", "workload": workload, "batch": batch, "dimension": args.dimension,
                "indexed_fields": fields, "cardinality": cardinality, "pool_requests": args.pool_requests,
                "total_requests": args.total_requests, "payload_file": str(specs_dir / f"{name}.payload.json")}
        warm = dict(spec, store="set_warmup", workload="update", pool_requests=4,
                    payload_file=str(specs_dir / f"{name}.warmup.payload.json"))
        warm_file = specs_dir / f"{name}.warmup.json"
        write_json(spec_file, spec)
        write_json(warm_file, warm)
        helper(args, "generate", spec_file, args.results / "generation.log")
        helper(args, "generate", warm_file, args.results / "generation.log")
        config.setdefault("payload_sha256", {})[name] = sha256(Path(spec["payload_file"]))
        scenarios.append((spec, spec_file, warm, warm_file))
    write_json(args.results / "RUN.json", config)
    records = []
    for repeat in range(1, args.repeats + 1):
        order = [("control", args.control_bin), ("candidate", args.candidate_bin)]
        if repeat % 2 == 0:
            order.reverse()
        for spec, spec_file, warm, warm_file in scenarios:
            for concurrency in args.concurrency:
                for variant, binary in order:
                    print(f"{spec_file.stem} c={concurrency} repeat={repeat} {variant}", flush=True)
                    record = measure(args, variant, binary, spec, spec_file, warm, warm_file, concurrency, repeat)
                    records.append(record)
                    with (args.results / "records.jsonl").open("a") as out:
                        out.write(json.dumps(record) + "\n")
    summary(args, records)
    print(f"Results: {args.results / 'SUMMARY.md'}")


if __name__ == "__main__":
    def interrupt(_signum, _frame):
        raise KeyboardInterrupt
    signal.signal(signal.SIGTERM, interrupt)
    main()
