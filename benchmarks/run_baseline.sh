#!/usr/bin/env bash
#
# Baseline QPS benchmark for ahnlich-db.
#
# Starts a server, loads SIFT into an unindexed and an HNSW store, drives GetSimN
# through ghz at several concurrency levels, writes a summary.
#
# Settings below are overridable:
#   TOTAL_REQUESTS=50000 CONCURRENCY_LEVELS="1 8 32" ./run_baseline.sh
# Use the locally installed SIFT1M corpus:
#   STORE_SIZE=100000 ./run_baseline.sh --sift-1m
#
set -euo pipefail

SIFT_DATASET="sift_10k"

usage() {
    cat <<EOF
Usage: $0 [--sift-1m]

  --sift-1m  Load the dataset from ahnlich/similarity/sift_1m instead of SIFT10K.
EOF
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --sift-1m) SIFT_DATASET="sift_1m" ;;
        -h|--help) usage; exit 0 ;;
        *) echo "error: unknown argument: $1" >&2; usage >&2; exit 1 ;;
    esac
    shift
done

HOST="${HOST:-127.0.0.1}"
PORT="${PORT:-1369}"
CONCURRENCY_LEVELS="${CONCURRENCY_LEVELS:-1 10 50 100}"
TOTAL_REQUESTS="${TOTAL_REQUESTS:-10000}"
CLOSEST_N="${CLOSEST_N:-10}"
REQUEST_TIMEOUT="${REQUEST_TIMEOUT:-60s}"

# Capped at the concurrency level; ghz rejects more connections than workers.
CONNECTIONS="${CONNECTIONS:-8}"

WARMUP_REQUESTS="${WARMUP_REQUESTS:-500}"

# The HNSW index is built with this; the linear store is queried with the matching
# algorithm.
DISTANCE_METRIC="${DISTANCE_METRIC:-euclidean}"

# Number of runs per configuration. Results are the median of these.
REPEATS="${REPEATS:-3}"

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd)"
AHNLICH_DIR="$REPO_DIR/ahnlich"
PROTO_DIR="$REPO_DIR/protos"

RESULTS_DIR="${RESULTS_DIR:-$SCRIPT_DIR/results/baseline_$(date +%Y%m%d_%H%M%S)}"
PAYLOAD_DIR="$RESULTS_DIR/payloads"
SERVER_LOG="$RESULTS_DIR/server.log"

log() { printf '\n==> %s\n' "$*"; }

require() {
    command -v "$1" >/dev/null 2>&1 || {
        echo "error: '$1' is required but not installed${2:+ ($2)}" >&2
        exit 1
    }
}

require cargo "https://rustup.rs"
require ghz "brew install ghz"

[ -d "$PROTO_DIR" ] || { echo "error: proto dir not found at $PROTO_DIR" >&2; exit 1; }

mkdir -p "$RESULTS_DIR" "$PAYLOAD_DIR"

# Resolve the output directory from cargo rather than assuming ./target.
release_dir_for() {
    local manifest="$1" target_dir
    target_dir="$(cargo metadata --no-deps --format-version 1 --manifest-path "$manifest" \
        | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')"
    [ -n "$target_dir" ] || { echo "error: could not read target dir for $manifest" >&2; exit 1; }
    echo "$target_dir/release"
}

log "Building ahnlich-db"
cargo build --release --manifest-path "$AHNLICH_DIR/Cargo.toml" --bin ahnlich-db
SERVER_BIN_DIR="$(release_dir_for "$AHNLICH_DIR/Cargo.toml")"

log "Building harness"
cargo build --release --manifest-path "$SCRIPT_DIR/Cargo.toml" --bins
HARNESS_BIN_DIR="$(release_dir_for "$SCRIPT_DIR/Cargo.toml")"

[ -x "$SERVER_BIN_DIR/ahnlich-db" ] \
    || { echo "error: $SERVER_BIN_DIR/ahnlich-db missing after build" >&2; exit 1; }
for bin in setup_sift summarize; do
    [ -x "$HARNESS_BIN_DIR/$bin" ] \
        || { echo "error: $HARNESS_BIN_DIR/$bin missing after build" >&2; exit 1; }
done

# Refuse to run if the port is taken.
if (exec 3<>"/dev/tcp/$HOST/$PORT") 2>/dev/null; then
    echo "error: something is already listening on $HOST:$PORT (set PORT= to move)" >&2
    exit 1
fi

SERVER_PID=""

cleanup() {
    if [ -n "$SERVER_PID" ] && kill -0 "$SERVER_PID" 2>/dev/null; then
        log "Stopping server (pid $SERVER_PID)"
        kill "$SERVER_PID" 2>/dev/null || true
        # Bounded wait, then SIGKILL.
        for _ in $(seq 1 20); do
            kill -0 "$SERVER_PID" 2>/dev/null || break
            sleep 0.5
        done
        kill -9 "$SERVER_PID" 2>/dev/null || true
        wait "$SERVER_PID" 2>/dev/null || true
    fi
}
trap cleanup EXIT
trap 'cleanup; exit 130' INT TERM

# Pin the flags that affect performance so a changed default in utils/src/cli.rs does
# not show up as a benchmark result.
SERVER_ARGS=(
    run
    --host "$HOST"
    --port "$PORT"
    --threadpool-size "${THREADPOOL_SIZE:-16}"
    --size-calculation-interval "${SIZE_CALCULATION_INTERVAL:-60}"
)

{
    echo "commit: $(git -C "$REPO_DIR" rev-parse --short HEAD 2>/dev/null || echo unknown)"
    if [ -n "$(git -C "$REPO_DIR" status --porcelain 2>/dev/null)" ]; then
        echo "tree: dirty"
    fi
    echo "host: $(uname -srm)"
    echo "ghz: $(ghz --version 2>&1 >/dev/null)"
    echo "server: ahnlich-db ${SERVER_ARGS[*]}"
    echo "requests: $TOTAL_REQUESTS x $REPEATS repeats, warmup $WARMUP_REQUESTS"
    echo "concurrency: $CONCURRENCY_LEVELS, connections: $CONNECTIONS"
    echo "metric: $DISTANCE_METRIC, closest_n: $CLOSEST_N, ef_construction: ${EF_CONSTRUCTION:-100}"
    echo "dataset: $SIFT_DATASET"
    echo "store_size: ${STORE_SIZE:-full dataset}"
} > "$RESULTS_DIR/RUN.txt"

log "Starting ahnlich-db on $HOST:$PORT"
"$SERVER_BIN_DIR/ahnlich-db" "${SERVER_ARGS[@]}" >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!

# Wait for the port. Uses /dev/tcp rather than nc, which differs between BSD and GNU.
for attempt in $(seq 1 60); do
    if ! kill -0 "$SERVER_PID" 2>/dev/null; then
        echo "error: server exited during startup" >&2
        cat "$SERVER_LOG" >&2
        exit 1
    fi
    if (exec 3<>"/dev/tcp/$HOST/$PORT") 2>/dev/null; then
        echo "Server ready after $attempt attempt(s)"
        break
    fi
    if [ "$attempt" -eq 60 ]; then
        echo "error: server did not open $HOST:$PORT within 60s" >&2
        cat "$SERVER_LOG" >&2
        exit 1
    fi
    sleep 1
done

log "Loading SIFT dataset and generating ghz payloads"
AHNLICH_DB_ADDR="$HOST:$PORT" \
PAYLOAD_DIR="$PAYLOAD_DIR" \
DISTANCE_METRIC="$DISTANCE_METRIC" \
CLOSEST_N="$CLOSEST_N" \
SIFT_DATASET="$SIFT_DATASET" \
    "$HARNESS_BIN_DIR/setup_sift"

# Cumulative CPU seconds of the server process. Reads /proc on Linux for 10ms
# resolution; `ps -o time=` there is whole seconds.
server_cpu_seconds() {
    if ! kill -0 "$SERVER_PID" 2>/dev/null; then
        echo "error: server process $SERVER_PID is gone" >&2
        return 1
    fi

    if [ -r "/proc/$SERVER_PID/stat" ]; then
        awk -v hz="$(getconf CLK_TCK)" '{ printf "%.3f", ($14 + $15) / hz }' \
            "/proc/$SERVER_PID/stat"
        return
    fi

    ps -o time= -p "$SERVER_PID" \
        | awk -F: '{n=NF; s=$n; m=(n>1?$(n-1):0); h=(n>2?$(n-2):0); printf "%.2f", h*3600+m*60+s}'
}

scenario_label() {
    case "$1" in
        linear) echo "linear" ;;
        linear_5k) echo "linear_5k" ;;
        linear_1k) echo "linear_1k" ;;
        linear_100) echo "linear_100" ;;
        hnsw) echo "hnsw" ;;
        hnsw_5k) echo "hnsw_5k" ;;
        hnsw_1k) echo "hnsw_1k" ;;
        hnsw_100) echo "hnsw_100" ;;
        ping) echo "ping" ;;
        *) echo "$1" ;;
    esac
}

run_ghz() {
    local label="$1" concurrency="$2" method="$3" payload="$4" repeat="$5"
    local cpu_before cpu_after

    # ghz rejects more connections than workers.
    local connections=$(( concurrency < CONNECTIONS ? concurrency : CONNECTIONS ))

    echo "  run $repeat/$REPEATS: $label @ concurrency=$concurrency (connections=$connections)"
    cpu_before="$(server_cpu_seconds)"
    ghz --insecure \
        --proto "$PROTO_DIR/services/db_service.proto" \
        --import-paths "$PROTO_DIR" \
        --call "services.db_service.DBService/$method" \
        --data-file "$PAYLOAD_DIR/$payload" \
        --concurrency "$concurrency" \
        --connections "$connections" \
        --total "$((TOTAL_REQUESTS + WARMUP_REQUESTS))" \
        --skipFirst "$WARMUP_REQUESTS" \
        --timeout "$REQUEST_TIMEOUT" \
        --format json \
        --output "$RESULTS_DIR/${label}_c${concurrency}_r${repeat}.json" \
        "$HOST:$PORT"

    # Fail the run if the server died or any request returned non-OK. ghz exits 0
    # either way.
    if ! kill -0 "$SERVER_PID" 2>/dev/null; then
        echo "error: server died during $label c=$concurrency" >&2
        tail -20 "$SERVER_LOG" >&2
        exit 1
    fi
    local status leftover
    status="$(grep -o '"statusCodeDistribution":{[^}]*}' \
        "$RESULTS_DIR/${label}_c${concurrency}_r${repeat}.json" || true)"
    # Anything left after removing the OK entry is a failure status.
    leftover="$(printf '%s' "$status" | sed 's/"statusCodeDistribution"//; s/"OK":[0-9]*//; s/[{}:,]//g')"
    if [ -z "$status" ] || [ -n "$leftover" ]; then
        echo "error: $label c=$concurrency did not return all-OK: ${status:-no status in report}" >&2
        exit 1
    fi

    cpu_after="$(server_cpu_seconds)"
    awk -v a="$cpu_before" -v b="$cpu_after" -v n="$((TOTAL_REQUESTS + WARMUP_REQUESTS))" \
        'BEGIN { printf "%.1f", (b - a) * 1000000 / n }' \
        > "$RESULTS_DIR/${label}_c${concurrency}_r${repeat}.cpu"
}

log "Benchmarking ($TOTAL_REQUESTS requests per run, $WARMUP_REQUESTS warmup, $REPEATS repeats)"

# Scenarios to benchmark
SCENARIOS="ping linear linear_5k linear_1k linear_100 hnsw hnsw_5k hnsw_1k hnsw_100"

# Repeats are the outer loop so background noise spreads across all configurations.
for repeat in $(seq 1 "$REPEATS"); do
    for concurrency in $CONCURRENCY_LEVELS; do
        for scenario in $SCENARIOS; do
            label="$(scenario_label "$scenario")"
            
            # Determine method and payload based on scenario
            if [ "$scenario" = "ping" ]; then
                method="Ping"
                payload="ping.json"
            else
                method="GetSimN"
                payload="getsimn_${scenario}.json"
            fi
            
            run_ghz "$label" "$concurrency" "$method" "$payload" "$repeat"
        done
    done
done

log "Summary"
"$HARNESS_BIN_DIR/summarize" "$RESULTS_DIR"

log "Results written to $RESULTS_DIR"
