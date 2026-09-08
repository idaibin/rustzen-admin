import { expect, test } from "bun:test";

import { safeSubjectDestination, safeSubjectHref } from "./subject-navigation";

const item = {
    producer: "monitor",
    topic: "monitor.incident.opened",
    subjectKind: "monitor-incident",
    subjectId: "a/b?url=https://evil",
} as Notifications.Item;
test("subject navigation uses only the fixed producer/topic/kind mapping", () => {
    expect(safeSubjectHref(item)).toBe(
        "/monitoring/incidents?incidentId=a%2Fb%3Furl%3Dhttps%3A%2F%2Fevil",
    );
    expect(safeSubjectDestination(item)).toEqual({
        to: "/monitoring/incidents",
        search: { incidentId: "a/b?url=https://evil" },
    });
    expect(safeSubjectHref({ ...item, topic: "monitor.incident.unknown" })).toBeUndefined();
    expect(
        safeSubjectHref({
            ...item,
            producer: "reports",
            topic: "reports.run.failed",
            subjectKind: "reports-run",
            subjectId: "run-1",
        }),
    ).toBe("/reports/runs?runId=run-1");
});
