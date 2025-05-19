#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
TARGET=${1:-weaver}
PORT=${2:-19000}

PORT=$PORT tarantool-runner run -p ./target/release/lib"${TARGET}"_bench.so -e run_server &
task_pid=$!

cleanup() {
  echo "Cleaning up… killing PID'n'kids $task_pid"
  pkill -TERM -P $task_pid
}

trap cleanup EXIT INT TERM

HEALTH_URL="http://localhost:$PORT/health"
until curl --silent --fail --output /dev/null "$HEALTH_URL"; do
  echo "Waiting for $HEALTH_URL to return 200…"
  sleep 2
done

K6_WEB_DASHBOARD=true K6_WEB_DASHBOARD_EXPORT="$SCRIPT_DIR"/../../data/bench_summary_"${TARGET}".html k6 run -e PORT="$PORT" -e HTML_REPORT_PATH= --summary-export="$SCRIPT_DIR"/../../data/bench_summary_"${TARGET}".json "$SCRIPT_DIR"/bench.ts