import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";

const pairs = [
  {
    baseline: ".perf-baselines/bundle.json",
    current: ".perf-results/bundle.json",
    metric: "totalBytes",
  },
  {
    baseline: ".perf-baselines/build-time.json",
    current: ".perf-results/build-time.json",
    metric: "buildMs",
  },
];

mkdirSync(".perf-baselines", { recursive: true });

for (const pair of pairs) {
  if (!existsSync(pair.current)) {
    console.warn(
      `Skipping baseline bootstrap for ${pair.metric}: missing ${pair.current}`,
    );
    continue;
  }

  const current = JSON.parse(readFileSync(pair.current, "utf8"));
  const value = current[pair.metric];
  if (typeof value !== "number" || !Number.isFinite(value) || value <= 0) {
    console.warn(
      `Skipping baseline bootstrap for ${pair.metric}: invalid metric value ${String(value)}`,
    );
    continue;
  }

  const next = {
    [pair.metric]: value,
    capturedAt: new Date().toISOString(),
    source: pair.current,
  };

  writeFileSync(pair.baseline, `${JSON.stringify(next, null, 2)}\n`);
  console.log(`Bootstrapped ${pair.baseline} from ${pair.current}`);
}
