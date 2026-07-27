import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

const owner = 'SpiritUrban';
const repo = 'file-forge';

const rootPkg = JSON.parse(fs.readFileSync(path.join(rootDir, 'package.json'), 'utf8'));
const defaultVersion = rootPkg.version || '0.1.0';

async function generateManifest() {
  const ref = process.env.GITHUB_REF_NAME || '';
  const isTag = /^v\d+\.\d+\.\d+$/.test(ref);
  const apiUrl = isTag
    ? `https://api.github.com/repos/${owner}/${repo}/releases/tags/${ref}`
    : `https://api.github.com/repos/${owner}/${repo}/releases/latest`;

  const headers = { 'User-Agent': `${repo}-site-builder` };
  if (process.env.GITHUB_TOKEN) {
    headers.Authorization = `Bearer ${process.env.GITHUB_TOKEN}`;
  }

  let version = defaultVersion;
  let releaseUrl = `https://github.com/${owner}/${repo}/releases`;
  let assets = [];
  let publishedAt = new Date().toISOString();

  try {
    console.log(`Fetching release data from ${apiUrl}...`);
    const res = await fetch(apiUrl, { headers });
    if (res.ok) {
      const data = await res.json();
      version = (data.tag_name || defaultVersion).replace(/^v/, '');
      releaseUrl = data.html_url || releaseUrl;
      publishedAt = data.published_at || publishedAt;

      assets = (data.assets || [])
        .filter((a) => {
          const n = a.name.toLowerCase();
          return !n.endsWith('.sig') && n !== 'latest.json';
        })
        .map((a) => {
          const n = a.name.toLowerCase();
          let platform = 'windows';
          if (n.includes('macos') || n.includes('darwin') || n.endsWith('.dmg') || n.endsWith('.app.tar.gz')) {
            platform = 'macos';
          } else if (n.includes('linux') || n.endsWith('.appimage') || n.endsWith('.deb') || n.endsWith('.rpm')) {
            platform = 'linux';
          }

          const architecture = n.includes('arm64') || n.includes('aarch64') ? 'arm64' : 'x64';

          return {
            platform,
            architecture,
            fileName: a.name,
            downloadUrl: a.browser_download_url,
            size: a.size,
          };
        });
      console.log(`Successfully fetched release ${version} with ${assets.length} assets.`);
    } else {
      console.warn(`Release API returned status ${res.status}. Falling back to default version ${defaultVersion} with empty assets.`);
    }
  } catch (err) {
    console.warn(`Failed to fetch release from GitHub API: ${err.message}. Using fallback manifest.`);
  }

  const manifest = {
    version,
    releaseUrl,
    publishedAt,
    assets,
  };

  // Ensure output folders exist
  const sitePublicDir = path.join(rootDir, 'site', 'public');
  if (!fs.existsSync(sitePublicDir)) {
    fs.mkdirSync(sitePublicDir, { recursive: true });
  }
  const rootPublicDir = path.join(rootDir, 'public');
  if (!fs.existsSync(rootPublicDir)) {
    fs.mkdirSync(rootPublicDir, { recursive: true });
  }

  const manifestContent = JSON.stringify(manifest, null, 2) + '\n';
  fs.writeFileSync(path.join(sitePublicDir, 'download-manifest.json'), manifestContent, 'utf8');
  fs.writeFileSync(path.join(rootPublicDir, 'download-manifest.json'), manifestContent, 'utf8');

  console.log(`Manifest written successfully to site/public/download-manifest.json and public/download-manifest.json`);
}

generateManifest();
