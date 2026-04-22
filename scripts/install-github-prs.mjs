#!/usr/bin/env node
// Copy extensions/github-prs/ into the active storage's extensions/ directory.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';

function appDataDir() {
    if (process.platform === 'win32') return path.join(process.env.APPDATA || '', 'com.littlebrushgames.feature-hub');
    if (process.platform === 'darwin') return path.join(os.homedir(), 'Library', 'Application Support', 'com.littlebrushgames.feature-hub');
    return path.join(process.env.XDG_DATA_HOME || path.join(os.homedir(), '.local', 'share'), 'com.littlebrushgames.feature-hub');
}

const cfgPath = path.join(appDataDir(), 'config.json');
if (!fs.existsSync(cfgPath)) {
    console.error(`No FH config found at ${cfgPath}. Open Feature Hub once to set up a storage.`);
    process.exit(1);
}
const cfg = JSON.parse(fs.readFileSync(cfgPath, 'utf8'));
const active = cfg.storages.find(s => s.id === cfg.active_storage_id);
if (!active) { console.error('No active storage configured.'); process.exit(1); }

const src = path.resolve('extensions/github-prs');
const dst = path.join(active.path, 'extensions', 'github-prs');

if (!fs.existsSync(src)) {
    console.error(`Source not found: ${src}`);
    process.exit(1);
}

fs.mkdirSync(path.dirname(dst), { recursive: true });
fs.rmSync(dst, { recursive: true, force: true });
fs.cpSync(src, dst, { recursive: true });
console.log(`Copied ${src} → ${dst}`);
console.log('Restart Feature Hub to load the extension.');
