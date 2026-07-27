import { useState, useEffect } from 'react';
import './App.css';

interface Asset {
  platform: 'windows' | 'macos' | 'linux';
  architecture: 'x64' | 'arm64';
  fileName: string;
  downloadUrl: string;
  size?: number;
}

interface DownloadManifest {
  version: string;
  releaseUrl: string;
  publishedAt?: string;
  assets: Asset[];
}

const AUTHOR_NAME = 'Spirit Urban';
const AUTHOR_URL = 'https://spiriturban.github.io/';
const REPO_URL = 'https://github.com/SpiritUrban/file-forge';

function App() {
  const [manifest, setManifest] = useState<DownloadManifest | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [detectedOs, setDetectedOs] = useState<'windows' | 'macos' | 'linux'>('windows');

  useEffect(() => {
    // Detect OS
    const ua = navigator.userAgent.toLowerCase();
    if (ua.includes('mac')) {
      setDetectedOs('macos');
    } else if (ua.includes('linux')) {
      setDetectedOs('linux');
    } else {
      setDetectedOs('windows');
    }

    // Fetch manifest
    const fetchUrl = `${import.meta.env.BASE_URL}download-manifest.json`;
    fetch(fetchUrl)
      .then((res) => {
        if (!res.ok) throw new Error('Manifest not found');
        return res.json();
      })
      .then((data: DownloadManifest) => {
        setManifest(data);
        setLoading(false);
      })
      .catch((err) => {
        console.warn('Failed to load manifest:', err);
        setManifest({
          version: '0.1.0',
          releaseUrl: `${REPO_URL}/releases`,
          assets: [],
        });
        setLoading(false);
      });
  }, []);

  const formatSize = (bytes?: number) => {
    if (!bytes) return '';
    const mb = bytes / (1024 * 1024);
    return `${mb.toFixed(1)} MB`;
  };

  const getAssetsForPlatform = (platform: 'windows' | 'macos' | 'linux') => {
    if (!manifest) return [];
    return manifest.assets.filter((a) => a.platform === platform);
  };

  const windowsAssets = getAssetsForPlatform('windows');
  const macosAssets = getAssetsForPlatform('macos');
  const linuxAssets = getAssetsForPlatform('linux');

  return (
    <div className="site-wrapper">
      {/* Background Ambient Lights */}
      <div className="ambient-bg">
        <div className="orb orb-1"></div>
        <div className="orb orb-2"></div>
        <div className="orb orb-3"></div>
      </div>

      {/* Navigation Header */}
      <header className="site-nav">
        <div className="nav-container">
          <div className="brand">
            <span className="brand-logo">✨</span>
            <span className="brand-name">FileForge</span>
            <span className="version-pill">v{manifest?.version || '0.1.0'}</span>
          </div>
          <nav className="nav-links">
            <a href="#features">Features</a>
            <a href="#download">Download</a>
            <a href={REPO_URL} target="_blank" rel="noopener noreferrer" className="github-btn">
              <svg className="github-icon" viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
                <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z" />
              </svg>
              GitHub
            </a>
          </nav>
        </div>
      </header>

      {/* Hero Section */}
      <section className="hero-section">
        <div className="hero-container">
          <div className="hero-badge">
            <span className="sparkle">✨</span> High-Performance Media Optimizer
          </div>
          <h1 className="hero-title">
            Forge Smaller Files.<br />
            <span className="gradient-text">Zero Quality Loss.</span>
          </h1>
          <p className="hero-subtitle">
            FileForge is a lightning-fast desktop application designed for batch image optimization, WebP & SVG conversion, and high-efficiency video compression. 100% local, secure, and private.
          </p>

          {/* Download CTA Card */}
          <div className="download-cta-box" id="download">
            <div className="cta-header">
              <span className="os-badge">
                {detectedOs === 'windows' && '🪟 Windows Detected'}
                {detectedOs === 'macos' && '🍎 macOS Detected'}
                {detectedOs === 'linux' && '🐧 Linux Detected'}
              </span>
              <h2 className="cta-title">Get FileForge v{manifest?.version || '0.1.0'}</h2>
            </div>

            {loading ? (
              <div className="loading-spinner">Loading latest release...</div>
            ) : manifest && manifest.assets.length > 0 ? (
              <div className="download-options-grid">
                {/* Windows Card */}
                <div className={`platform-card ${detectedOs === 'windows' ? 'highlighted' : ''}`}>
                  <div className="platform-icon">🪟</div>
                  <div className="platform-name">Windows</div>
                  <div className="platform-arch">Windows 10 / 11 (64-bit)</div>
                  {windowsAssets.map((asset) => (
                    <a key={asset.downloadUrl} href={asset.downloadUrl} className="download-btn btn-primary">
                      <span>Download {asset.fileName.endsWith('.msi') ? 'MSI Installer' : 'Setup EXE'}</span>
                      {asset.size && <span className="file-size">{formatSize(asset.size)}</span>}
                    </a>
                  ))}
                </div>

                {/* macOS Card */}
                <div className={`platform-card ${detectedOs === 'macos' ? 'highlighted' : ''}`}>
                  <div className="platform-icon">🍎</div>
                  <div className="platform-name">macOS</div>
                  <div className="platform-arch">Apple Silicon & Intel</div>
                  {macosAssets.length > 0 ? (
                    macosAssets.map((asset) => (
                      <a key={asset.downloadUrl} href={asset.downloadUrl} className="download-btn btn-secondary">
                        <span>Download {asset.fileName.endsWith('.dmg') ? 'DMG' : 'App Archive'}</span>
                        {asset.size && <span className="file-size">{formatSize(asset.size)}</span>}
                      </a>
                    ))
                  ) : (
                    <a href={manifest.releaseUrl} target="_blank" rel="noopener noreferrer" className="download-btn btn-secondary">
                      View macOS Assets
                    </a>
                  )}
                </div>

                {/* Linux Card */}
                <div className={`platform-card ${detectedOs === 'linux' ? 'highlighted' : ''}`}>
                  <div className="platform-icon">🐧</div>
                  <div className="platform-name">Linux</div>
                  <div className="platform-arch">AppImage / DEB</div>
                  {linuxAssets.length > 0 ? (
                    linuxAssets.map((asset) => (
                      <a key={asset.downloadUrl} href={asset.downloadUrl} className="download-btn btn-secondary">
                        <span>Download {asset.fileName.endsWith('.deb') ? 'DEB Package' : 'AppImage'}</span>
                        {asset.size && <span className="file-size">{formatSize(asset.size)}</span>}
                      </a>
                    ))
                  ) : (
                    <a href={manifest.releaseUrl} target="_blank" rel="noopener noreferrer" className="download-btn btn-secondary">
                      View Linux Assets
                    </a>
                  )}
                </div>
              </div>
            ) : (
              <div className="fallback-download-box">
                <p className="fallback-text">
                  Releases are published directly on GitHub Releases. Click below to download the latest binary for your operating system.
                </p>
                <a href={manifest?.releaseUrl || `${REPO_URL}/releases`} target="_blank" rel="noopener noreferrer" className="download-btn btn-primary btn-large">
                  <span>Download from GitHub Releases</span>
                  <span className="arrow">➔</span>
                </a>
              </div>
            )}
          </div>
        </div>
      </section>

      {/* Feature Grid Section */}
      <section className="features-section" id="features">
        <div className="section-container">
          <div className="section-header">
            <h2 className="section-title">Built for Speed, Privacy & Precision</h2>
            <p className="section-subtitle">Everything you need to optimize photos, graphics, and video files in seconds.</p>
          </div>

          <div className="features-grid">
            <div className="feature-card">
              <div className="feature-icon">⚡</div>
              <h3 className="feature-title">Native Rust & Rayon Performance</h3>
              <p className="feature-desc">
                Leverages multi-core CPU parallelism with Rust & Tauri v2 for maximum processing speed without browser overhead.
              </p>
            </div>

            <div className="feature-card">
              <div className="feature-icon">🔒</div>
              <h3 className="feature-title">100% Offline & Private</h3>
              <p className="feature-desc">
                Your media files never leave your machine. No telemetry, no cloud servers, no bandwidth restrictions.
              </p>
            </div>

            <div className="feature-card">
              <div className="feature-icon">🖼️</div>
              <h3 className="feature-title">WebP & SVG Optimization</h3>
              <p className="feature-desc">
                Convert PNG/JPEG to modern WebP format and clean up SVG vector artwork by stripping unnecessary editor metadata.
              </p>
            </div>

            <div className="feature-card">
              <div className="feature-icon">🎥</div>
              <h3 className="feature-title">Video & Audio Compression</h3>
              <p className="feature-desc">
                Compress AVI, VOB, and MP4 videos using H.264 / H.265 (HEVC) presets, or extract pristine MP3 audio tracks via FFmpeg.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="site-footer">
        <div className="footer-container">
          <div className="footer-left">
            <span className="brand-logo">✨</span>
            <span className="footer-brand">FileForge</span>
            <span className="footer-copy">© 2026 Spirit Urban. Distributed under the MIT License.</span>
          </div>

          <div className="footer-right">
            <a href={AUTHOR_URL} target="_blank" rel="noopener noreferrer" className="author-link">
              <span className="sparkle-icon">✨</span> Built by <strong className="author-name">{AUTHOR_NAME}</strong>
            </a>
            <a href={REPO_URL} target="_blank" rel="noopener noreferrer" className="footer-link">Source Code</a>
            <a href={`${REPO_URL}/blob/main/LICENSE`} target="_blank" rel="noopener noreferrer" className="footer-link">License</a>
          </div>
        </div>
      </footer>
    </div>
  );
}

export default App;
