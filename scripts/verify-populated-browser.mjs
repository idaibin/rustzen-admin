import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync, openSync, closeSync } from "node:fs";
// Owned local acceptance only: no existing databases or external deployment.
import { createServer as httpServer } from "node:http";
import { createRequire } from "node:module";
import { createServer, createConnection } from "node:net";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(fileURLToPath(new URL("..", import.meta.url)));
const output = resolve(
    process.env.RUSTZEN_BROWSER_OUTPUT ?? "target/rz/populated-browser",
    new Date().toISOString().replaceAll(":", "-"),
);
mkdirSync(output, { recursive: true });
const runtime = resolve(output, "runtime");
mkdirSync(runtime);
const require = createRequire(import.meta.url);
const { chromium } = require(process.env.RUSTZEN_PLAYWRIGHT_MODULE ?? "playwright");
const browserPath = process.env.RUSTZEN_BROWSER_PATH;
assert(browserPath, "Set RUSTZEN_BROWSER_PATH to the isolated Chromium executable");
const sha = (path) => createHash("sha256").update(readFileSync(path)).digest("hex");
const source = execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim();
const binaries = Object.fromEntries(
    ["admin", "monitor", "insights", "reports"].map((name) => [
        name,
        sha(resolve(root, `target/debug/rz-${name}`)),
    ]),
);
const env = Object.fromEntries(
    Object.entries(process.env).filter(([key]) => !key.startsWith("RUSTZEN_")),
);
Object.assign(env, {
    RUSTZEN_ENV: "development",
    RUSTZEN_RUNTIME_ROOT: runtime,
    RUSTZEN_ADMIN_HOST: "127.0.0.1",
    RUSTZEN_INTERNAL_HOST: "127.0.0.1",
    RUSTZEN_BUILD_ID: "a".repeat(64),
    RUSTZEN_COMPOSITION_ID: "b".repeat(64),
    RUSTZEN_MONITOR_SCHEMA_FINGERPRINT: "c".repeat(64),
    RUSTZEN_MONITOR_DATA_CONTRACT_ID: "d".repeat(64),
    RUSTZEN_REPORTS_BROWSER_PATH: resolve(runtime, "chromium-wrapper"),
    RUST_LOG: "warn",
});
const processes = new Map();
const records = [];
const screenshots = [];
let browser;
let page;
const start = (name) => {
    const fd = openSync(resolve(output, `${name}.log`), "a");
    const child = spawn(
        resolve(root, `target/debug/rz-${name}`),
        [name === "monitor" ? "controller" : "serve"],
        { cwd: root, env, stdio: ["ignore", fd, fd] },
    );
    closeSync(fd);
    processes.set(name, child);
    return child;
};
const stop = async (name) => {
    const child = processes.get(name);
    if (!child || child.exitCode !== null || child.signalCode !== null) return;
    await new Promise((resolveStop) => {
        const timer = setTimeout(() => child.kill("SIGKILL"), 5000);
        child.once("exit", () => {
            clearTimeout(timer);
            resolveStop();
        });
        child.kill("SIGTERM");
    });
};
let agentPhase = true;
let auxiliaryAttempts = 0;
const countHTTP = () => {
    if (agentPhase) {
        auxiliaryAttempts++;
        assert(auxiliaryAttempts <= 6, "Agent auxiliary HTTP budget exhausted");
    }
};
const health = async (port) => {
    for (let i = 0; i < 100; i++) {
        countHTTP();
        try {
            const response = await fetch(`http://127.0.0.1:${port}/health`, {
                signal: AbortSignal.timeout(5000),
            });
            if (response.ok) return await response.json();
        } catch {}
        await new Promise((r) => setTimeout(r, 100));
    }
    throw new Error(`Service ${port} not healthy`);
};
const shot = async (name) => {
    const path = resolve(output, `${name}.png`);
    assert.equal(
        await page.evaluate(() => document.documentElement.scrollWidth > innerWidth),
        false,
        "Document overflow",
    );
    const modal = page.locator(".ant-modal:visible");
    if (await modal.count()) {
        const bounds = await modal.evaluate((el) => {
            const r = el.getBoundingClientRect();
            return { left: r.left, right: r.right, width: r.width, contentWidth: el.scrollWidth };
        });
        assert(
            bounds.left >= 0 && bounds.right <= (await page.evaluate(() => innerWidth)),
            JSON.stringify(bounds),
        );
        record("modal-contained", bounds);
    }
    await page.screenshot({ path });
    screenshots.push({
        name: `${name}.png`,
        sha256: sha(path),
        viewport: await page.evaluate(() => [innerWidth, innerHeight]),
        url: page.url(),
    });
};
const record = (name, data = {}) => {
    records.push({ name, ...data });
    console.log(JSON.stringify({ name, ...data }));
};
const base = "http://127.0.0.1:9801";
writeFileSync(
    env.RUSTZEN_REPORTS_BROWSER_PATH,
    `#!/bin/sh
exec "${browserPath}" --no-sandbox --no-zygote --disable-dev-shm-usage --disable-gpu "$@"
`,
    { mode: 0o700 },
);
const delay = (ms) => new Promise((r) => setTimeout(r, ms));
const db = (name, sql) =>
    JSON.parse(
        execFileSync(
            "python3",
            [
                "-c",
                "import sqlite3,json,sys; c=sqlite3.connect('file:'+sys.argv[1]+'?mode=ro',uri=True);c.row_factory=sqlite3.Row;print(json.dumps([dict(r) for r in c.execute(sys.argv[2])]))",
                resolve(runtime, `data/db/${name}.db`),
                sql,
            ],
            { encoding: "utf8" },
        ),
    );
let fixture;
let token;
const api = async (path, method = "GET", body) => {
    countHTTP();
    const response = await fetch(base + path, {
        signal: AbortSignal.timeout(5000),
        method,
        headers: {
            "content-type": "application/json",
            ...(token ? { authorization: `Bearer ${token}` } : {}),
        },
        ...(body ? { body: JSON.stringify(body) } : {}),
    });
    assert(response.ok, `${method} ${path}: ${response.status} ${await response.clone().text()}`);
    return (await response.json()).data;
};
try {
    for (const port of [9801, 9802, 9803, 9804, 9805])
        await new Promise((ok, no) => {
            const server = createServer();
            server.once("error", no);
            server.listen(port, "127.0.0.1", () => server.close(ok));
        });
    for (const name of Object.keys(binaries)) start(name);
    // Startup readiness uses TCP, preserving the six-request Agent budget.
    for (const port of [9801, 9802, 9803, 9804]) {
        let ready = false;
        for (let i = 0; i < 150; i++) {
            ready = await new Promise((ok) => {
                const socket = createConnection({ port, host: "127.0.0.1" });
                socket.once("connect", () => {
                    socket.destroy();
                    ok(true);
                });
                socket.once("error", () => ok(false));
                socket.setTimeout(500, () => {
                    socket.destroy();
                    ok(false);
                });
            });
            if (ready) break;
            await delay(100);
        }
        assert(ready, `Service port ${port} not ready`);
    }
    for (const port of [9801, 9802]) record(`health-${port}`, { response: await health(port) });
    // Agent phase: two health checks, login and two gateway reads.
    token = (await api("/api/auth/login", "POST", { username: "owner", password: "rustzen@123" }))
        .token;
    // The Agent is the sole writer of Monitor resource samples.
    const fd = openSync(resolve(output, "agent.log"), "a");
    const agent = spawn(resolve(root, "target/debug/rz-monitor-agent"), [], {
        cwd: root,
        env: {
            ...env,
            RUSTZEN_MONITOR_NODE_ID: "owned-real-agent",
            RUSTZEN_MONITOR_CONTROLLER_URL: base,
        },
        stdio: ["ignore", fd, fd],
    });
    closeSync(fd);
    processes.set("agent", agent);
    const started = Date.now();
    let samples = [];
    while (Date.now() - started < 90000) {
        samples = db("monitor", "select * from resource_samples order by id");
        if (samples.length >= 2) break;
        assert(agent.exitCode === null, "Agent exited before two samples");
        await delay(500);
    }
    await stop("agent");
    assert.equal(samples.length, 2);
    assert(
        samples.every(
            (x) => x.memory_total_bytes > 0 && x.cpu_percent >= 0 && x.cpu_percent <= 100,
        ),
    );
    const spacing =
        (Date.parse(samples[1].collected_at) - Date.parse(samples[0].collected_at)) / 1000;
    assert(spacing >= 29 && spacing <= 32, `Unexpected cadence ${spacing}`);
    const nodes = await api("/api/monitor/nodes");
    const metrics = await api("/api/monitor/nodes/owned-real-agent/metrics?bucket=raw");
    assert.equal(metrics.points.length, 2);
    assert.equal(nodes[0].sequence, 2);
    assert.equal(nodes[0].nodeId, "owned-real-agent");
    record("real-agent-two-samples", {
        samples,
        spacing,
        nodes,
        auxiliaryAttempts,
        agentSHA256: sha(resolve(root, "target/debug/rz-monitor-agent")),
    });
    agentPhase = false;
    for (const port of [9803, 9804]) record(`health-${port}`, { response: await health(port) });
    browser = await chromium.launch({
        executablePath: browserPath,
        headless: true,
        args: ["--no-sandbox", "--no-zygote", "--disable-dev-shm-usage", "--disable-gpu"],
    });
    const context = await browser.newContext({ viewport: { width: 1920, height: 1080 } });
    page = await context.newPage();
    const errors = [];
    page.on("pageerror", (e) => errors.push(e.message));
    await page.goto(base + "/login");
    await page.locator("#login_username").fill("owner");
    await page.locator("#login_password").fill("rustzen@123");
    await page.locator("#login_password").press("Enter");
    await page.waitForURL(base + "/");
    await page.getByRole("heading", { name: "仪表盘", exact: true }).waitFor();
    for (const theme of ["light", "dark"]) {
        await page.evaluate((t) => localStorage.setItem("rustzen-admin-theme", t), theme);
        await page.goto(base + "/monitoring/nodes");
        await page.locator('[data-testid="monitor-node-view"]').first().click();
        await page.locator('[data-testid="monitor-node-details"]').waitFor();
        await delay(700);
        assert(await page.getByText("未采集到磁盘挂载点", { exact: true }).count());
        assert(
            await page
                .getByText(`${nodes[0].memory.usagePercent.toFixed(1)}%`, { exact: true })
                .count(),
        );
        await shot(`real-agent-${theme}`);
        const chart = page.getByTestId("monitor-node-history-5m-owned-real-agent");
        await chart.scrollIntoViewIfNeeded();
        await chart.locator(".recharts-dot").last().hover();
        await delay(600);
        const tooltip = await chart.locator(".recharts-tooltip-wrapper").innerText();
        assert(!/\d+\.\d{3,}/.test(tooltip), "Tooltip percentage precision");
        await shot(`real-agent-history-${theme}`);
    }
    fixture = httpServer((req, res) => {
        res.setHeader("content-type", "text/html; charset=utf-8");
        res.end(
            req.url.startsWith("/api/")
                ? "owned response"
                : '<!doctype html><html><meta charset="utf-8"><title>Owned acceptance</title><h1 id="owned-title">Rustzen owned acceptance</h1><p>真实采集与报表渲染测试</p><script src="http://127.0.0.1:9801/api/insights/tracker.js"></script></html>',
        );
    });
    await new Promise((r) => fixture.listen(9805, "127.0.0.1", r));
    await api("/api/insights/collection-policy", "PUT", {
        collectionEnabled: true,
        projectKey: "owned-project-key",
        allowedOrigins: ["http://127.0.0.1:9805"],
    });
    const tracker = await context.newPage();
    await tracker.goto("http://127.0.0.1:9805/owned?private=discard");
    assert.equal(db("insights", "select * from insights_events").length, 0);
    assert.equal(await tracker.evaluate(() => localStorage.getItem("rz_vid")), null);
    await tracker.evaluate(async () => {
        window.rustzenAnalytics.enable({
            consent: true,
            projectKey: "owned-project-key",
            endpoint: "http://127.0.0.1:9801/api/insights/track",
        });
        await fetch("/api/owned");
        window.rustzenAnalytics.track("custom_export", {
            properties: { feature: "owned", format: "png", result: "success" },
        });
    });
    for (let i = 0; i < 40 && db("insights", "select * from insights_events").length < 3; i++)
        await delay(250);
    const events = db(
        "insights",
        "select event_name,page_path,api_path from insights_events order by id",
    );
    assert.equal(events.length, 3);
    assert(events.some((e) => e.page_path === "/owned"));
    await tracker.evaluate(async () => {
        window.rustzenAnalytics.optOut();
        await fetch("/api/after-optout");
        window.rustzenAnalytics.track("custom_export");
    });
    await delay(5500);
    assert.equal(db("insights", "select * from insights_events").length, 3);
    assert.equal(await tracker.evaluate(() => localStorage.getItem("rz_vid")), null);
    record("real-browser-tracker-consent-and-optout", { events });
    await tracker.close();
    for (const [width, height] of [
        [1920, 1080],
        [1440, 900],
        [390, 844],
    ]) {
        await page.setViewportSize({ width, height });
        for (const theme of ["light", "dark"]) {
            await page.evaluate((t) => localStorage.setItem("rustzen-admin-theme", t), theme);
            await page.goto(base + "/analytics/overview");
            await delay(1000);
            assert.equal(await page.locator(".recharts-wrapper").count(), 1);
            await page.getByRole("heading",{name:"分析概览",exact:true}).scrollIntoViewIfNeeded();
            await shot(`populated-analytics-top-${theme}-${width}`);
            await page.locator(".recharts-dot").last().hover();
            await delay(600);
            assert.equal(
                await page.evaluate(() => document.documentElement.classList.contains("dark")),
                theme === "dark",
            );
            await shot(`populated-analytics-${theme}-${width}`);
        }
    }
    await page.setViewportSize({ width: 1920, height: 1080 });
    const system = await api("/api/reports/systems", "POST", {
        name: "Owned local acceptance",
        baseUrl: "http://127.0.0.1:9805",
    });
    const flow = await api("/api/reports/flows", "POST", {
        systemId: system.id,
        name: "Owned rendering",
        steps: [
            { action: "goto", url: "/" },
            { action: "waitFor", selector: "#owned-title" },
            { action: "assertText", selector: "#owned-title", text: "Rustzen owned acceptance" },
            { action: "screenshot", name: "owned-render" },
        ],
    });
    await page.goto(base + "/reports/runs");
    await page.getByTestId("run-create").click();
    const runResponse = page.waitForResponse(
        (r) => r.url().endsWith("/api/reports/runs") && r.request().method() === "POST",
    );
    await page.getByRole("button", { name: "提交执行", exact: true }).click();
    const run = (await (await runResponse).json()).data;
    let current;
    for (let i = 0; i < 90; i++) {
        current = await api(`/api/reports/runs/${run.id}`);
        if (!["queued", "running", "cancelling"].includes(current.status)) break;
        await delay(1000);
    }
    assert.equal(current.status, "succeeded", JSON.stringify(current));
    const steps = await api(`/api/reports/runs/${run.id}/steps`);
    assert.equal(steps.length, 4);
    assert(steps.every((s) => s.status === "succeeded"));
    const artifacts = await api(`/api/reports/runs/${run.id}/artifacts`);
    assert(artifacts.some((a) => a.kind === "screenshot"));
    for (const artifact of artifacts) {
        const response = await fetch(
            `${base}/api/reports/runs/${run.id}/artifacts/${artifact.id}`,
            { headers: { authorization: `Bearer ${token}` } },
        );
        assert(response.ok);
        const bytes = Buffer.from(await response.arrayBuffer());
        assert(bytes.length > 100);
        assert.equal(bytes.subarray(0, 8).toString("hex"), "89504e470d0a1a0a");
        artifact.downloadSHA256 = createHash("sha256").update(bytes).digest("hex");
        writeFileSync(
            resolve(
                output,
                `report-${artifact.kind}-${artifact.id}.${artifact.kind === "pdf" ? "pdf" : "png"}`,
            ),
            bytes,
        );
    }
    const reloadAudit = async (label) => {
        const control = page.locator(".ant-modal button").filter({ hasText: label });
        await control.scrollIntoViewIfNeeded();
        const debug = await control.evaluate((el) => ({ label: el.textContent, visibility: getComputedStyle(el).visibility, rect: {top:el.getBoundingClientRect().top,bottom:el.getBoundingClientRect().bottom}, disabled: el.disabled }));
        record("audit-reload-control", debug);
        assert.equal(await page.getByRole("button", { name: label, exact: true }).count(), 1, "Reload control missing from accessibility tree");
        await control.click();
    };
    for (const [width, height] of [
        [1920, 1080],
        [1440, 900],
        [390, 844],
    ]) {
        await page.setViewportSize({ width, height });
        for (const theme of ["light", "dark"]) {
            await page.evaluate((t) => localStorage.setItem("rustzen-admin-theme", t), theme);
            await page.goto(base + "/reports/runs");
            await page.getByTestId(`run-view-${run.id}`).click();
            await page.getByTestId("run-audit").waitFor({ state: "attached" });
            await delay(500);
            assert.equal(
                await page.evaluate(() => document.documentElement.classList.contains("dark")),
                theme === "dark",
            );
            await shot(`reports-success-${theme}-${width}`);
            await page
                .getByRole("button", {
                    name: artifacts.find((a) => a.kind === "screenshot").fileName,
                    exact: true,
                })
                .scrollIntoViewIfNeeded();
            await shot(`reports-artifacts-${theme}-${width}`);

            // Keyboard dismissal and focus restoration use the existing Ant Modal owner.
            await page.keyboard.press("Escape");
            await page.getByRole("dialog").waitFor({ state: "hidden" });
            const trigger = page.getByTestId(`run-view-${run.id}`);
            assert(await trigger.evaluate((el) => el === document.activeElement), "Audit trigger focus not restored");

            // Explicit owned transport faults: no backend business result is fabricated.
            let phase = "hold";
            let releaseReads;
            const readsHeld = new Promise((resolveRead) => { releaseReads = resolveRead; });
            const readFault = async (route) => {
                if (phase === "hold") await readsHeld;
                if (phase === "pass") return route.continue();
                return route.fulfill({ status: 503, contentType: "application/json", body: JSON.stringify({ code: 503, message: "Owned audit read fault" }) });
            };
            const stepPath = `${base}/api/reports/runs/${run.id}/steps`;
            const artifactPath = `${base}/api/reports/runs/${run.id}/artifacts`;
            await page.route(stepPath, readFault);
            await page.route(artifactPath, readFault);
            await page.reload(); // Fresh client query cache for initial loading/failure acceptance.
            await trigger.focus();
            await trigger.press("Enter");
            await page.getByText("正在加载步骤", { exact: true }).waitFor();
            await page.getByText("正在加载产物", { exact: true }).waitFor();
            assert(await page.evaluate(() => Boolean(document.activeElement?.closest('.ant-modal'))), "Initial focus outside audit");
            await delay(500); // Settle the modal opening animation before geometry/capture.
            await shot(`audit-loading-${theme}-${width}`);
            phase = "fail";
            releaseReads();
            await page.getByText("步骤加载失败", { exact: true }).waitFor();
            await page.getByText("产物加载失败", { exact: true }).waitFor();
            assert.equal(await page.getByText("暂无产物", { exact: true }).count(), 0);
            await page.getByText("产物加载失败", { exact: true }).scrollIntoViewIfNeeded();
            await shot(`audit-initial-error-${theme}-${width}`);
            phase = "pass";
            await reloadAudit("重新加载产物");
            await page.getByText("产物加载失败", { exact: true }).waitFor({ state: "hidden" });
            await reloadAudit("重新加载步骤");
            const artifactControl = page.getByRole("button", { name: artifacts.find((a) => a.kind === "screenshot").fileName, exact: true });
            await artifactControl.waitFor();
            await page.getByText("步骤加载失败", { exact: true }).waitFor({ state: "hidden" });
            await page.getByText("产物加载失败", { exact: true }).waitFor({ state: "hidden" });
            await artifactControl.scrollIntoViewIfNeeded();
            await shot(`audit-recovered-${theme}-${width}`);
            phase = "fail";
            await reloadAudit("重新加载步骤");
            await reloadAudit("重新加载产物");
            await page.getByText("产物加载失败", { exact: true }).waitFor();
            assert.equal(await artifactControl.count(), 1, "Cached artifact was lost");
            assert(await page.getByText("1. goto", { exact: true }).count(), "Cached steps were lost");
            await page.getByText("产物加载失败", { exact: true }).scrollIntoViewIfNeeded();
            await shot(`audit-cached-error-${theme}-${width}`);
            phase = "pass";
            await reloadAudit("重新加载产物");
            await page.getByText("产物加载失败", { exact: true }).waitFor({ state: "hidden" });
            await reloadAudit("重新加载步骤");
            await page.getByText("步骤加载失败", { exact: true }).waitFor({ state: "hidden" });
            await page.getByText("产物加载失败", { exact: true }).waitFor({ state: "hidden" });
            await page.unroute(stepPath, readFault);
            await page.unroute(artifactPath, readFault);

            // Hold a real download action, reject once, then retry the native artifact bytes.
            let downloadRequests = 0;
            let releaseDownload;
            const downloadHeld = new Promise((resolveDownload) => { releaseDownload = resolveDownload; });
            const downloadPath = `${base}/api/reports/runs/${run.id}/artifacts/${artifacts.find((a) => a.kind === "screenshot").id}`;
            const downloadFault = async (route) => {
                downloadRequests++;
                await downloadHeld;
                return route.fulfill({ status: 503, contentType: "application/json", body: JSON.stringify({ code: 503, message: "Owned download fault" }) });
            };
            await page.route(downloadPath, downloadFault);
            await artifactControl.evaluate((el) => { el.click(); el.click(); el.click(); });
            await page.waitForFunction(() => document.querySelector('button[aria-busy="true"]'));
            await artifactControl.scrollIntoViewIfNeeded();
            await shot(`download-pending-${theme}-${width}`);
            assert.equal(downloadRequests, 1, "Repeated activation dispatched duplicate download");
            releaseDownload();
            await page.getByText("Owned download fault", { exact: true }).waitFor();
            await page.waitForFunction(() => !document.querySelector('button[aria-busy="true"]'));
            await shot(`download-error-${theme}-${width}`);
            await page.unroute(downloadPath, downloadFault);
            const recoveredDownload = page.waitForEvent("download");
            await artifactControl.focus();
            await artifactControl.press("Enter");
            const recovered = await recoveredDownload;
            const recoveredFile = resolve(output, `recovered-download-${theme}-${width}.png`);
            await recovered.saveAs(recoveredFile);
            assert.equal(sha(recoveredFile), artifacts.find((a) => a.kind === "screenshot").downloadSHA256);
            await page.waitForFunction(() => !document.querySelector('button[aria-busy="true"]'));
            assert(await artifactControl.evaluate((el) => el === document.activeElement), "Download did not retain keyboard focus");
            // Actual Tab navigation must remain within the dialog, including its boundary.
            for (let tab = 0; tab < 12; tab++) {
                await page.keyboard.press("Tab");
                const active = await page.evaluate(() => ({ inside: Boolean(document.activeElement?.closest('.ant-modal')), tag: document.activeElement?.tagName, name: document.activeElement?.getAttribute('aria-label'), text: document.activeElement?.textContent?.slice(0, 80) }));
                assert(active.inside, `Tab escaped audit: ${JSON.stringify(active)}`);
            }
            await page.keyboard.press("Shift+Tab");
            assert(await page.evaluate(() => Boolean(document.activeElement?.closest('.ant-modal'))), "Reverse Tab escaped audit");
            await shot(`audit-keyboard-${theme}-${width}`);
            record("reports-interaction-recovery", { width, height, theme, initialLoading: true, initialReadFailure: true, staleRowsRetained: true, realRetry: true, downloadRequests, downloadSHA256: sha(recoveredFile), focusTrap: true, focusRestored: true, fault: "owned 503 browser transport interception" });
        }
    }
    const screenshotArtifact = artifacts.find((a) => a.kind === "screenshot");
    const downloadPromise = page.waitForEvent("download");
    await page.getByRole("button", { name: screenshotArtifact.fileName, exact: true }).click();
    const download = await downloadPromise;
    const downloadedPath = resolve(output, "report-ui-download.png");
    await download.saveAs(downloadedPath);
    assert.equal(sha(downloadedPath), screenshotArtifact.downloadSHA256);
    record("real-reports-ui-run-and-artifact-download", { run: current, steps, artifacts });
    // Cross-consumer acceptance for the shared Ant dialog focus owner; no profile writes.
    for (const [width, height] of [[1920,1080],[390,844]]) {
        await page.setViewportSize({width,height});
        for (const theme of ["light","dark"]) {
            await page.evaluate((t)=>localStorage.setItem("rustzen-admin-theme",t),theme);
            await page.goto(base+"/profile");
            const editProfile=page.getByRole("button",{name:"编辑个人资料",exact:true});
            await editProfile.focus();await editProfile.press("Enter");
            await page.getByRole("dialog").waitFor();await delay(500);
            for (const key of ["Tab","Shift+Tab"]) for(let i=0;i<16;i++) {
                await page.keyboard.press(key);
                assert(await page.evaluate(()=>Boolean(document.activeElement?.closest('.ant-modal'))), `Profile ${key} focus escaped`);
            }
            await shot(`profile-keyboard-${theme}-${width}`);
            await page.keyboard.press("Escape");
            await page.getByRole("dialog").waitFor({state:"hidden"});
            assert(await editProfile.evaluate(el=>el===document.activeElement),"Profile focus not restored");
            record("shared-dialog-keyboard",{width,height,theme,forward:true,reverse:true,focusRestored:true,profileWrites:0});
        }
    }
    // A separate authenticated browser receives only the Reports read capability.
    const menus = await api("/api/system/menus");
    const runView = menus.find((m) => m.code === "reports:run:view");
    assert(runView);
    const role = {
        name: "Owned Reports viewer",
        code: "owned_reports_viewer",
        status: 1,
        menuIds: [runView.id],
        description: "Owned browser fixture",
    };
    await api("/api/system/roles", "POST", role);
    const roles = await api("/api/system/roles?current=1&pageSize=100");
    const roleId = roles.find((r) => r.code === role.code).id;
    await api("/api/system/users", "POST", {
        username: "owned_reports_viewer",
        email: "viewer@example.invalid",
        password: "OwnedReportsFixture123!",
        realName: "Owned viewer",
        roleIds: [roleId],
        status: 1,
    });
    const ownerToken = token;
    token = (
        await api("/api/auth/login", "POST", {
            username: "owned_reports_viewer",
            password: "OwnedReportsFixture123!",
        })
    ).token;
    const viewerToken = token;
    const me = await api("/api/auth/me");
    assert.deepEqual(me.permissions, ["reports:run:view"]);
    token = ownerToken;
    const viewerContext = await browser.newContext({ viewport: { width: 390, height: 844 } });
    page = await viewerContext.newPage();
    page.on("pageerror", (e) => errors.push(e.message));
    await page.goto(base + "/login");
    await page.locator("#login_username").fill("owned_reports_viewer");
    await page.locator("#login_password").fill("OwnedReportsFixture123!");
    await page.locator("#login_password").press("Enter");
    // This viewer lacks dashboard:view; the current root contract lands on 403.
    await page.waitForURL(base + "/403");
    await page.getByText("403 · 禁止访问", {exact:true}).waitFor();
    for (const theme of ["light", "dark"]) {
        await page.evaluate((t) => localStorage.setItem("rustzen-admin-theme", t), theme);
        await page.goto(base + "/reports/runs");
        await page.getByTestId(`run-view-${run.id}`).click();
        assert.equal(await page.getByTestId("run-create").count(), 0);
        await page
            .getByRole("button", { name: screenshotArtifact.fileName, exact: true })
            .scrollIntoViewIfNeeded();
        const pending = page.waitForEvent("download");
        await page.getByRole("button", { name: screenshotArtifact.fileName, exact: true }).click();
        const received = await pending;
        const file = resolve(output, `viewer-download-${theme}.png`);
        await received.saveAs(file);
        assert.equal(sha(file), screenshotArtifact.downloadSHA256);
        await shot(`reports-viewer-download-${theme}-390`);
    }
    const deniedWrite = await fetch(base + "/api/reports/runs", {
        method: "POST",
        headers: { authorization: `Bearer ${viewerToken}`, "content-type": "application/json" },
        body: JSON.stringify({ flowId: flow.id, input: {} }),
    });
    assert.equal(deniedWrite.status, 403);
    await api(`/api/system/roles/${roleId}`, "PUT", {
        ...role,
        menuIds: [menus.find((m) => m.code === "reports:schedule:view").id],
    });
    const deniedDownload = await fetch(
        `${base}/api/reports/runs/${run.id}/artifacts/${screenshotArtifact.id}`,
        { headers: { authorization: `Bearer ${viewerToken}` } },
    );
    assert.equal(deniedDownload.status, 403);
    record("reports-read-only-viewer", {
        permissions: me.permissions,
        downloadSHA256: screenshotArtifact.downloadSHA256,
        deniedWrite: deniedWrite.status,
        revokedDownload: deniedDownload.status,
    });
    assert.deepEqual(errors, []);
    assert.deepEqual(
        Object.fromEntries(
            Object.keys(binaries).map((name) => [
                name,
                sha(resolve(root, `target/debug/rz-${name}`)),
            ]),
        ),
        binaries,
    );
    writeFileSync(
        resolve(output, "result.json"),
        JSON.stringify(
            {
                status: "passed",
                source,
                runnerSHA256: sha(fileURLToPath(import.meta.url)),
                webPackageSHA256: sha(resolve(root,"apps/web/package.json")),
                webLockSHA256: sha(resolve(root,"apps/web/bun.lock")),
                dependencyPatches: Object.fromEntries(Object.entries(JSON.parse(readFileSync(resolve(root,"apps/web/package.json"),"utf8")).patchedDependencies ?? {}).map(([name,path])=>[name,{path,sha256:sha(resolve(root,"apps/web",path))}])),
                reportUISourceSHA256: sha(
                    resolve(root, "apps/web/src/routes/reports/-runs/run-details.tsx"),
                ),
                uiSourceSHA256: sha(
                    resolve(root, "apps/web/src/routes/monitoring/-node-details.tsx"),
                ),
                binaries,
                browserVersion: browser.version(),
                records,
                screenshots,
                limitations: [
                    "Owned development fixture only; no systemd, production, release or main acceptance",
                    "Single Agent stopped after second genuine sample",
                    "Reports target is an owned local fixture; not an external portal",
                ],
            },
            null,
            2,
        ),
    );
} catch (error) {
    if (page) {
        await page.screenshot({ path: resolve(output, "failure.png") }).catch(() => {});
        writeFileSync(resolve(output,"failure.html"), await page.content().catch(() => "Unavailable"));
    }
    writeFileSync(
        resolve(output, "result.json"),
        JSON.stringify(
            {
                status: "failed",
                source,
                binaries,
                error: String(error.stack),
                records,
                screenshots,
            },
            null,
            2,
        ),
    );
    throw error;
} finally {
    await browser?.close();
    fixture?.close();
    for (const name of processes.keys()) await stop(name);
    console.log(`Evidence: ${output}`);
}
