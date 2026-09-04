import { describe, expect, test } from "bun:test";

const route = await Bun.file(
    new URL("../src/routes/manage/deploy.tsx", import.meta.url),
).text();
const dialogs = await Bun.file(
    new URL("../src/routes/manage/-deploy-dialogs.tsx", import.meta.url),
).text();

describe("deployment route composition", () => {
    test("keeps the route as the list and authorization orchestrator", () => {
        expect(route.split("\n").length).toBeLessThan(400);
        expect(route).toContain('from "./-deploy-dialogs"');
        expect(route).toContain("<DeployActions record={row} onSuccess={refresh} />");
        expect(route).toContain('code="manage:deploy:create"');
        expect(route).toContain('code="manage:deploy:run"');
        expect(route).toContain('code="manage:deploy:update"');
        expect(route).toContain('code="manage:deploy:delete"');
        expect(route).toContain("function DeployStatusBadge");
        expect(route).toContain("function formatFileSize");
    });

    test("delegates stateful deployment dialogs without moving route authorization", () => {
        for (const component of [
            "UploadVersionDialog",
            "DeployVersionDialog",
            "ExpireVersionDialog",
            "DeleteVersionDialog",
            "CleanupDialog",
            "componentLabel",
        ]) {
            expect(dialogs).toContain(`export function ${component}`);
        }
        expect(dialogs).toContain("manageAPI.deploy.upload");
        expect(dialogs).toContain("manageAPI.deploy.deploy");
        expect(dialogs).toContain("manageAPI.deploy.expire");
        expect(dialogs).toContain("manageAPI.deploy.remove");
        expect(dialogs).toContain("manageAPI.deploy.cleanup");
        expect(dialogs).not.toContain("AuthWrap");
        expect(dialogs).not.toContain("DeployActions");
    });
});
