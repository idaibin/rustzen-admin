//! Per-preset selected-server descriptor shared by the activation path.

pub(super) struct ServerSelection {
    /// Second service of the pair: "monitor" or "insights".
    pub(super) secondary: &'static str,
    /// Notification delivery is selected (monitor-notify only).
    pub(super) notify: bool,
    pub(super) units: [&'static str; 3],
    pub(super) binaries: [&'static str; 2],
    pub(super) services: [&'static str; 2],
}

impl ServerSelection {
    pub(super) fn secondary_binary(&self) -> &'static str {
        match self.secondary {
            "monitor" => "rz-monitor",
            "insights" => "rz-insights",
            _ => unreachable!("validated secondary service"),
        }
    }
    pub(super) fn secondary_config(&self) -> String {
        format!("rz-{}.env", self.secondary)
    }
    pub(super) fn secondary_runtime_root(&self) -> String {
        format!("/var/lib/rustzen-{}", self.secondary)
    }
    pub(super) fn secondary_database(&self) -> String {
        format!("{}.db", self.secondary)
    }
    pub(super) fn secondary_port_key(&self) -> &'static str {
        match self.secondary {
            "monitor" => "RUSTZEN_MONITOR_PORT",
            "insights" => "RUSTZEN_INSIGHTS_PORT",
            _ => unreachable!("validated secondary service"),
        }
    }
    /// Mirrors the service binary's built-in default when the port key is omitted.
    pub(super) fn secondary_port_default(&self) -> u16 {
        match self.secondary {
            "monitor" => 9802,
            "insights" => 9803,
            _ => unreachable!("validated secondary service"),
        }
    }
    pub(super) fn secondary_sqlite_key(&self) -> &'static str {
        match self.secondary {
            "monitor" => "RUSTZEN_MONITOR_SQLITE_PATH",
            "insights" => "RUSTZEN_INSIGHTS_SQLITE_PATH",
            _ => unreachable!("validated secondary service"),
        }
    }
    pub(super) fn excluded_services(&self) -> [&'static str; 2] {
        match self.secondary {
            "monitor" => ["rz-insights", "rz-reports"],
            "insights" => ["rz-monitor", "rz-reports"],
            _ => unreachable!("validated secondary service"),
        }
    }
}

pub(super) fn for_preset(preset: &str) -> Result<ServerSelection, String> {
    match preset {
        "monitor" | "monitor-notify" => Ok(ServerSelection {
            secondary: "monitor",
            notify: preset == "monitor-notify",
            units: ["rz-admin.service", "rz-monitor.service", "rz.target"],
            binaries: ["rz-admin", "rz-monitor"],
            services: ["admin", "monitor"],
        }),
        "analytics" => Ok(ServerSelection {
            secondary: "insights",
            notify: false,
            units: ["rz-admin.service", "rz-insights.service", "rz.target"],
            binaries: ["rz-admin", "rz-insights"],
            services: ["admin", "insights"],
        }),
        _ => Err("selected server preset is invalid".into()),
    }
}
