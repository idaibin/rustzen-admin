/** Canonical bytewise path order; never depend on the host locale. */
export const compareContainerExportPath = (left: string, right: string): number =>
    left < right ? -1 : left > right ? 1 : 0;
