// Copies built fh and fh-mcp binaries to the system install directory.
// Windows: %LOCALAPPDATA%\Programs\FeatureHub\
// macOS/Linux: ~/.local/bin/

import { cpSync, mkdirSync, existsSync } from "fs";
import { join } from "path";

const isWindows = process.platform === "win32";
const ext = isWindows ? ".exe" : "";
const isRelease = process.argv.includes("--release");
const srcDir = join("src-tauri", "target", isRelease ? "release" : "debug");
const binaries = [`fh${ext}`, `fh-mcp${ext}`];

// Determine install directory
let installDir;
if (isWindows) {
  const localAppData = process.env.LOCALAPPDATA;
  if (!localAppData) {
    console.error("LOCALAPPDATA not set");
    process.exit(1);
  }
  installDir = join(localAppData, "Programs", "FeatureHub");
} else {
  installDir = join(process.env.HOME, ".local", "bin");
}

mkdirSync(installDir, { recursive: true });

let copied = 0;
for (const bin of binaries) {
  const src = join(srcDir, bin);
  if (!existsSync(src)) {
    console.warn(`  skip ${bin} (not found)`);
    continue;
  }
  const dest = join(installDir, bin);
  cpSync(src, dest);
  console.log(`  ${bin} -> ${dest}`);
  copied++;
}

if (copied > 0) {
  console.log(`Installed ${copied} binary(ies) to ${installDir}`);
} else {
  console.error("No binaries found. Run 'cargo build' in src-tauri/ first.");
  process.exit(1);
}
