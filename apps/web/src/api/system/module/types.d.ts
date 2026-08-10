declare namespace SystemModule {
    type Id = "monitor" | "insights" | "reports";
    type RoutePath =
        | "/monitoring/overview"
        | "/monitoring/nodes"
        | "/monitoring/checks"
        | "/monitoring/incidents"
        | "/analytics/overview"
        | "/analytics/details"
        | "/reports/templates"
        | "/reports/runs";
    type Icon = "monitor" | "chart-no-axes-combined" | "file-text";

    interface Item {
        id: Id;
        name: string;
        enabled: boolean;
        available: boolean;
        compatible: boolean;
        releaseVersion: string | null;
        lastSeenAt: string | null;
        error: string | null;
    }

    interface NavigationItem {
        module: Id;
        moduleName: string;
        code: string;
        title: string;
        path: RoutePath;
        icon: Icon;
        sortOrder: number;
        permission: string;
    }

    interface UpdateRequest {
        enabled: boolean;
    }
}
