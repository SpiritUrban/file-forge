export type JobStatus =
  | "idle"
  | "scanning"
  | "processing"
  | "completed"
  | "failed"
  | "cancelled";

export interface JobProgress {
  status: JobStatus;
  totalFiles: number;
  processedFiles: number;
  currentFile: string | null;
  optimizedFiles: number;
  copiedFiles: number;
  originalKeptFiles: number;
  skippedFiles: number;
  failedFiles: number;
  originalBytes: number;
  outputBytes: number;
  currentFileProgress?: number | null;
}

export interface FolderSelection {
  inputPath: string;
  outputPath: string;
}

export interface JobOptions {
  convertPngToWebp: boolean;
  optimizeSvg: boolean;
  optimizeWebp: boolean;
  jpegQuality: number;
  resizeImages: boolean;
  maxWidth: number;
  maxHeight: number;
  convertVideo: boolean;
  videoCrf: number;
  useH265: boolean;
  extractAudio: boolean;
  convertWavToMp3: boolean;
  mp3Bitrate: number;
  optimizeMp4: boolean;
  convertGifToMp4: boolean;
}
