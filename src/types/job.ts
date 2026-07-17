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
  failedFiles: number;
  originalBytes: number;
  outputBytes: number;
}

export interface FolderSelection {
  inputPath: string;
  outputPath: string;
}
