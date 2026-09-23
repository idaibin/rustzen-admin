import { canonicalJson } from "../distribution/release-manifest-core.ts";

const object = (value: unknown): value is Record<string, any> => Boolean(value) && typeof value === "object" && !Array.isArray(value);
const exact = (value: unknown, keys: string[]) => object(value) && canonicalJson(Object.keys(value).sort()) === canonicalJson(keys);

export const incidentPage = (value: unknown) => {
    const page = value as any;
    if (!exact(page, ["data", "success", "total"]) || !Array.isArray(page.data) || !Number.isSafeInteger(page.total) || page.total < 0 || page.success !== true) throw Error("incident page shape differs");
    return page;
};

export const inboxPage = (value: unknown) => {
    const page = value as any;
    if (!exact(page, ["items", "nextCursor", "retentionDays", "revision", "snapshot"]) || !Array.isArray(page.items) || typeof page.snapshot !== "string" || !page.snapshot || !Number.isSafeInteger(page.revision) || page.revision < 0 || !Number.isSafeInteger(page.retentionDays) || page.retentionDays < 1 || (page.nextCursor !== null && typeof page.nextCursor !== "string")) throw Error("inbox page shape differs");
    return page;
};

export const businessJourneyItems = (incidents: any, inbox: any, nodeId: string) => {
    const incident = incidents.data.find((item: any) => item.kind === "cpuHigh" && item.nodeId === nodeId);
    const message = inbox.items.find((item: any) => item.subjectId === incident?.id && item.producer === "monitor" && item.topic === "monitor.incident.opened");
    return { incident, message };
};
