#!/usr/bin/env node
import { execFileSync } from "node:child_process";
import { cwd } from "node:process";

const projectRoot = cwd();
let failed = false;

function log(status, message) {
  const prefix = status ? "[ok]" : "[error]";
  console.log(`${prefix} ${message}`);
}

function warn(message) {
  console.warn(`[warn] ${message}`);
}

function checkCommand(bin, args = ["--version"]) {
  try {
    const out = execFileSync(bin, args, { encoding: "utf8" }).trim();
    log(true, `${bin}: ${out}`);
    return out;
  } catch {
    log(false, `${bin} is required but not available`);
    failed = true;
    return "";
  }
}

if (projectRoot.includes(":")) {
  warn("The current path contains ':'. Rust and pnpm can fail in this directory.");
  warn("Rust commands are wrapped via scripts/run-in-safe-cwd.mjs to avoid failures.");
} else {
  log(true, "Project path is separator-safe.");
}

const nodeVersion = checkCommand("node");
const npmVersion = checkCommand("npm");
checkCommand("cargo");
checkCommand("rustc");

const nodeMajor = Number(nodeVersion.replace(/^v/, "").split(".")[0]);
if (!Number.isFinite(nodeMajor) || nodeMajor < 20) {
  log(false, `Node.js >=20 is required (found ${nodeVersion || "unknown"})`);
  failed = true;
}

const npmMajor = Number(npmVersion.split(".")[0]);
if (!Number.isFinite(npmMajor) || npmMajor < 10) {
  log(false, `npm >=10 is required (found ${npmVersion || "unknown"})`);
  failed = true;
}

if (failed) {
  process.exit(1);
}
