import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { JobProgress, FolderSelection } from "./types/job";
import { PRODUCT_METADATA } from "./metadata";
import { openUrl } from "@tauri-apps/plugin-opener";
import "./App.css";

function App() {
  const [inputPath, setInputPath] = useState<string>("");
  const [outputPath, setOutputPath] = useState<string>("");
  const [error, setError] = useState<string | null>(null);
  const [convertPngToWebp, setConvertPngToWebp] = useState<boolean>(false);
  const [optimizeSvg, setOptimizeSvg] = useState<boolean>(true);
  const [optimizeWebp, setOptimizeWebp] = useState<boolean>(true);
  const [jpegQuality, setJpegQuality] = useState<number>(82);
  const [resizeImages, setResizeImages] = useState<boolean>(false);
  const [maxWidth, setMaxWidth] = useState<number>(1920);
  const [maxHeight, setMaxHeight] = useState<number>(1080);
  const [convertVideo, setConvertVideo] = useState<boolean>(false);
  const [videoCrf, setVideoCrf] = useState<number>(23);
  const [optimizeMp4, setOptimizeMp4] = useState<boolean>(false);
  const [useH265, setUseH265] = useState<boolean>(false);
  const [extractAudio, setExtractAudio] = useState<boolean>(false);
  const [convertWavToMp3, setConvertWavToMp3] = useState<boolean>(false);
  const [mp3Bitrate, setMp3Bitrate] = useState<number>(128);
  const [convertGifToMp4, setConvertGifToMp4] = useState<boolean>(false);
  const [ffmpegAvailable, setFfmpegAvailable] = useState<boolean | null>(null);
  const [theme, setTheme] = useState<"light" | "dark">(
    (localStorage.getItem("theme") as any) || "light"
  );
  const [isCancelling, setIsCancelling] = useState<boolean>(false);
  const [progress, setProgress] = useState<JobProgress>({
    status: "idle",
    totalFiles: 0,
    processedFiles: 0,
    currentFile: null,
    optimizedFiles: 0,
    copiedFiles: 0,
    originalKeptFiles: 0,
    skippedFiles: 0,
    failedFiles: 0,
    originalBytes: 0,
    outputBytes: 0,
  });

  // Sync initial state and listen to backend events
  useEffect(() => {
    // 1. Get current job progress on load
    invoke<JobProgress>("get_job_progress")
      .then((initialProgress) => {
        setProgress(initialProgress);
      })
      .catch((err) => {
        console.error("Failed to fetch initial job progress:", err);
      });

    // 2. Listen to job-progress updates
    let unlistenProgress: () => void;
    let unlistenError: () => void;

    const setupListeners = async () => {
      unlistenProgress = await listen<JobProgress>("job-progress", (event) => {
        setProgress(event.payload);
        if (event.payload.status !== "processing" && event.payload.status !== "scanning") {
          setIsCancelling(false);
        }
      });

      unlistenError = await listen<string>("job-error", (event) => {
        setError(event.payload);
      });
    };

    setupListeners();

    return () => {
      if (unlistenProgress) unlistenProgress();
      if (unlistenError) unlistenError();
    };
  }, []);

  // Sync theme
  useEffect(() => {
    if (theme === "dark") {
      document.body.classList.add("dark");
    } else {
      document.body.classList.remove("dark");
    }
    localStorage.setItem("theme", theme);
  }, [theme]);

  // Check FFmpeg availability on mount
  useEffect(() => {
    invoke<boolean>("check_ffmpeg").then(setFfmpegAvailable).catch(() => setFfmpegAvailable(false));
  }, []);

  const handleOpenAuthor = async () => {
    try {
      await openUrl(PRODUCT_METADATA.authorUrl);
    } catch {
      window.open(PRODUCT_METADATA.authorUrl, "_blank");
    }
  };

  const handleSelectFolder = async () => {
    try {
      setError(null);
      const result = await invoke<FolderSelection | null>("select_folder");
      if (result) {
        setInputPath(result.inputPath);
        setOutputPath(result.outputPath);
      }
    } catch (err: any) {
      setError(err.toString());
    }
  };

  const handleStartOptimization = async () => {
    if (!inputPath || !outputPath) return;
    try {
      setError(null);
      await invoke("start_optimization", { 
        inputPath, 
        outputPath,
        options: {
          convertPngToWebp,
          optimizeSvg,
          optimizeWebp,
          jpegQuality,
          resizeImages,
          maxWidth,
          maxHeight,
          convertVideo,
          videoCrf,
          useH265,
          extractAudio,
          convertWavToMp3,
          mp3Bitrate,
          optimizeMp4,
          convertGifToMp4,
        }
      });
    } catch (err: any) {
      setError(err.toString());
    }
  };

  const handleOpenFolder = async () => {
    if (!outputPath) return;
    try {
      await invoke("open_folder", { path: outputPath });
    } catch (err: any) {
      console.error("Failed to open folder:", err);
    }
  };

  const handleCancel = async () => {
    try {
      setIsCancelling(true);
      await invoke("cancel_optimization");
    } catch (err: any) {
      setError(err.toString());
      setIsCancelling(false);
    }
  };

  const handleReset = () => {
    setInputPath("");
    setOutputPath("");
    setError(null);
    setIsCancelling(false);
    setProgress({
      status: "idle",
      totalFiles: 0,
      processedFiles: 0,
      currentFile: null,
      optimizedFiles: 0,
      copiedFiles: 0,
      originalKeptFiles: 0,
      skippedFiles: 0,
      failedFiles: 0,
      originalBytes: 0,
      outputBytes: 0,
    });
  };

  // Bytes Formatter
  const formatBytes = (bytes: number, decimals = 2) => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + " " + sizes[i];
  };

  const status = progress.status;
  const isBusy = status === "scanning" || status === "processing";

  // Calculations for Completed screen
  const savedBytes = progress.originalBytes - progress.outputBytes;
  const savedPercent = progress.originalBytes > 0 ? (savedBytes / progress.originalBytes) * 100 : 0;
  const processedPercent = progress.totalFiles > 0 ? (progress.processedFiles / progress.totalFiles) * 100 : 0;

  return (
    <div className="app-container">
      <div className="card">
        {/* Brand Header */}
        <header className="app-header">
          <div className="header-top-row">
            <div className="logo-container">
              <span className="logo-icon">✨</span>
              <h1 className="logo-text">FileForge</h1>
            </div>
            <button
              className="theme-toggle-btn"
              onClick={() => setTheme(theme === "light" ? "dark" : "light")}
              title="Перемкнути тему"
            >
              {theme === "light" ? "🌙" : "☀️"}
            </button>
          </div>
          <p className="app-subtitle">
            Швидка та безпечна оптимізація ваших зображень без втрати якості
          </p>
        </header>

        {/* Error Notification */}
        {error && (
          <div className="error-box">
            <div className="error-icon">⚠️</div>
            <div className="error-message">{error}</div>
          </div>
        )}

        {/* 1. IDLE STATE */}
        {status === "idle" && !inputPath && (
          <div className="idle-state">
            <div className="welcome-graphic">
              <span className="graphic-icon">📂</span>
            </div>
            <button className="btn btn-primary" onClick={handleSelectFolder}>
              Вибрати папку
            </button>
            <p className="hint-text">Оберіть папку, що містить JPEG чи PNG файли</p>
          </div>
        )}

        {/* 2. SELECTED STATE (Ready to optimize) */}
        {status === "idle" && inputPath && (
          <div className="selected-state">
            <div className="path-group">
              <div className="path-row">
                <span className="path-label">Вхідна папка:</span>
                <span className="path-value">{inputPath}</span>
              </div>
              <div className="path-row">
                <span className="path-label">Папка результату:</span>
                <span className="path-value highlight">{outputPath}</span>
              </div>
            </div>

            {/* Options Settings Card */}
            <div className="settings-panel">
              <h3 className="settings-title">Налаштування обробки</h3>
              <div className="settings-options">
                <label className="setting-control">
                  <div className="setting-info">
                    <span className="setting-name">Конвертувати PNG в WebP</span>
                    <span className="setting-desc">Автоматично конвертує оригінальні PNG у WebP формат.</span>
                  </div>
                  <div className="toggle-container">
                    <input
                      type="checkbox"
                      id="convertPngToWebp"
                      className="toggle-checkbox"
                      checked={convertPngToWebp}
                      onChange={(e) => setConvertPngToWebp(e.target.checked)}
                    />
                    <label htmlFor="convertPngToWebp" className="toggle-label"></label>
                  </div>
                </label>

                <label className="setting-control">
                  <div className="setting-info">
                    <span className="setting-name">Оптимізувати SVG</span>
                    <span className="setting-desc">Видаляє коментарі, службові теги редакторів та зайві пробіли.</span>
                  </div>
                  <div className="toggle-container">
                    <input
                      type="checkbox"
                      id="optimizeSvg"
                      className="toggle-checkbox"
                      checked={optimizeSvg}
                      onChange={(e) => setOptimizeSvg(e.target.checked)}
                    />
                    <label htmlFor="optimizeSvg" className="toggle-label"></label>
                  </div>
                </label>

                <label className="setting-control">
                  <div className="setting-info">
                    <span className="setting-name">Оптимізувати WebP</span>
                    <span className="setting-desc">Стискає існуючі статичні WebP зображення без втрати якості.</span>
                  </div>
                  <div className="toggle-container">
                    <input
                      type="checkbox"
                      id="optimizeWebp"
                      className="toggle-checkbox"
                      checked={optimizeWebp}
                      onChange={(e) => setOptimizeWebp(e.target.checked)}
                    />
                    <label htmlFor="optimizeWebp" className="toggle-label"></label>
                  </div>
                </label>

                <div className="setting-divider"></div>

                {/* Quality Slider Control */}
                <div className="setting-control-column">
                  <div className="setting-info">
                    <div className="slider-header-row">
                      <span className="setting-name">Якість JPEG</span>
                      <span className="slider-badge">{jpegQuality}%</span>
                    </div>
                    <span className="setting-desc">Вкажіть бажану якість стиснення від 1% до 100%.</span>
                  </div>
                  <div className="slider-container">
                    <input
                      type="range"
                      min="1"
                      max="100"
                      value={jpegQuality}
                      onChange={(e) => setJpegQuality(Number(e.target.value))}
                      className="quality-slider"
                    />
                  </div>
                </div>

                <div className="setting-divider"></div>

                {/* Resizing Controls */}
                <div className="setting-control-column">
                  <label className="setting-control">
                    <div className="setting-info">
                      <span className="setting-name">Зменшувати великі зображення</span>
                      <span className="setting-desc">Зменшує роздільну здатність файлів пропорційно під задані ліміти.</span>
                    </div>
                    <div className="toggle-container">
                      <input
                        type="checkbox"
                        id="resizeImages"
                        className="toggle-checkbox"
                        checked={resizeImages}
                        onChange={(e) => setResizeImages(e.target.checked)}
                      />
                      <label htmlFor="resizeImages" className="toggle-label"></label>
                    </div>
                  </label>

                  {resizeImages && (
                    <div className="resize-inputs-row">
                      <div className="resize-input-group">
                        <label htmlFor="maxWidth" className="input-mini-label">Макс. ширина (px)</label>
                        <input
                          type="number"
                          id="maxWidth"
                          min="10"
                          max="99999"
                          value={maxWidth}
                          onChange={(e) => setMaxWidth(Math.max(10, Number(e.target.value)))}
                          className="number-input"
                        />
                      </div>
                      <div className="resize-input-group">
                        <label htmlFor="maxHeight" className="input-mini-label">Макс. висота (px)</label>
                        <input
                          type="number"
                          id="maxHeight"
                          min="10"
                          max="99999"
                          value={maxHeight}
                          onChange={(e) => setMaxHeight(Math.max(10, Number(e.target.value)))}
                          className="number-input"
                        />
                      </div>
                    </div>
                  )}
                </div>

                <div className="setting-divider"></div>

                {/* Video Conversion Section */}
                <div className="setting-control-column">
                  {ffmpegAvailable === false && (
                    <div className="ffmpeg-banner">
                      <span className="ffmpeg-banner-icon">⚠️</span>
                      <div className="ffmpeg-banner-text">
                        <span className="ffmpeg-banner-title">FFmpeg не знайдено</span>
                        <span className="ffmpeg-banner-desc">
                          Встановіть FFmpeg та перезапустіть застосунок, щоб увімкнути конвертацію відео.
                        </span>
                      </div>
                    </div>
                  )}
                  <label className="setting-control">
                    <div className="setting-info">
                      <span className="setting-name">Конвертувати VOB/AVI в MP4</span>
                      <span className="setting-desc">
                        Перекодує відеофайли у компактний MP4 (H.264 + AAC) через FFmpeg.
                        {ffmpegAvailable === false && " (потребує FFmpeg)"}
                      </span>
                    </div>
                    <div className="toggle-container">
                      <input
                        type="checkbox"
                        id="convertVideo"
                        className="toggle-checkbox"
                        checked={convertVideo}
                        disabled={ffmpegAvailable === false}
                        onChange={(e) => setConvertVideo(e.target.checked)}
                      />
                      <label
                        htmlFor="convertVideo"
                        className={`toggle-label${ffmpegAvailable === false ? " toggle-disabled" : ""}`}
                      ></label>
                    </div>
                  </label>

                  {convertVideo && ffmpegAvailable !== false && (
                    <div className="setting-control-column" style={{ marginTop: "8px" }}>
                      <div className="setting-info">
                        <div className="slider-header-row">
                          <span className="setting-name">Якість відео (CRF)</span>
                          <span className="slider-badge">{videoCrf}</span>
                        </div>
                        <span className="setting-desc">
                          0 = максимальна якість, 51 = мінімальна. Рекомендовано: 18–28.
                        </span>
                      </div>
                      <div className="slider-container">
                        <input
                          type="range"
                          min="0"
                          max="51"
                          value={videoCrf}
                          onChange={(e) => setVideoCrf(Number(e.target.value))}
                          className="quality-slider"
                        />
                      </div>
                    </div>
                  )}

                  <label className="setting-control" style={{ marginTop: "16px" }}>
                    <div className="setting-info">
                      <span className="setting-name">Оптимізувати MP4 (профіль YouTube)</span>
                      <span className="setting-desc">
                        Перекодовує MP4 файли для суттєвого зменшення розміру (CRF {videoCrf}, slow preset).
                        {ffmpegAvailable === false && " (потребує FFmpeg)"}
                      </span>
                    </div>
                    <div className="toggle-container">
                      <input
                        type="checkbox"
                        id="optimizeMp4"
                        className="toggle-checkbox"
                        checked={optimizeMp4}
                        disabled={ffmpegAvailable === false}
                        onChange={(e) => setOptimizeMp4(e.target.checked)}
                      />
                      <label
                        htmlFor="optimizeMp4"
                        className={`toggle-label${ffmpegAvailable === false ? " toggle-disabled" : ""}`}
                      ></label>
                    </div>
                  </label>

                  <label className="setting-control" style={{ marginTop: "16px" }}>
                    <div className="setting-info">
                      <span className="setting-name">Ультра-стиснення (H.265 / HEVC)</span>
                      <span className="setting-desc">
                        Дає вдвічі менший розмір файлу ніж H.264 при тій самій якості. Займає більше часу на кодування.
                        {ffmpegAvailable === false && " (потребує FFmpeg)"}
                      </span>
                    </div>
                    <div className="toggle-container">
                      <input
                        type="checkbox"
                        id="useH265"
                        className="toggle-checkbox"
                        checked={useH265}
                        disabled={ffmpegAvailable === false || (!convertVideo && !optimizeMp4)}
                        onChange={(e) => setUseH265(e.target.checked)}
                      />
                      <label
                        htmlFor="useH265"
                        className={`toggle-label${ffmpegAvailable === false || (!convertVideo && !optimizeMp4) ? " toggle-disabled" : ""}`}
                      ></label>
                    </div>
                  </label>

                  <label className="setting-control" style={{ marginTop: "16px" }}>
                    <div className="setting-info">
                      <span className="setting-name">Витягнути аудіо (MP3) з відео</span>
                      <span className="setting-desc">
                        Замість обробки відео буде витягнуто тільки звук і збережено як MP3 файл.
                        {ffmpegAvailable === false && " (потребує FFmpeg)"}
                      </span>
                    </div>
                    <div className="toggle-container">
                      <input
                        type="checkbox"
                        id="extractAudio"
                        className="toggle-checkbox"
                        checked={extractAudio}
                        disabled={ffmpegAvailable === false}
                        onChange={(e) => setExtractAudio(e.target.checked)}
                      />
                      <label
                        htmlFor="extractAudio"
                        className={`toggle-label${ffmpegAvailable === false ? " toggle-disabled" : ""}`}
                      ></label>
                    </div>
                  </label>

                  <label className="setting-control" style={{ marginTop: "16px" }}>
                    <div className="setting-info">
                      <span className="setting-name">Конвертувати WAV в MP3</span>
                      <span className="setting-desc">
                        Зменшує розмір аудіо за рахунок перекодування у формат MP3.
                        {ffmpegAvailable === false && " (потребує FFmpeg)"}
                      </span>
                    </div>
                    <div className="toggle-container">
                      <input
                        type="checkbox"
                        id="convertWavToMp3"
                        className="toggle-checkbox"
                        checked={convertWavToMp3}
                        disabled={ffmpegAvailable === false}
                        onChange={(e) => setConvertWavToMp3(e.target.checked)}
                      />
                      <label
                        htmlFor="convertWavToMp3"
                        className={`toggle-label${ffmpegAvailable === false ? " toggle-disabled" : ""}`}
                      ></label>
                    </div>
                  </label>

                  {convertWavToMp3 && ffmpegAvailable !== false && (
                    <div className="setting-control-column" style={{ marginTop: "8px" }}>
                      <div className="setting-info">
                        <div className="slider-header-row">
                          <span className="setting-name">Бітрейт MP3 (kbps)</span>
                          <span className="slider-badge">{mp3Bitrate}</span>
                        </div>
                        <span className="setting-desc">
                          Більше значення = краща якість і більший розмір файлу.
                        </span>
                      </div>
                      <div className="slider-container">
                        <input
                          type="range"
                          min="64"
                          max="320"
                          step="32"
                          value={mp3Bitrate}
                          onChange={(e) => setMp3Bitrate(Number(e.target.value))}
                          className="quality-slider"
                        />
                      </div>
                    </div>
                  )}

                  <label className="setting-control" style={{ marginTop: "16px" }}>
                    <div className="setting-info">
                      <span className="setting-name">Конвертувати GIF у MP4</span>
                      <span className="setting-desc">
                        Зменшує розмір анімацій у 5-10 разів за рахунок конвертації у тихе відео.
                        {ffmpegAvailable === false && " (потребує FFmpeg)"}
                      </span>
                    </div>
                    <div className="toggle-container">
                      <input
                        type="checkbox"
                        id="convertGifToMp4"
                        className="toggle-checkbox"
                        checked={convertGifToMp4}
                        disabled={ffmpegAvailable === false}
                        onChange={(e) => setConvertGifToMp4(e.target.checked)}
                      />
                      <label
                        htmlFor="convertGifToMp4"
                        className={`toggle-label${ffmpegAvailable === false ? " toggle-disabled" : ""}`}
                      ></label>
                    </div>
                  </label>
                </div>
              </div>
            </div>

            <div className="action-row">
              <button
                className="btn btn-secondary"
                onClick={handleSelectFolder}
                disabled={isBusy}
              >
                Змінити папку
              </button>
              <button
                className="btn btn-primary btn-glow"
                onClick={handleStartOptimization}
                disabled={isBusy}
              >
                Оптимізувати
              </button>
            </div>
          </div>
        )}

        {/* 3. SCANNING STATE */}
        {status === "scanning" && (
          <div className="scanning-state">
            <div className="spinner-container">
              <div className="pulse-spinner"></div>
            </div>
            <h2 className="state-title">Підготовка файлів...</h2>
            <p className="state-detail">Рекурсивне сканування папки та підрахунок розміру</p>
            
            <div className="action-row" style={{ marginTop: "24px" }}>
              <button
                className="btn btn-secondary btn-danger-hover"
                onClick={handleCancel}
                disabled={isCancelling}
              >
                {isCancelling ? "Скасування..." : "Скасувати"}
              </button>
            </div>
          </div>
        )}

        {/* 4. PROCESSING STATE */}
        {status === "processing" && (
          <div className="processing-state">
            <h2 className="state-title">Оптимізація файлів</h2>
            
            <div className="progress-stats">
              <span className="progress-counter">
                Оброблено: <strong>{progress.processedFiles}</strong> із <strong>{progress.totalFiles}</strong>
              </span>
              <span className="progress-percent">{Math.round(processedPercent)}%</span>
            </div>

            {/* Progress Bar */}
            <div className="progress-bar-container">
              <div
                className="progress-bar-fill"
                style={{ width: `${processedPercent}%` }}
              ></div>
            </div>

            {/* Current File */}
            {progress.currentFile && (
              <div className="current-file-box">
                <span className="current-file-label">Обробляється:</span>
                <span className="current-file-name" title={progress.currentFile}>
                  {progress.currentFile}
                </span>
                <div className="file-progress-container">
                  {progress.currentFileProgress != null ? (
                    <div 
                      className="file-progress-bar determinate" 
                      style={{ width: `${progress.currentFileProgress}%` }}
                    ></div>
                  ) : (
                    <div className="file-progress-bar indeterminate"></div>
                  )}
                </div>
                {progress.currentFileProgress != null && (
                  <div className="file-progress-text">
                    {Math.round(progress.currentFileProgress)}%
                  </div>
                )}
              </div>
            )}

            {/* Real-time counters */}
            <div className="stats-mini-grid">
              <div className="mini-card">
                <span className="mini-num text-success">{progress.optimizedFiles}</span>
                <span className="mini-label">Стиснуто</span>
              </div>
              <div className="mini-card">
                <span className="mini-num text-info">{progress.copiedFiles}</span>
                <span className="mini-label">Скопійовано</span>
              </div>
              <div className="mini-card">
                <span className="mini-num text-warning">{progress.originalKeptFiles}</span>
                <span className="mini-label">Без змін</span>
              </div>
              {progress.skippedFiles > 0 && (
                <div className="mini-card">
                  <span className="mini-num" style={{ color: "var(--text-muted)" }}>{progress.skippedFiles}</span>
                  <span className="mini-label">Пропущено</span>
                </div>
              )}
              {progress.failedFiles > 0 && (
                <div className="mini-card">
                  <span className="mini-num text-danger">{progress.failedFiles}</span>
                  <span className="mini-label">Помилки</span>
                </div>
              )}
            </div>

            <div className="action-row" style={{ marginTop: "24px" }}>
              <button
                className="btn btn-secondary btn-danger-hover"
                onClick={handleCancel}
                disabled={isCancelling}
              >
                {isCancelling ? "Скасування..." : "Скасувати"}
              </button>
            </div>
          </div>
        )}

        {/* 5. COMPLETED STATE */}
        {status === "completed" && (
          <div className="completed-state">
            <div className="success-header">
              <div className="success-badge">✓</div>
              <h2 className="state-title">Оптимізацію завершено!</h2>
            </div>

            {/* Size Savings Dashboard */}
            <div className="savings-dashboard">
              <div className="savings-main">
                <span className="savings-amount">{formatBytes(savedBytes)}</span>
                <span className="savings-label">Простору зекономлено</span>
              </div>
              {savedPercent > 0 && (
                <div className="savings-badge">-{savedPercent.toFixed(1)}%</div>
              )}
            </div>

            <div className="size-comparison-row">
              <div className="size-part">
                <span className="size-label">Початковий розмір</span>
                <span className="size-val">{formatBytes(progress.originalBytes)}</span>
              </div>
              <div className="size-divider">➔</div>
              <div className="size-part">
                <span className="size-label">Кінцевий розмір</span>
                <span className="size-val">{formatBytes(progress.outputBytes)}</span>
              </div>
            </div>

            {/* Detail stats grid */}
            <div className="stats-grid">
              <div className="stat-card">
                <span className="stat-value">{progress.totalFiles}</span>
                <span className="stat-label">Всього файлів</span>
              </div>
              <div className="stat-card">
                <span className="stat-value text-success">{progress.optimizedFiles}</span>
                <span className="stat-label">Оптимізовано</span>
              </div>
              <div className="stat-card">
                <span className="stat-value text-info">{progress.copiedFiles}</span>
                <span className="stat-label">Скопійовано</span>
              </div>
              <div className="stat-card">
                <span className="stat-value text-warning">{progress.originalKeptFiles}</span>
                <span className="stat-label">Залишено без змін</span>
              </div>
              {progress.skippedFiles > 0 && (
                <div className="stat-card">
                  <span className="stat-value" style={{ color: "var(--text-muted)" }}>{progress.skippedFiles}</span>
                  <span className="stat-label">Пропущено</span>
                </div>
              )}
              {progress.failedFiles > 0 && (
                <div className="stat-card error-border">
                  <span className="stat-value text-danger">{progress.failedFiles}</span>
                  <span className="stat-label">Помилки</span>
                </div>
              )}
            </div>

            <div className="action-row font-large">
              <button className="btn btn-secondary" onClick={handleReset}>
                Обрати іншу папку
              </button>
              <button className="btn btn-primary btn-glow" onClick={handleOpenFolder}>
                Відкрити результат
              </button>
            </div>
          </div>
        )}

        {/* 6. CANCELLED STATE */}
        {status === "cancelled" && (
          <div className="cancelled-state">
            <div className="warning-header">
              <div className="warning-badge">!</div>
              <h2 className="state-title">Оптимізацію скасовано</h2>
            </div>
            
            <p className="app-subtitle" style={{ marginBottom: "24px", textAlign: "center" }}>
              Обробку файлів було зупинено. Результати обробки до моменту скасування:
            </p>

            {/* Size Savings Dashboard */}
            <div className="savings-dashboard warning-bg">
              <div className="savings-main">
                <span className="savings-amount">{formatBytes(savedBytes)}</span>
                <span className="savings-label">Простору зекономлено</span>
              </div>
              {savedPercent > 0 && (
                <div className="savings-badge">-{savedPercent.toFixed(1)}%</div>
              )}
            </div>

            <div className="size-comparison-row">
              <div className="size-part">
                <span className="size-label">Початковий розмір</span>
                <span className="size-val">{formatBytes(progress.originalBytes)}</span>
              </div>
              <div className="size-divider">➔</div>
              <div className="size-part">
                <span className="size-label">Поточний розмір</span>
                <span className="size-val">{formatBytes(progress.outputBytes)}</span>
              </div>
            </div>

            {/* Detail stats grid */}
            <div className="stats-grid">
              <div className="stat-card">
                <span className="stat-value">{progress.processedFiles} / {progress.totalFiles}</span>
                <span className="stat-label">Оброблено файлів</span>
              </div>
              <div className="stat-card">
                <span className="stat-value text-success">{progress.optimizedFiles}</span>
                <span className="stat-label">Оптимізовано</span>
              </div>
              <div className="stat-card">
                <span className="stat-value text-info">{progress.copiedFiles}</span>
                <span className="stat-label">Скопійовано</span>
              </div>
              <div className="stat-card">
                <span className="stat-value text-warning">{progress.originalKeptFiles}</span>
                <span className="stat-label">Без змін</span>
              </div>
            </div>

            <div className="action-row font-large">
              <button className="btn btn-secondary" onClick={handleReset}>
                Обрати іншу папку
              </button>
              <button className="btn btn-primary btn-glow" onClick={handleOpenFolder}>
                Відкрити результат
              </button>
            </div>
          </div>
        )}

        {/* 7. FAILED STATE */}
        {status === "failed" && (
          <div className="failed-state">
            <div className="failed-header">
              <div className="failed-badge">❌</div>
              <h2 className="state-title">Помилка виконання job</h2>
            </div>
            <p className="hint-text">Перевірте правильність вказаних шляхів та спробуйте знову</p>
            <button className="btn btn-secondary" onClick={handleReset}>
              Назад
            </button>
          </div>
        )}
        {/* Author Footer (Section 7) */}
        <button
          onClick={handleOpenAuthor}
          title={`More projects and services by ${PRODUCT_METADATA.author}`}
          className="author-nav-btn"
        >
          <span className="author-sparkle">✨</span>
          <span>More by {PRODUCT_METADATA.author}</span>
          <span className="author-arrow">↗</span>
        </button>
      </div>
    </div>
  );
}

export default App;
