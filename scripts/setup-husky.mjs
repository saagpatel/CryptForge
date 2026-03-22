#!/usr/bin/env node
import { existsSync } from "node:fs";
import { spawnSync } from "node:child_process";

const huskyBin = "./node_modules/husky/bin.js";

if (!existsSync(huskyBin)) {
  console.log("[info] husky not installed yet; skipping hook setup");
  process.exit(0);
}

const result = spawnSync(process.execPath, [huskyBin], {
  stdio: "inherit",
  env: process.env,
});

process.exit(result.status ?? 0);
