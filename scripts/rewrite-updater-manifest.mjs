#!/usr/bin/env node
/**
 * 将 Tauri updater manifest（latest.json）中的 platforms.*.url
 * 改写为公开 GitHub 代理前缀，生成 latest-cn.json 供国内更新。
 * signature / version 原样保留（校验的是文件内容，与 URL 无关）。
 *
 * 用法:
 *   node scripts/rewrite-updater-manifest.mjs <input.json> <output.json> [proxyPrefix]
 *   GH_PROXY_PREFIX=https://ghfast.top/ node scripts/rewrite-updater-manifest.mjs in.json out.json
 */
import { readFileSync, writeFileSync } from "node:fs";

const GITHUB_PREFIX = "https://github.com/";
const DEFAULT_PROXY = "https://ghfast.top/";

function usage() {
  console.error(
    "Usage: node scripts/rewrite-updater-manifest.mjs <input.json> <output.json> [proxyPrefix]",
  );
  process.exit(1);
}

function normalizePrefix(prefix) {
  const raw = (prefix || DEFAULT_PROXY).trim();
  if (!raw) return DEFAULT_PROXY;
  return raw.endsWith("/") ? raw : `${raw}/`;
}

function main() {
  const [input, output, prefixArg] = process.argv.slice(2);
  if (!input || !output) usage();

  const prefix = normalizePrefix(prefixArg ?? process.env.GH_PROXY_PREFIX);
  const manifest = JSON.parse(readFileSync(input, "utf8"));
  const platforms = manifest.platforms;
  if (!platforms || typeof platforms !== "object") {
    console.error("Invalid manifest: missing platforms object");
    process.exit(1);
  }

  const entries = Object.entries(platforms);
  if (entries.length === 0) {
    console.error("Invalid manifest: platforms is empty");
    process.exit(1);
  }

  for (const [key, entry] of entries) {
    if (!entry || typeof entry.url !== "string" || typeof entry.signature !== "string") {
      console.error(`Platform ${key} is missing url or signature`);
      process.exit(1);
    }
    if (!entry.url.startsWith(GITHUB_PREFIX)) {
      console.error(`Platform ${key} url is not a github.com URL: ${entry.url}`);
      process.exit(1);
    }
    if (entry.url.startsWith(prefix)) {
      continue;
    }
    entry.url = `${prefix}${entry.url}`;
  }

  writeFileSync(output, `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
  console.log(
    `Wrote ${output} (${entries.length} platforms) with proxy prefix ${prefix}`,
  );
}

main();
