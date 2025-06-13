#!/bin/bash


tarantool-runner run -p ./target/debug/libintegration_suite_app."$LIB_EXT" -e run_server &
task_pid=$!

cleanup() {
  echo "Cleaning up… killing PID'n'kids $task_pid"
  pkill -TERM -P $task_pid
}
trap cleanup EXIT INT TERM

HEALTH_URL="http://localhost:18989/echo"
until curl --silent --fail --output /dev/null "$HEALTH_URL"; do
  echo "Waiting for $HEALTH_URL to return 200…"
  sleep 2
done

pytest