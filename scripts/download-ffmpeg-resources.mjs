import fs from 'node:fs';
import path from 'node:path';
import https from 'node:https';
import { fileURLToPath } from 'node:url';
import { execSync } from 'node:child_process';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');
const resourcesDir = path.join(rootDir, 'src-tauri', 'resources');

function getFFmpegPlatformKey() {
  const arg = process.argv[2];
  let p = arg;
  if (!p) {
    const platform = process.platform;
    const arch = process.arch;
    if (platform === 'win32') p = 'win-64';
    else if (platform === 'linux') p = arch === 'arm64' ? 'linux-arm-64' : 'linux-64';
    else if (platform === 'darwin') p = 'macos-64';
    else p = 'win-64';
  }

  if (p === 'osx-64' || p === 'macos-64' || p === 'macos' || p === 'darwin') return 'macos-64';
  if (p === 'win-64' || p === 'win' || p === 'windows') return 'win-64';
  if (p === 'linux-64' || p === 'linux') return 'linux-64';
  if (p === 'linux-arm64') return 'linux-arm-64';
  return p;
}

const targetPlatform = getFFmpegPlatformKey();
console.log(`Downloading static FFmpeg binaries for platform key: ${targetPlatform}...`);

if (!fs.existsSync(resourcesDir)) {
  fs.mkdirSync(resourcesDir, { recursive: true });
}

const baseUrl = 'https://github.com/ffbinaries/ffbinaries-prebuilt/releases/download/v6.1';
const ffmpegUrl = `${baseUrl}/ffmpeg-6.1-${targetPlatform}.zip`;
const ffprobeUrl = `${baseUrl}/ffprobe-6.1-${targetPlatform}.zip`;

async function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const follow = (currentUrl) => {
      https.get(currentUrl, (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          return follow(res.headers.location);
        }
        if (res.statusCode !== 200) {
          return reject(new Error(`Failed to download ${currentUrl}: HTTP ${res.statusCode}`));
        }
        const fileStream = fs.createWriteStream(dest);
        res.pipe(fileStream);
        fileStream.on('finish', () => {
          fileStream.close(() => resolve());
        });
      }).on('error', reject);
    };
    follow(url);
  });
}

async function unzip(zipPath, outDir) {
  if (process.platform === 'win32') {
    execSync(`powershell -Command "Expand-Archive -Path '${zipPath}' -DestinationPath '${outDir}' -Force"`);
  } else {
    execSync(`unzip -o "${zipPath}" -d "${outDir}"`);
  }
}

async function main() {
  const ffmpegZip = path.join(resourcesDir, 'ffmpeg.zip');
  const ffprobeZip = path.join(resourcesDir, 'ffprobe.zip');

  console.log(`Fetching ffmpeg from ${ffmpegUrl}...`);
  await downloadFile(ffmpegUrl, ffmpegZip);
  await unzip(ffmpegZip, resourcesDir);
  fs.unlinkSync(ffmpegZip);

  console.log(`Fetching ffprobe from ${ffprobeUrl}...`);
  await downloadFile(ffprobeUrl, ffprobeZip);
  await unzip(ffprobeZip, resourcesDir);
  fs.unlinkSync(ffprobeZip);

  if (process.platform !== 'win32') {
    try {
      execSync(`chmod +x "${path.join(resourcesDir, 'ffmpeg')}" "${path.join(resourcesDir, 'ffprobe')}"`);
    } catch (e) {
      // ignore
    }
  }

  console.log('FFmpeg binaries successfully downloaded & extracted into src-tauri/resources!');
}

main().catch((err) => {
  console.error('Failed to download FFmpeg binaries:', err);
  process.exit(1);
});
