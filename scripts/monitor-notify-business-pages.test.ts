import { expect, test } from "bun:test";

import { businessJourneyItems, inboxPage, incidentPage } from "./monitor-notify-business-pages.ts";

test("P8f-B accepts pending empty pages but requires the eventual matching incident and message", () => {
    const incidents = incidentPage({ data: [], success: true, total: 0 });
    const inbox = inboxPage({ items: [], nextCursor: null, retentionDays: 30, revision: 0, snapshot: "pending" });
    expect(businessJourneyItems(incidents, inbox, "p8fb-witness-node")).toEqual({ incident: undefined, message: undefined });
    const eventualIncidents = incidentPage({ data: [{ id: "incident", kind: "cpuHigh", nodeId: "p8fb-witness-node", title: "CPU high" }], success: true, total: 1 });
    const eventualInbox = inboxPage({ items: [{ id: "message", subjectId: "incident", producer: "monitor", topic: "monitor.incident.opened", title: "CPU high" }], nextCursor: null, retentionDays: 30, revision: 1, snapshot: "eventual" });
    expect(businessJourneyItems(eventualIncidents, eventualInbox, "p8fb-witness-node")).toMatchObject({ incident: { id: "incident" }, message: { id: "message" } });
});
