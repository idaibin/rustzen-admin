import {
    confirmModuleLogCleanup,
    getBackupModuleLogsUrl,
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
import { generatedBlobResponse } from "@/api/request";

export const MODULE_LOG_MODULES = ["admin", "monitor", "insights", "reports"] as const;

export type ModuleLogModule = (typeof MODULE_LOG_MODULES)[number];

export type ModuleLogFile = ModuleLogFileResp;
export type ModuleLogTail = ModuleLogTailResp;
export type ModuleLogCleanupPreview = ModuleLogCleanupPreviewResp;
export type ModuleLogCleanupResult = ModuleLogCleanupResultResp;
export type ModuleLogFailure = ModuleLogItemFailure;

export interface ModuleLogBackup {
    filename: string;
    fileCount: number;
    archiveSha256: string;
}

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

const downloadBlob = (blob: Blob, filename: string): void => {
    const downloadUrl = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = downloadUrl;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    URL.revokeObjectURL(downloadUrl);
    document.body.removeChild(link);
};

const archiveMetadata = (headers: Headers): Omit<ModuleLogBackup, "blob"> => {
    const contentDisposition = headers.get("content-disposition");
    const filename = contentDisposition?.match(
        /^attachment;\s*filename=([A-Za-z0-9][A-Za-z0-9._-]*)$/i,
    )?.[1];
    if (!filename) {
        throw new Error("Module log backup is missing a valid Content-Disposition filename.");
    }

    const archiveSha256 = headers.get("x-rustzen-archive-sha256");
    if (!archiveSha256 || !/^[a-f0-9]{64}$/.test(archiveSha256)) {
        throw new Error("Module log backup is missing a valid X-RustZen-Archive-SHA256 header.");
    }

    const fileCountValue = headers.get("x-rustzen-archive-file-count");
    if (!fileCountValue || !/^[1-9][0-9]*$/.test(fileCountValue)) {
        throw new Error(
            "Module log backup is missing a valid X-RustZen-Archive-File-Count header.",
        );
    }
    const fileCount = Number(fileCountValue);
    if (!Number.isSafeInteger(fileCount)) {
        throw new Error("Module log backup has an invalid X-RustZen-Archive-File-Count header.");
    }

    return { filename, fileCount, archiveSha256 };
};

const sha256 = async (blob: Blob): Promise<string> => {
    const digest = await crypto.subtle.digest("SHA-256", await blob.arrayBuffer());
    return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0")).join(
        "",
    );
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

    backup: async (files: ModuleLogFileSelector[]): Promise<ModuleLogBackup> => {
        if (files.length === 0) {
            throw new Error("Select at least one module log file.");
        }
        const normalizedFiles = Array.from(
            new Map(
                files.map((selector) => {
                    const normalized = normalizeSelector(selector);
                    return [`${normalized.module}\u0000${normalized.date}`, normalized];
                }),
            ).values(),
        );
        const response = await generatedBlobResponse(getBackupModuleLogsUrl(), {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ files: normalizedFiles }),
        });
        const metadata = archiveMetadata(response.headers);
        if (metadata.fileCount !== normalizedFiles.length) {
            throw new Error("Module log backup file count does not match the selected files.");
        }
        if ((await sha256(response.blob)) !== metadata.archiveSha256) {
            throw new Error("Module log backup SHA-256 does not match X-RustZen-Archive-SHA256.");
        }
        downloadBlob(response.blob, metadata.filename);
        return metadata;
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
