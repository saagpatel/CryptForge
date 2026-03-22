#!/usr/bin/env node
import { mkdtempSync, rmSync, symlinkSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { spawnSync } from "node:child_process";
import { cwd, argv, exit } from "node:process";

const markerIndex = argv.indexOf("--");
const commandArgs = markerIndex >= 0 ? argv.slice(markerIndex + 1) : argv.slice(2);

if (commandArgs.length === 0) {
  console.error("Usage: node scripts/run-in-safe-cwd.mjs -- <command> [args...]");
  exit(2);
}

const root = cwd();
const command = commandArgs[0];
const args = commandArgs.slice(1);

let safeCwd = root;
let cleanupDir;
let safeCargoTargetDir;

if (root.includes(":")) {
  const tmpBase = mkdtempSync(join(tmpdir(), "cryptforge-safe-"));
  const linkPath = join(tmpBase, "workspace");
  const cargoTargetPath = join(tmpdir(), "cryptforge-cargo-target");
  symlinkSync(root, linkPath, "dir");
  safeCwd = linkPath;
  safeCargoTargetDir = cargoTargetPath;
  cleanupDir = dirname(linkPath);
  console.log(`[info] Running in safe workspace alias: ${safeCwd}`);
}

const env = { ...process.env };
if (safeCargoTargetDir) {
  env.CARGO_TARGET_DIR = safeCargoTargetDir;
}

const result = spawnSync(command, args, {
  cwd: safeCwd,
  stdio: "inherit",
  env,
});

if (cleanupDir) {
  rmSync(cleanupDir, { recursive: true, force: true });
}

if (result.error) {
  console.error(result.error.message);
  exit(1);
}

exit(result.status ?? 1);
