const BYTE_UNITS = ["B", "KB", "MB", "GB", "TB"] as const;

export function formatBytes(bytes: number): string {
    if (!bytes) {
        return "0 B";
    }
    let value = bytes;
    let unitIndex = 0;
    while (value >= 1024 && unitIndex < BYTE_UNITS.length - 1) {
        value /= 1024;
        unitIndex += 1;
    }
    const precision = unitIndex === 0 ? 0 : 1;
    return `${Number(value.toFixed(precision))} ${BYTE_UNITS[unitIndex]}`;
}

export function formatPercent(value: number): string {
    return `${Number(value.toFixed(1))}%`;
}

export function formatDuration(durationMs?: number | null): string {
    if (durationMs == null) {
        return "-";
    }
    return `${durationMs} ms`;
}
