declare namespace Monitor {
    type IncidentKind = "cpuHigh" | "memoryHigh" | "diskHigh" | "nodeOffline";
    type IncidentStatus = "active" | "resolved";

    interface Usage {
        usedBytes: number;
        totalBytes: number;
        usagePercent: number;
    }

    interface DiskUsage extends Usage {
        mountPoint: string;
        collectedAt: string;
    }

    interface LatestDiskUsage extends Usage {
        mountPoint: string;
    }

    interface Overview {
        registeredNodes: number;
        onlineNodes: number;
        offlineNodes: number;
        activeIncidents: number;
        latestResource: {
            nodeId: string;
            collectedAt: string;
            lastReceivedAt: string;
            cpuPercent: number;
            memoryPercent: number;
            disks: LatestDiskUsage[];
        } | null;
    }

    interface Node {
        nodeId: string;
        hostname: string;
        agentVersion: string;
        bootId: string;
        sequence: number;
        lastReportAt: string;
        lastReceivedAt: string;
        status: "online" | "offline";
        alertPolicySource: "global" | "custom";
        cpuPercent: number;
        memory: Usage;
        disks: DiskUsage[];
        createdAt: string;
        updatedAt: string;
    }

    interface MetricPoint {
        collectedAt: string;
        cpuPercent: number;
        memoryPercent: number;
    }

    interface DiskMetricSeries {
        mountPoint: string;
        points: Array<{ collectedAt: string; percent: number }>;
    }

    interface Metrics {
        bucket: "raw" | "5m" | "1h";
        points: MetricPoint[];
        disks: DiskMetricSeries[];
    }

    interface MetricsQuery {
        from?: string;
        to?: string;
        bucket?: "raw" | "5m" | "1h";
    }

    interface IncidentSummary {
        id: string;
        nodeId: string;
        kind: IncidentKind;
        target: string;
        status: IncidentStatus;
        title: string;
        thresholdPercent: number | null;
        observedPercent: number | null;
        openedAt: string;
        lastObservedAt: string;
        resolvedAt: string | null;
        resolutionReason: string | null;
        details: Record<string, unknown>;
    }

    interface IncidentDetail extends IncidentSummary {
        node: Pick<Node, "nodeId" | "hostname" | "agentVersion" | "lastReportAt">;
    }

    interface IncidentQuery {
        current?: number;
        pageSize?: number;
        status?: IncidentStatus;
        kind?: IncidentKind;
        nodeId?: string;
        from?: string;
        to?: string;
    }

    interface AlertThreshold {
        enabled: boolean;
        thresholdPercent: number;
    }

    interface AlertSettings {
        cpu: AlertThreshold;
        memory: AlertThreshold;
        disk: AlertThreshold;
        offline: { enabled: boolean; afterSeconds: number };
        updatedAt: string;
        source: "global" | "custom";
        isCustom: boolean;
    }

    type UpdateAlertSettings = Omit<AlertSettings, "updatedAt" | "source" | "isCustom">;

    interface SummaryRange {
        min: number | null;
        avg: number | null;
        max: number | null;
    }

    interface DailySummary {
        nodeId: string;
        date: string;
        sampleCount: number;
        coverage: number;
        coveragePercent: number;
        cpu: SummaryRange;
        memory: SummaryRange;
        diskSummary: Record<string, SummaryRange>;
        offlineSeconds: number;
        incidentCount: number;
    }

    interface DailySummaryQuery {
        current?: number;
        pageSize?: number;
        nodeId?: string;
        from?: string;
        to?: string;
    }

    interface Page<T> {
        data: T[];
        total: number;
        success: boolean;
    }
}
