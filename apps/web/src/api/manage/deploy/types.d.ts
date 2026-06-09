declare namespace Deploy {
    type Component = "server" | "web";

    interface Item {
        id: number;
        component: Component;
        version: string;
        arch: string;
        filePath: string;
        fileSize: number;
        fileHash: string;
        isCurrent: boolean;
        isDeployed: boolean;
        isExpired: boolean;
        deployedAt?: string | null;
        expiredAt?: string | null;
        deletedAt?: string | null;
        deployedBy?: string | null;
        notes?: string | null;
        createdAt: string;
        updatedAt: string;
    }

    interface ListParams {
        current?: number;
        pageSize?: number;
        component?: Component;
        isCurrent?: boolean;
        isDeployed?: boolean;
        isExpired?: boolean;
        search?: string;
    }

    interface UploadForm {
        component: Component;
        version: string;
        arch?: string;
        notes?: string;
        file: File;
    }

    interface DeployRequest {
        versionId: number;
        deployedBy?: string | null;
    }

    interface ExpireRequest {
        notes?: string | null;
    }
}
