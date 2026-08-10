import {
    backupModuleLogs,
    confirmModuleLogCleanup,
    listModuleLogs,
    previewModuleLogCleanup,
    tailModuleLog,
    type ModuleLogCleanupPreviewResp,
    type ModuleLogCleanupResultResp,
    type ModuleLogFileResp,
    type ModuleLogFileSelector,
    type ModuleLogItemFailure,
    type ModuleLogTailResp,
} from "@/api/generated/admin-contract";

export const MODULE_LOG_MODULES = ["admin", "monitor", "insights", "reports"] as const;

export type ModuleLogModule = (typeof MODULE_LOG_MODULES)[number];

export type ModuleLogFile = ModuleLogFileResp;
export type ModuleLogTail = ModuleLogTailResp;
export type ModuleLogCleanupPreview = ModuleLogCleanupPreviewResp;
export type ModuleLogCleanupResult = ModuleLogCleanupResultResp;
export type ModuleLogFailure = ModuleLogItemFailure;

export interface ModuleLogListParams {
    module?: ModuleLogModule;
    date?: string;
}

const normalizeModule = (module: string): ModuleLogModule => {
    if ((MODULE_LOG_MODULES as readonly string[]).includes(module)) {
        return module as ModuleLogModule;
    }
    throw new Error(`Unsupported module log module: ${module}`);
};

const normalizeSelector = (selector: ModuleLogFileSelector): ModuleLogFileSelector => ({
    module: normalizeModule(selector.module),
    date: selector.date,
});

const downloadBlob = (blob: Blob, filename: string): string => {
    const downloadUrl = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = downloadUrl;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    URL.revokeObjectURL(downloadUrl);
    document.body.removeChild(link);
    return filename;
};

export const moduleLogAPI = {
    list: async (params?: ModuleLogListParams): Promise<ModuleLogFile[]> => {
        const response = await listModuleLogs(params);
        return response.data;
    },

    tail: async (
        selector: ModuleLogFileSelector,
        cursor?: string | null,
    ): Promise<ModuleLogTail> => {
        const normalized = normalizeSelector(selector);
        const response = await tailModuleLog({
            module: normalized.module,
            date: normalized.date,
            cursor: cursor ?? undefined,
        });
        return response.data;
    },

    backup: async (files: ModuleLogFileSelector[]): Promise<string> => {
        if (files.length === 0) {
            throw new Error("Select at least one module log file.");
        }
        const normalizedFiles = files.map(normalizeSelector);
        const archive = await backupModuleLogs({ files: normalizedFiles });
        return downloadBlob(archive, "rustzen-module-logs.tar");
    },

    previewCleanup: async (): Promise<ModuleLogCleanupPreview> => {
        const response = await previewModuleLogCleanup();
        return response.data;
    },

    confirmCleanup: async (token: string): Promise<ModuleLogCleanupResult> => {
        if (!token.trim()) {
            return Promise.reject(new Error("Cleanup confirmation token is required."));
        }
        const response = await confirmModuleLogCleanup({ token });
        return response.data;
    },
};
