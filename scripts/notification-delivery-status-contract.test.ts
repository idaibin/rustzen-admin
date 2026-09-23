import { expect, test } from "bun:test";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const root = resolve(import.meta.dir, "..");
const fields = [
    "id",
    "pending_count",
    "pending_bytes",
    "quarantine_count",
    "quarantine_bytes",
    "omitted_count",
    "expired_count",
    "unconfirmed_count",
    "quarantined_count",
    "quarantine_evicted_count",
    "first_gap_at",
    "last_gap_at",
    "last_success_at",
];

test("delivery-status documentation, producer migrations, and API rows share one field order", async () => {
    const [contract, monitorSql, reportsSql, monitorStatus, reportsStatus] =
        await Promise.all([
            text("docs/product/features/composable-distribution/contracts.md"),
            text(
                "apps/monitor/migrations-notifications/0001_notification_outbox.sql",
            ),
            text(
                "apps/reports/migrations-notifications/0001_notification_outbox.sql",
            ),
            text("apps/monitor/src/features/monitoring/delivery.rs"),
            text("apps/reports/src/features/automation/delivery.rs"),
        ]);

    expect(documentedFields(contract)).toEqual(fields);
    expect(sqlFields(monitorSql)).toEqual(fields);
    expect(sqlFields(reportsSql)).toEqual(fields);
    expect(statusFields(monitorStatus)).toEqual(fields.slice(1));
    expect(statusFields(reportsStatus)).toEqual(fields.slice(1));
});

async function text(path: string) {
    return readFile(resolve(root, path), "utf8");
}

function documentedFields(source: string) {
    const block = source.match(/notification_delivery_status\n([\s\S]*?)\n```/);
    if (!block)
        throw new Error(
            "notification_delivery_status documentation is missing",
        );
    return block[1]
        .trim()
        .split("\n")
        .map((line) => line.trim().split(" = ", 1)[0]);
}

function sqlFields(source: string) {
    const table = source.match(
        /CREATE TABLE notification_delivery_status \(([\s\S]*?)\n\);/,
    );
    if (!table)
        throw new Error("notification_delivery_status table is missing");
    return [...table[1].matchAll(/^\s{4}([a-z_]+)\s+/gm)].map(
        (match) => match[1],
    );
}

function statusFields(source: string) {
    const status = source.match(/struct DeliveryStatus \{([\s\S]*?)\n\}/);
    if (!status) throw new Error("DeliveryStatus is missing");
    return [...status[1].matchAll(/^\s{4}([a-z_]+):/gm)].map(
        (match) => match[1],
    );
}
