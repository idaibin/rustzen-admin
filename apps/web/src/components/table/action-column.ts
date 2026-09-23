/**
 * Shared fixed-right action column sizing: one slot per icon button, floored
 * at the width of the two-character "操作" header.
 */
export const actionColumnWidth = (iconCount: number) => Math.max(64, iconCount * 36 + 28);
