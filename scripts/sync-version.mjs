import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

const versionArg = process.argv[2];
if (!versionArg) {
  console.error('Usage: node scripts/sync-version.mjs <version>');
  console.error('Example: node scripts/sync-version.mjs 0.1.0');
  process.exit(1);
}

const cleanVersion = versionArg.replace(/^v/, '');
if (!/^\d+\.\d+\.\d+$/.test(cleanVersion)) {
  console.error(`Invalid version format: "${versionArg}". Expected semver (e.g. 0.1.0).`);
  process.exit(1);
}

console.log(`Syncing version ${cleanVersion}...`);

// 1. Root package.json
const rootPkgPath = path.join(rootDir, 'package.json');
if (fs.existsSync(rootPkgPath)) {
  const pkg = JSON.parse(fs.readFileSync(rootPkgPath, 'utf8'));
  pkg.version = cleanVersion;
  fs.writeFileSync(rootPkgPath, JSON.stringify(pkg, null, 2) + '\n', 'utf8');
  console.log(`Updated ${rootPkgPath}`);
}

// 2. Site package.json (if exists)
const sitePkgPath = path.join(rootDir, 'site', 'package.json');
if (fs.existsSync(sitePkgPath)) {
  const pkg = JSON.parse(fs.readFileSync(sitePkgPath, 'utf8'));
  pkg.version = cleanVersion;
  fs.writeFileSync(sitePkgPath, JSON.stringify(pkg, null, 2) + '\n', 'utf8');
  console.log(`Updated ${sitePkgPath}`);
}

// 3. tauri.conf.json
const tauriConfPath = path.join(rootDir, 'src-tauri', 'tauri.conf.json');
if (fs.existsSync(tauriConfPath)) {
  const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
  tauriConf.version = cleanVersion;
  fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n', 'utf8');
  console.log(`Updated ${tauriConfPath}`);
}

// 4. Cargo.toml
const cargoTomlPath = path.join(rootDir, 'src-tauri', 'Cargo.toml');
if (fs.existsSync(cargoTomlPath)) {
  let content = fs.readFileSync(cargoTomlPath, 'utf8');
  content = content.replace(/^version\s*=\s*"[^"]+"/m, `version = "${cleanVersion}"`);
  fs.writeFileSync(cargoTomlPath, content, 'utf8');
  console.log(`Updated ${cargoTomlPath}`);
}

// 5. Cargo.lock (regex replacement of package version)
const cargoLockPath = path.join(rootDir, 'src-tauri', 'Cargo.lock');
if (fs.existsSync(cargoLockPath)) {
  let content = fs.readFileSync(cargoLockPath, 'utf8');
  // Match name = "file_forge"\nversion = "..."
  const regex = /(name\s*=\s*"file_forge"\s*\nversion\s*=\s*")[^"]+(")/;
  if (regex.test(content)) {
    content = content.replace(regex, `$1${cleanVersion}$2`);
    fs.writeFileSync(cargoLockPath, content, 'utf8');
    console.log(`Updated ${cargoLockPath}`);
  } else {
    console.warn(`Could not find package "file_forge" in Cargo.lock to update version.`);
  }
}

console.log(`Version synchronization to ${cleanVersion} completed successfully!`);
