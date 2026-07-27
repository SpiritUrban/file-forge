import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

const versions = {};

// 1. package.json
const rootPkgPath = path.join(rootDir, 'package.json');
if (fs.existsSync(rootPkgPath)) {
  const pkg = JSON.parse(fs.readFileSync(rootPkgPath, 'utf8'));
  versions['package.json'] = pkg.version;
}

// 2. tauri.conf.json
const tauriConfPath = path.join(rootDir, 'src-tauri', 'tauri.conf.json');
if (fs.existsSync(tauriConfPath)) {
  const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
  versions['tauri.conf.json'] = tauriConf.version;
}

// 3. Cargo.toml
const cargoTomlPath = path.join(rootDir, 'src-tauri', 'Cargo.toml');
if (fs.existsSync(cargoTomlPath)) {
  const content = fs.readFileSync(cargoTomlPath, 'utf8');
  const match = content.match(/^version\s*=\s*"([^"]+)"/m);
  if (match) {
    versions['Cargo.toml'] = match[1];
  }
}

// 4. Cargo.lock
const cargoLockPath = path.join(rootDir, 'src-tauri', 'Cargo.lock');
if (fs.existsSync(cargoLockPath)) {
  const content = fs.readFileSync(cargoLockPath, 'utf8');
  const match = content.match(/name\s*=\s*"file_forge"\s*\nversion\s*=\s*"([^"]+)"/);
  if (match) {
    versions['Cargo.lock'] = match[1];
  }
}

console.log('Checked file versions:', versions);

const versionValues = Object.values(versions);
if (versionValues.length === 0) {
  console.error('No version files found!');
  process.exit(1);
}

const firstVersion = versionValues[0];
const mismatch = Object.entries(versions).find(([file, ver]) => ver !== firstVersion);

if (mismatch) {
  console.error(`Version mismatch detected! ${mismatch[0]} has version "${mismatch[1]}", expected "${firstVersion}".`);
  console.error('Run "npm run version:sync <version>" to fix.');
  process.exit(1);
}

// Check tag if running in GitHub Actions tag push
const refName = process.env.GITHUB_REF_NAME || '';
if (refName.startsWith('v')) {
  const tagVersion = refName.replace(/^v/, '');
  if (tagVersion !== firstVersion) {
    console.error(`Tag version mismatch! GITHUB_REF_NAME tag is "${refName}" (version "${tagVersion}"), but project files have version "${firstVersion}".`);
    process.exit(1);
  }
  console.log(`Tag version "${refName}" matches project version "${firstVersion}".`);
}

console.log(`All versions match! Current version: ${firstVersion}`);
