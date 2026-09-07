import { Database } from "bun:sqlite";
import { readdir, readFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { expect, test } from "bun:test";
import { resolveSelection } from "./resolver.ts";

const repositoryRoot = resolve(import.meta.dir, "..");
const notificationTables = [
    "notification_accounting",
    "notification_receipts",
    "notification_recipients",
    "notification_user_state",
    "notifications",
];

async function freshInventory(...migrationPaths: string[]) {
    const db = new Database(":memory:");
    try {
        db.exec("PRAGMA foreign_keys = ON");
        for (const path of migrationPaths)
            db.exec(await readFile(resolve(repositoryRoot, path), "utf8"));
        return db
            .query<{ name: string }, []>(
                "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name",
            )
            .all()
            .map(({ name }) => name);
    } finally {
        db.close();
    }
}

test("fresh compositions select the optional inbox owner exactly", async () => {
    const monitorAdmin = await freshInventory(
        "apps/admin/migrations/sqlite-monitor/0001_init.sql",
    );
    const fullAdmin = await freshInventory("apps/admin/migrations/sqlite/0001_init.sql");
    const monitorNotifyAdmin = await freshInventory(
        "apps/admin/migrations/sqlite-monitor/0001_init.sql",
        "apps/admin/migrations/sqlite-notifications/0001_notifications.sql",
    );
    const monitor = await freshInventory("apps/monitor/migrations/0001_init.sql");
    const monitorNotify = await freshInventory(
        "apps/monitor/migrations/0001_init.sql",
        "apps/monitor/migrations-notifications/0001_notification_outbox.sql",
    );
    const reports = await freshInventory("apps/reports/migrations/0001_init.sql");

    expect(monitorAdmin.filter((name) => name.startsWith("notification"))).toEqual([]);
    expect(monitor.filter((name) => name.startsWith("notification"))).toEqual([]);
    expect(monitorNotify.filter((name) => name.startsWith("notification"))).toEqual([
        "notification_delivery_status",
        "notification_outbox",
    ]);
    expect(reports.filter((name) => name.startsWith("notification"))).toEqual([]);
    expect(fullAdmin.filter((name) => name.startsWith("notification"))).toEqual(
        notificationTables,
    );
    expect(monitorNotifyAdmin.filter((name) => name.startsWith("notification"))).toEqual(
        notificationTables,
    );
    expect(resolveSelection({ preset: "monitor" }).schemaOwners).toEqual([
        "admin",
        "monitor",
    ]);
    expect(resolveSelection({ preset: "reports" }).schemaOwners).toEqual([
        "admin",
        "reports",
    ]);
    expect(resolveSelection({ preset: "monitor-notify" }).schemaOwners).toEqual([
        "admin",
        "admin-notifications",
        "monitor",
        "monitor-notifications",
    ]);
    const fullSql = await readFile(
        resolve(repositoryRoot, "apps/admin/migrations/sqlite/0001_init.sql"),
        "utf8",
    );
    const fragmentSql = await readFile(
        resolve(
            repositoryRoot,
            "apps/admin/migrations/sqlite-notifications/0001_notifications.sql",
        ),
        "utf8",
    );
    expect(fullSql.endsWith(fragmentSql)).toBeTrue();
});

test("reports source contains no inbox API, queue or retry owner", async () => {
    const roots = [
        "apps/reports/src",
        "apps/reports/migrations",
    ];
    const forbidden = [
        "/api/notifications",
        "inbox.changed",
        "notification_events",
        "notification_outbox",
        "notification_relay",
        "notification_retry",
    ];
    for (const root of roots) {
        const text = (
            await Promise.all(
                (await sourceFiles(resolve(repositoryRoot, root))).map((file) =>
                    Bun.file(file).text(),
                ),
            )
        ).join("\n");
        for (const marker of forbidden) expect(text).not.toContain(marker);
    }
});

async function sourceFiles(root: string): Promise<string[]> {
    const entries = await readdir(root, { withFileTypes: true });
    const paths = await Promise.all(
        entries.map((entry) => {
            const path = join(root, entry.name);
            return entry.isDirectory() ? sourceFiles(path) : Promise.resolve([path]);
        }),
    );
    return paths.flat();
}
