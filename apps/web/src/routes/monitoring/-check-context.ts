export type CheckIncidentFilter = {
    sourceType: "check";
    sourceId: string;
};

export function checkIncidentFilter(checkId: string): CheckIncidentFilter {
    return { sourceType: "check", sourceId: checkId };
}

export function checkIncidentHref(checkId: string): string {
    const filter = checkIncidentFilter(checkId);
    const params = new URLSearchParams(filter);
    return `/monitoring/incidents?${params.toString()}`;
}
