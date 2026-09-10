#!/usr/bin/env bash
#
# Baseline Set QPS benchmark for ahnlich-db.
#
# Starts a server, creates stores with deterministic synthetic vectors, drives Set
# through ghz at several concurrency levels, verifies the stores, writes a summary.
#
# Settings below are overridable:
#   TOTAL_REQUESTS=50000 CONCURRENCY_LEVELS="1 8 32" ./run_set.sh
# Compare prebuilt binaries with identical settings:
#   SERVER_BIN=/tmp/control/ahnlich-db RESULTS_DIR=/tmp/set-control ./run_set.sh
#   SERVER_BIN=/tmp/candidate/ahnlich-db RESULTS_DIR=/tmp/set-candidate ./run_set.sh
#
set -euo pipefail

HOST="${HOST:-127.0.0.1}"
PORT="${PORT:-1369}"
CONCURRENCY_LEVELS="${CONCURRENCY_LEVELS:-1 10 50 100}"
TOTAL_REQUESTS="${TOTAL_REQUESTS:-10000}"
BATCH_SIZE="${BATCH_SIZE:-1000}"
VECTOR_DIMENSION="${VECTOR_DIMENSION:-128}"
REQUEST_TIMEOUT="${REQUEST_TIMEOUT:-60s}"

# Capped at the concurrency level; ghz rejects more connections than workers.
CONNECTIONS="${CONNECTIONS:-8}"

WARMUP_REQUESTS="${WARMUP_REQUESTS:-500}"
POOL_REQUESTS="${POOL_REQUESTS:-32}"

# Number of runs per configuration. Results are the median of these.
REPEATS="${REPEATS:-3}"

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd)"
AHNLICH_DIR="$REPO_DIR/ahnlich"
PROTO_DIR="$REPO_DIR/protos"

RESULTS_DIR="${RESULTS_DIR:-$SCRIPT_DIR/results/set_$(date +%Y%m%d_%H%M%S)}"
PAYLOAD_DIR="$RESULTS_DIR/payloads"
SPEC_DIR="$RESULTS_DIR/specs"
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

mkdir -p "$RESULTS_DIR" "$PAYLOAD_DIR" "$SPEC_DIR"

# Resolve the output directory from cargo rather than assuming ./target.
release_dir_for() {
    local manifest="$1" target_dir
    target_dir="$(cargo metadata --no-deps --format-version 1 --manifest-path "$manifest" \
        | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')"
    [ -n "$target_dir" ] || { echo "error: could not read target dir for $manifest" >&2; exit 1; }
    echo "$target_dir/release"
}

if [ -n "${SERVER_BIN:-}" ]; then
    [ -x "$SERVER_BIN" ] || { echo "error: SERVER_BIN is not executable: $SERVER_BIN" >&2; exit 1; }
    log "Using prebuilt ahnlich-db: $SERVER_BIN"
else
    log "Building ahnlich-db"
    cargo build --release --manifest-path "$AHNLICH_DIR/Cargo.toml" --bin ahnlich-db
    SERVER_BIN_DIR="$(release_dir_for "$AHNLICH_DIR/Cargo.toml")"
    SERVER_BIN="$SERVER_BIN_DIR/ahnlich-db"
fi

log "Building harness"
cargo build --release --manifest-path "$SCRIPT_DIR/Cargo.toml" --bins
HARNESS_BIN_DIR="$(release_dir_for "$SCRIPT_DIR/Cargo.toml")"

for bin in setup_set summarize; do
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

# Pin the flags that affect performance so a changed default does not appear as a
# benchmark result.
SERVER_ARGS=(
    run
    --host "$HOST"
    --port "$PORT"
    --threadpool-size "${THREADPOOL_SIZE:-16}"
    --size-calculation-interval "${SIZE_CALCULATION_INTERVAL:-60}"
)

SCENARIOS="${SCENARIOS:-set_no_index set_1_index set_4_indexes}"

{
    echo "commit: $(git -C "$REPO_DIR" rev-parse --short HEAD 2>/dev/null || echo unknown)"
    if [ -n "$(git -C "$REPO_DIR" status --porcelain 2>/dev/null)" ]; then
        echo "tree: dirty"
    fi
    echo "host: $(uname -srm)"
    echo "ghz: $(ghz --version 2>&1 >/dev/null)"
    echo "server binary: $SERVER_BIN"
    echo "server sha256: $(shasum -a 256 "$SERVER_BIN" | awk '{print $1}')"
    echo "server: ahnlich-db ${SERVER_ARGS[*]}"
    echo "requests: $TOTAL_REQUESTS x $REPEATS repeats, warmup $WARMUP_REQUESTS"
    echo "concurrency: $CONCURRENCY_LEVELS, connections: $CONNECTIONS"
    echo "scenarios: $SCENARIOS"
    echo "batch_size: $BATCH_SIZE, dimension: $VECTOR_DIMENSION, pool_requests: $POOL_REQUESTS"
} > "$RESULTS_DIR/RUN.txt"

log "Starting ahnlich-db on $HOST:$PORT"
"$SERVER_BIN" "${SERVER_ARGS[@]}" >"$SERVER_LOG" 2>&1 &
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

scenario_config() {
    case "$1" in
        set_no_index) echo "0 100" ;;
        set_1_index) echo "1 100" ;;
        set_4_indexes) echo "4 100" ;;
        *) echo "error: unknown scenario '$1'" >&2; return 1 ;;
    esac
}

log "Creating stores and generating ghz payloads"
for scenario in $SCENARIOS; do
    read -r indexed_fields cardinality <<< "$(scenario_config "$scenario")"
    spec="$SPEC_DIR/$scenario.json"
    payload="$PAYLOAD_DIR/$scenario.json"
    cat > "$spec" <<EOF
{
  "store": "$scenario",
  "workload": "update",
  "batch": $BATCH_SIZE,
  "dimension": $VECTOR_DIMENSION,
  "indexed_fields": $indexed_fields,
  "cardinality": $cardinality,
  "pool_requests": $POOL_REQUESTS,
  "total_requests": $TOTAL_REQUESTS,
  "payload_file": "$payload"
}
EOF
    "$HARNESS_BIN_DIR/setup_set" generate "$spec"
    AHNLICH_DB_ADDR="$HOST:$PORT" "$HARNESS_BIN_DIR/setup_set" prepare "$spec"
done

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

run_ghz() {
    local label="$1" concurrency="$2" repeat="$3"
    local cpu_before cpu_after
    local connections=$(( concurrency < CONNECTIONS ? concurrency : CONNECTIONS ))

    echo "  run $repeat/$REPEATS: $label @ concurrency=$concurrency (connections=$connections)"
    cpu_before="$(server_cpu_seconds)"
    ghz --insecure \
        --proto "$PROTO_DIR/services/db_service.proto" \
        --import-paths "$PROTO_DIR" \
        --call "services.db_service.DBService/Set" \
        --data-file "$PAYLOAD_DIR/$label.json" \
        --disable-template-data \
        --concurrency "$concurrency" \
        --connections "$connections" \
        --total "$((TOTAL_REQUESTS + WARMUP_REQUESTS))" \
        --skipFirst "$WARMUP_REQUESTS" \
        --timeout "$REQUEST_TIMEOUT" \
        --format json \
        --output "$RESULTS_DIR/${label}_c${concurrency}_r${repeat}.json" \
        "$HOST:$PORT"

    if ! kill -0 "$SERVER_PID" 2>/dev/null; then
        echo "error: server died during $label c=$concurrency" >&2
        tail -20 "$SERVER_LOG" >&2
        exit 1
    fi
    local status leftover
    status="$(grep -o '"statusCodeDistribution":{[^}]*}' \
        "$RESULTS_DIR/${label}_c${concurrency}_r${repeat}.json" || true)"
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

# Repeats are the outer loop so background noise spreads across all configurations.
for repeat in $(seq 1 "$REPEATS"); do
    for concurrency in $CONCURRENCY_LEVELS; do
        for scenario in $SCENARIOS; do
            run_ghz "$scenario" "$concurrency" "$repeat"
        done
    done
done

log "Verifying stores"
for scenario in $SCENARIOS; do
    AHNLICH_DB_ADDR="$HOST:$PORT" \
        "$HARNESS_BIN_DIR/setup_set" verify "$SPEC_DIR/$scenario.json"
done

log "Summary"
"$HARNESS_BIN_DIR/summarize" "$RESULTS_DIR"

log "Results written to $RESULTS_DIR"
