const UNITS = ["B", "KB", "MB", "GB", "TB"];

export function formatBytes(bytes: number | null | undefined): string {
    if (bytes == null) return "—";
    if (bytes <= 0) return "0 B";
    const i = Math.min(Math.floor(Math.log2(bytes) / 10), UNITS.length - 1);
    const value = bytes / 1024 ** i;
    return `${i === 0 ? value : value.toFixed(2)} ${UNITS[i]}`;
}
