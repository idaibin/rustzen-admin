const MONITOR_TOPICS = new Set(["monitor.incident.opened", "monitor.incident.resolved"]);
const REPORT_TOPICS = new Set([
    "reports.run.completed",
    "reports.run.failed",
    "reports.run.cancelled",
]);
const REPORTS_SELECTED = import.meta.env.VITE_RUSTZEN_REPORTS_SELECTED !== "false";

export type SubjectDestination =
    | { to: "/monitoring/incidents"; search: { incidentId: string } }
    | { to: "/reports/runs"; search: { runId: string } };

export const safeSubjectDestination = (
    item: Notifications.Item,
): SubjectDestination | undefined => {
    if (
        item.producer === "monitor" &&
        item.subjectKind === "monitor-incident" &&
        MONITOR_TOPICS.has(item.topic)
    )
        return { to: "/monitoring/incidents", search: { incidentId: item.subjectId } };
    if (
        REPORTS_SELECTED &&
        item.producer === "reports" &&
        item.subjectKind === "reports-run" &&
        REPORT_TOPICS.has(item.topic)
    )
        return { to: "/reports/runs", search: { runId: item.subjectId } };
    return undefined;
};

export const safeSubjectHref = (item: Notifications.Item): string | undefined => {
    const destination = safeSubjectDestination(item);
    if (!destination) return undefined;
    const value = Object.values(destination.search)[0];
    const key = destination.to === "/monitoring/incidents" ? "incidentId" : "runId";
    return `${destination.to}?${key}=${encodeURIComponent(value)}`;
};
