// Owned local acceptance only: no existing databases or external deployment.
import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { createServer } from "node:net";
import { mkdirSync, readFileSync, writeFileSync, openSync, closeSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(fileURLToPath(new URL("..", import.meta.url)));
const output = resolve(process.env.RUSTZEN_BROWSER_OUTPUT ?? "target/rz/profile-browser", new Date().toISOString().replaceAll(":", "-"));
mkdirSync(output, { recursive: true });
const runtime = resolve(output, "runtime");
mkdirSync(runtime);
const require = createRequire(import.meta.url);
const { chromium } = require(process.env.RUSTZEN_PLAYWRIGHT_MODULE ?? "playwright");
const browserPath = process.env.RUSTZEN_BROWSER_PATH;
assert(browserPath, "Set RUSTZEN_BROWSER_PATH to the isolated Chromium executable");
const sha = (path) => createHash("sha256").update(readFileSync(path)).digest("hex");
const source = execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim();
const binaries = Object.fromEntries(["admin", "monitor", "insights", "reports"].map(name => [name, sha(resolve(root, `target/debug/rz-${name}`))]));
const env = Object.fromEntries(Object.entries(process.env).filter(([key]) => !key.startsWith("RUSTZEN_")));
Object.assign(env, {
    RUSTZEN_ENV: "development", RUSTZEN_RUNTIME_ROOT: runtime,
    RUSTZEN_ADMIN_HOST: "127.0.0.1", RUSTZEN_INTERNAL_HOST: "127.0.0.1",
    RUSTZEN_BUILD_ID: "a".repeat(64), RUSTZEN_COMPOSITION_ID: "b".repeat(64),
    RUSTZEN_MONITOR_SCHEMA_FINGERPRINT: "c".repeat(64), RUSTZEN_MONITOR_DATA_CONTRACT_ID: "d".repeat(64),
    RUSTZEN_REPORTS_BROWSER_PATH: resolve(runtime, "no-report-browser"), RUST_LOG: "warn",
});
const processes = new Map();
const records = [];
const screenshots = [];
let browser;
let page;
const start = (name) => {
    const fd = openSync(resolve(output, `${name}.log`), "a");
    const child = spawn(resolve(root, `target/debug/rz-${name}`), [name === "monitor" ? "controller" : "serve"], { cwd: root, env, stdio: ["ignore", fd, fd] });
    closeSync(fd);
    processes.set(name, child);
    return child;
};
const stop = async (name) => {
    const child = processes.get(name);
    if (!child || child.exitCode !== null) return;
    await new Promise(resolveStop => {
        const timer = setTimeout(() => child.kill("SIGKILL"), 5000);
        child.once("exit", () => { clearTimeout(timer); resolveStop(); });
        child.kill("SIGTERM");
    });
};
const health = async (port) => {
    for (let i = 0; i < 100; i++) {
        try { const response = await fetch(`http://127.0.0.1:${port}/health`); if (response.ok) return await response.json(); } catch {}
        await new Promise(r => setTimeout(r, 100));
    }
    throw new Error(`Service ${port} not healthy`);
};
const shot = async (name) => {
    const path = resolve(output, `${name}.png`);
    await page.screenshot({ path });
    screenshots.push({ name: `${name}.png`, sha256: sha(path), viewport: await page.evaluate(() => [innerWidth, innerHeight]), url: page.url() });
};
const record = (name, data = {}) => { records.push({ name, ...data }); console.log(JSON.stringify({ name, ...data })); };
const base = "http://127.0.0.1:9801";
try {
    // Refuse occupied ports before starting any process or authenticating.
    for (const port of [9801, 9802, 9803, 9804]) {
        await new Promise((resolvePort, reject) => {
            const server = createServer();
            server.once("error", reject);
            server.listen(port, "127.0.0.1", () => server.close(resolvePort));
        });
    }
    for (const name of Object.keys(binaries)) start(name);
    for (const port of [9801, 9802, 9803, 9804]) record(`health-${port}`, { response: await health(port) });
    browser = await chromium.launch({ executablePath: browserPath, headless: true, args: ["--no-sandbox", "--no-zygote", "--disable-dev-shm-usage", "--disable-gpu"] });
    const context = await browser.newContext({ viewport: { width: 1920, height: 1080 }, deviceScaleFactor: 1 });
    page = await context.newPage();
    const errors = [];
    page.on("pageerror", error => errors.push(error.message));
    await page.goto(`${base}/login`);
    await page.locator("#login_username").fill("owner");
    await page.locator("#login_password").fill("rustzen@123");
    await page.locator("#login_password").press("Enter");
    await page.waitForURL(`${base}/`);
    await page.goto(`${base}/profile`);
    await page.getByRole("button", { name: /上传头像/ }).waitFor();
    const input = page.locator('input[type="file"]');
    let requests = 0;
    page.on("request", request => { if (request.method() === "POST" && new URL(request.url()).pathname === "/api/account/avatar") requests++; });
    const image = readFileSync(resolve(root, "apps/web/public/rustzen.png"));
    const fixture = { name: "owned-avatar.png", mimeType: "image/png", buffer: image };
    assert(image.length < 1024 * 1024);
    for (const rejected of [
        { name: "owned-invalid.txt", mimeType: "text/plain", buffer: Buffer.from("invalid") },
        { name: "owned-large.png", mimeType: "image/png", buffer: Buffer.alloc(1024 * 1024 + 1) },
    ]) {
        const before = requests;
        await input.setInputFiles(rejected);
        await page.locator(".ant-message-error").last().waitFor();
        assert.equal(requests, before, "Client rejects unsupported and oversized files before dispatch");
        record(`client-reject-${rejected.name}`, { requests: requests - before });
    }
    // This is a genuine backend decode rejection, not a mocked API response.
    const corruptResponse = page.waitForResponse(r => r.url().endsWith("/api/account/avatar") && r.request().method() === "POST");
    await input.setInputFiles({ name: "owned-corrupt.png", mimeType: "image/png", buffer: Buffer.from("not an image") });
    const rejected = await corruptResponse;
    assert.equal(rejected.ok(), false);
    record("backend-reject-corrupt", { status: rejected.status() });
    await page.locator(".ant-message-error").last().waitFor();
    await shot("avatar-backend-rejection-light");
    // One explicitly injected transport fault, followed by a real retry.
    await page.route("**/api/account/avatar", route => route.abort("failed"), { times: 1 });
    const failedRequest = page.waitForEvent("requestfailed", request => request.url().endsWith("/api/account/avatar"));
    await input.setInputFiles(fixture);
    await failedRequest;
    await page.locator(".ant-message-error").last().waitFor();
    await page.waitForFunction(() => !document.querySelector(".ant-btn-loading"));
    record("injected-network-failure", { mock: "one aborted avatar request" });
    let release;
    const held = new Promise(r => { release = r; });
    let entered;
    const pending = new Promise(r => { entered = r; });
    await page.route("**/api/account/avatar", async route => { entered(); await held; await route.continue(); }, { times: 1 });
    const before = requests;
    const responsePromise = page.waitForResponse(r => r.url().endsWith("/api/account/avatar") && r.request().method() === "POST");
    // Trigger the actual browser filechooser event; no native OS picker claim.
    const chooserPromise = page.waitForEvent("filechooser");
    await page.getByRole("button", { name: /上传头像/ }).click();
    const chooser = await chooserPromise;
    await chooser.setFiles(fixture);
    await pending;
    await page.getByRole("button", { name: /正在上传/ }).waitFor();
    assert.equal(await input.isDisabled(), true);
    await shot("avatar-upload-pending-light");
    release();
    const response = await responsePromise;
    assert(response.ok());
    const body = await response.json();
    const avatarUrl = body.data;
    assert.equal(typeof avatarUrl, "string");
    assert.equal(requests - before, 1);
    const avatar = page.locator(`img[src="${avatarUrl}"]`).first();
    await avatar.waitFor();
    await page.waitForFunction(url => [...document.images].some(img => img.getAttribute("src") === url && img.complete && img.naturalWidth > 0), avatarUrl);
    const fetched = await context.request.get(`${base}${avatarUrl}`);
    assert(fetched.ok());
    assert.deepEqual(await fetched.body(), image);
    record("real-avatar-upload", { status: response.status(), requests: requests - before, resourceSHA256: createHash("sha256").update(image).digest("hex") });
    await shot("profile-avatar-light");
    await page.reload();
    await avatar.waitFor();
    record("avatar-page-reload-persistence");
    await stop("admin"); start("admin"); await health(9801);
    await page.reload(); await avatar.waitFor();
    record("avatar-owned-admin-process-restart-persistence");
    // Fresh browser session proves the saved server value, without persisted Zustand state.
    await context.clearCookies();
    await page.evaluate(() => localStorage.clear());
    await page.goto(`${base}/login`);
    await page.locator("#login_username").fill("owner");
    await page.locator("#login_password").fill("rustzen@123");
    await page.locator("#login_password").press("Enter");
    await page.waitForURL(`${base}/`);
    await page.goto(`${base}/profile`); await page.locator(`img[src="${avatarUrl}"]`).first().waitFor();
    record("avatar-fresh-login-persistence");
    const routes = ["/", "/monitoring/overview", "/monitoring/nodes", "/monitoring/incidents", "/monitoring/summaries", "/analytics/overview", "/analytics/details", "/reports/templates", "/reports/runs", "/system/user", "/system/role", "/system/menu", "/system/module", "/system/status", "/system/module-log", "/manage/log", "/manage/task", "/manage/deploy", "/profile"];
    for (const [width, height] of [[1920, 1080], [1440, 900], [390, 844]]) {
        await page.setViewportSize({ width, height });
        for (const theme of ["light", "dark"]) {
            await page.evaluate(value => localStorage.setItem("rustzen-admin-theme", value), theme);
            for (const route of routes) {
                await page.goto(`${base}${route}`);
                await page.locator(".shell-content").waitFor();
                await page.waitForTimeout(350);
                assert.equal(new URL(page.url()).pathname, route);
                const state = await page.evaluate(() => ({ viewport: [innerWidth, innerHeight], dark: document.documentElement.classList.contains("dark"), overflow: document.documentElement.scrollWidth > innerWidth, heading: document.querySelector("h1")?.textContent, alerts: [...document.querySelectorAll('[role="alert"]')].map(el => el.textContent) }));
                assert.deepEqual(state.viewport, [width, height]);
                assert.equal(state.dark, theme === "dark"); assert.equal(state.overflow, false);
                assert(state.heading, "Page has a heading");
                assert.equal(state.alerts.length, 0, `Unexpected route error on ${route}`);
                record("route-render", { route, theme, ...state });
                if (width === 1920 && ["/", "/profile", "/system/user", "/system/module", "/analytics/overview", "/reports/runs"].includes(route)) await shot(`${route === "/" ? "dashboard" : route.replaceAll("/", "-").slice(1)}-${theme}`);
                if (route === "/profile" && width !== 1920) await shot(`profile-${theme}-${width}`);
            }
        }
    }
    assert.deepEqual(errors, []);
    assert.deepEqual(Object.fromEntries(Object.keys(binaries).map(name => [name, sha(resolve(root, `target/debug/rz-${name}`))])), binaries);
    record("no-browser-page-errors");
    writeFileSync(resolve(output, "result.json"), JSON.stringify({ status: "passed", source, binaries, browserVersion: browser.version(), runnerSHA256: sha(fileURLToPath(import.meta.url)), records, screenshots, avatarRequests: requests, limitations: ["Native operating-system picker UI not automated", "One network failure and one request delay explicitly injected", "Route matrix proves rendering, not all business write journeys", "Fresh fixture has no monitoring nodes or Analytics events", "No signed package, systemd, production or main integration acceptance"] }, null, 2));
} catch (error) {
    if (page) await shot("failure").catch(() => {});
    writeFileSync(resolve(output, "result.json"), JSON.stringify({ status: "failed", source, binaries, error: String(error.stack), records, screenshots }, null, 2));
    throw error;
} finally {
    await browser?.close();
    for (const name of processes.keys()) await stop(name);
    console.log(`Evidence: ${output}`);
}
