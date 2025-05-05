import http, { Response } from "k6/http";
import { check } from "k6";
import { isEqual } from "https://raw.githubusercontent.com/lodash/lodash/4.17.15-npm/core.js";

const HOST = "http://localhost:" + __ENV.PORT;

export const options = {
  scenarios: {
    warmup: {
      executor: "constant-vus",
      exec: "warmUp",
      vus: 2,
      duration: "10s",
    },
    load_test: {
      executor: "ramping-arrival-rate",
      exec: "runTest",
      startRate: 1000, // initial iterations/sec
      timeUnit: "1s", // per-second granularity
      preAllocatedVUs: 1000, // pre-allocate enough VUs
      maxVUs: 10000, // allow up to 10k VUs
      stages: [
        { target: 5000, duration: "1m" }, // ramp to 5 000 RPS
        { target: 20000, duration: "2m" }, // ramp to 20 000 RPS
        { target: 40000, duration: "2m" }, // then to 40 000 RPS over 2 min
      ],
      startTime: "10s", // start after warmup
    },
  },
};

export function warmUp(): void {
  http.get(`${HOST}/health`);
}

export function runTest(): void {
  const request = {
    some_string: "some_string",
    some_int: 1,
    properties: {
      prop_a: "prop_a",
      prop_b: "prop_b",
    },
  };
  const res: Response = http.post(
    `${HOST}/test/1/subcommand/2`,
    JSON.stringify(request)
  );
  const json = JSON.parse(res.body);

  check(res, {
    "status is 200": (r) => r.status === 200,
    "request capture is valid": (r) => {
      return isEqual(json.request, request);
    },
    "path capture is valid": (r) => {
      return json.path.param_a === "1" && json.path.param_b === "2";
    },
  });
}
