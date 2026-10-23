/**
 * Formatting and Utility Helpers for Digital Vigital Launcher.
 */

export function formatBytes(bytes: number, decimals = 1): string {
  if (!bytes || bytes <= 0) return "0 B";
  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
}

export function formatDownloads(count: number): string {
  if (!count || count <= 0) return "0";
  if (count >= 1_000_000) {
    return `${(count / 1_000_000).toFixed(1)}M`;
  }
  if (count >= 1_000) {
    return `${(count / 1_000).toFixed(0)}k`;
  }
  return count.toString();
}

export function formatISODate(dateStr: string): string {
  if (!dateStr) return "";
  try {
    return dateStr.split("T")[0];
  } catch {
    return dateStr;
  }
}
