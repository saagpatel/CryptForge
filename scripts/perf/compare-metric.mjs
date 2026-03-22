import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";

const [baselinePath, currentPath, metric, maxRatio] = process.argv.slice(2);
if (!baselinePath || !currentPath || !metric || !maxRatio) {
  console.error("usage: node compare-metric.mjs <baseline.json> <current.json> <metric> <max_ratio>");
  process.exit(2);
}

const baseline = JSON.parse(readFileSync(baselinePath, "utf8"));
const current = JSON.parse(readFileSync(currentPath, "utf8"));
const c = current[metric];

if (typeof c !== "number" || !Number.isFinite(c)) {
  console.error(`Metric ${metric} missing or not numeric in current report.`);
  process.exit(2);
}

const baselineValue = baseline[metric];
const shouldBootstrap = process.env.PERF_BOOTSTRAP_BASELINE !== "0";

if (
  typeof baselineValue !== "number" ||
  !Number.isFinite(baselineValue) ||
  baselineValue <= 0
) {
  if (!shouldBootstrap) {
    console.error(
      `Baseline metric ${metric} is missing/invalid (${String(baselineValue)}). ` +
        "Run npm run perf:baseline:bootstrap first.",
    );
    process.exit(2);
  }

  const bootstrapped = {
    ...baseline,
    [metric]: c,
    capturedAt: new Date().toISOString(),
    bootstrapNote: `Initialized ${metric} from ${currentPath}`,
  };

  mkdirSync(dirname(baselinePath), { recursive: true });
  writeFileSync(baselinePath, `${JSON.stringify(bootstrapped, null, 2)}\n`);

  console.log(
    JSON.stringify(
      {
        metric,
        baseline: c,
        current: c,
        ratio: 0,
        bootstrapped: true,
      },
      null,
      2,
    ),
  );
  process.exit(0);
}

const b = baselineValue;
const ratio = (c - b) / b;
console.log(
  JSON.stringify(
    { metric, baseline: b, current: c, ratio, bootstrapped: false },
    null,
    2,
  ),
);

if (ratio > Number(maxRatio)) {
  console.error(
    `Regression on ${metric}: ${(ratio * 100).toFixed(2)}% > ${(Number(maxRatio) * 100).toFixed(2)}%`,
  );
  process.exit(1);
}
