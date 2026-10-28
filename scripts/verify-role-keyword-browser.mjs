// D-04 only. Own the server, ephemeral SQLite and browser; never use an existing instance.
import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import { createHash, randomBytes } from "node:crypto";
import { createRequire } from "node:module";
import { createServer } from "node:net";
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync, rmSync, openSync, closeSync } from "node:fs";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

assert.equal(process.env.RUSTZEN_BROWSER_DISPOSABLE, "1", "Explicit disposable-run authorization required");
const root = resolve(fileURLToPath(new URL("..", import.meta.url)));
const require = createRequire(import.meta.url);
const { chromium } = require(process.env.RUSTZEN_PLAYWRIGHT_MODULE ?? "playwright");
const output = resolve(process.env.RUSTZEN_BROWSER_OUTPUT ?? "target/rz/role-keyword-browser");
mkdirSync(output, { recursive: true });
const runtime = mkdtempSync(resolve(tmpdir(), "rz-d04-browser-"));
const binary = resolve(root, "target/debug/rz-admin");
const sha = (p) => createHash("sha256").update(readFileSync(p)).digest("hex");
const source = execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim();
const port = 19817;
const base = `http://127.0.0.1:${port}`;
const env = Object.fromEntries(Object.entries(process.env).filter(([key]) => !key.startsWith("RUSTZEN_")));
Object.assign(env, { RUSTZEN_SKIP_DOTENV: "1", RUSTZEN_ENV: "development", RUSTZEN_RUNTIME_ROOT: runtime,
    RUSTZEN_ADMIN_SQLITE_PATH: "./data/db/admin.db", RUSTZEN_ADMIN_HOST: "127.0.0.1", RUSTZEN_ADMIN_PORT: String(port), RUST_LOG: "warn" });
const result = { source, binarySha256: sha(binary), browserPath: "Playwright-owned Chromium", checks: [], screenshots: [], console: [], cleanup: {}, status: "running" };
const delay = (ms) => new Promise((r) => setTimeout(r, ms));
let child, browser, page, ownerToken, readerToken;
let injected = false;
const requests = [];
const password = `D04-${randomBytes(14).toString("hex")}!`;
const fixtureRoles = [];
const fixtureUsers = [];
let readerRole;
const persist = () => writeFileSync(resolve(output, "result.json"), JSON.stringify(result, null, 2) + "\n");
async function freePort() {
    await new Promise((ok, no) => { const s = createServer(); s.once("error", no); s.listen(port, "127.0.0.1", () => s.close(ok)); });
}
async function start() {
    await freePort();
    const fd = openSync(resolve(output, "admin.log"), "a");
    child = spawn(binary, ["serve"], { cwd: root, env, stdio: ["ignore", fd, fd] }); closeSync(fd);
    let spawnError; child.once("error", (e) => { spawnError = e; });
    for (let i = 0; i < 150; i++) {
        if (spawnError) throw spawnError;
        assert(child.exitCode === null && child.signalCode === null, "Owned Admin exited before health");
        try { const r = await fetch(`${base}/health`, { signal: AbortSignal.timeout(1000) }); if (r.ok) return; } catch {}
        await delay(100);
    }
    throw new Error("Owned Admin startup timed out");
}
async function stop() {
    if (!child || child.exitCode !== null || child.signalCode !== null) return;
    await new Promise((ok) => { const t = setTimeout(() => child.kill("SIGKILL"), 5000); child.once("exit", () => { clearTimeout(t); ok(); }); child.kill("SIGTERM"); });
}
async function api(path, { method = "GET", body, token = ownerToken, expected = 200 } = {}) {
    const r = await fetch(base + path, { method, signal: AbortSignal.timeout(10000), headers: { "content-type": "application/json", ...(token ? { authorization: `Bearer ${token}` } : {}) }, ...(body === undefined ? {} : { body: JSON.stringify(body) }) });
    assert.equal(r.status, expected, `${method} ${path}: unexpected HTTP status`);
    const data = await r.json();
    if (expected === 200) assert.equal(data.code, 0, `${method} ${path}: business failure`);
    return data;
}
const roles = () => api("/api/system/roles?pageSize=100&current=1");
async function makeRole(name, code, status, menuIds) {
    const body = { name, code, status, menuIds, description: "Owned D-04 browser fixture" };
    await api("/api/system/roles", { method: "POST", body });
    const row = (await roles()).data.find((r) => r.code === code); assert(row, code);
    fixtureRoles.push({ ...row, body }); return fixtureRoles.at(-1);
}
async function makeUser(username, role) {
    await api("/api/system/users", { method: "POST", body: { username, email: `${username}@example.invalid`, password, realName: "D04 fixture", status: 1, roleIds: [role.id] } });
    const row = (await api(`/api/system/users?username=${username}&pageSize=100`)).data.find((u) => u.username === username);
    assert(row); fixtureUsers.push(row); return row;
}
async function login(p, username, secret) {
    await p.goto(`${base}/login`);
    await p.locator("#login_username").fill(username);
    await p.locator("#login_password").fill(secret);
    const response = p.waitForResponse((r) => new URL(r.url()).pathname === "/api/auth/login" && r.request().method() === "POST");
    await p.locator("#login_password").press("Enter");
    const payload = await (await response).json(); assert.equal(payload.code, 0);
    await p.waitForURL((url) => url.pathname !== "/login");
    return payload.data.token;
}
const keyword = (p = page) => p.getByRole("textbox", { name: /^(Role name or code|角色名称或编码)$/ });
const status = (p = page) => p.getByRole("combobox", { name: /^(Role status|角色状态)$/ });
function waitList(p, predicate = () => true) {
    const response = p.waitForResponse((r) => { const u = new URL(r.url()); return u.pathname === "/api/system/roles" && r.request().method() === "GET" && predicate(u.searchParams, r); }, { timeout: 15000 });
    // Keep rejection observed when the triggering action fails first; awaiting still throws.
    void response.catch(() => {});
    return response;
}
async function openRoles(p = page) {
    const pending = waitList(p, (_, r) => r.status() === 200);
    await p.goto(`${base}/system/role`); await pending;
    await keyword(p).waitFor();
    await p.locator(".ant-table-tbody tr[data-row-key]").first().waitFor();
}
async function search(value, p = page) {
    const pending = waitList(p, (s, r) => (s.get("keyword") ?? "") === value.trim() && r.status() === 200);
    await keyword(p).fill(value); const response = await pending; return response.json();
}
async function chooseStatus(label, value) {
    const pending = waitList(page, (s, r) => s.get("status") === value && r.status() === 200);
    await status().click();
    // Ant Design exposes hidden virtual-list ARIA options alongside visible items.
    await page.locator(".ant-select-dropdown:visible").getByText(label, { exact: true }).click();
    return (await pending).json();
}
async function shot(name, p = page) {
    const file = `${name}.png`; await p.screenshot({ path: resolve(output, file), animations: "disabled" });
    result.screenshots.push({ file, sha256: sha(resolve(output, file)), url: p.url(), viewport: p.viewportSize() });
}
async function check(name, fn) {
    try { await fn(); result.checks.push({ name, status: "passed" }); }
    catch (error) {
        result.checks.push({ name, status: "failed", error: String(error.message).slice(0, 1600) });
        if (page) { await shot(`failure-${name}`).catch(() => {}); }
    }
    console.log(JSON.stringify(result.checks.at(-1))); persist();
}
const db = (sql) => JSON.parse(execFileSync("python3", ["-c", "import sqlite3,json,sys; c=sqlite3.connect('file:'+sys.argv[1]+'?mode=ro',uri=True); c.row_factory=sqlite3.Row; print(json.dumps([dict(r) for r in c.execute(sys.argv[2])]))", resolve(runtime, "data/db/admin.db"), sql], { encoding: "utf8" }));
try {
    await start();
    ownerToken = (await api("/api/auth/login", { method: "POST", token: null, body: { username: "owner", password: "rustzen@123" } })).data.token;
    const options = (await api("/api/system/menus/options")).data;
    const permission = (code) => { const value = options.find((x) => x.code === code)?.value; assert(value, code); return value; };
    const readRole = permission("system:role:list"), readUser = permission("system:user:list");
    for (const [name, code, enabled] of [["D04 Needle by name", "d04_alpha", 1], ["D04 Other active", "d04_needle_code", 1], ["D04 Other inactive", "d04_needle_disabled", 2], ["D04 Needle inactive", "d04_beta", 2], ["D04 literal %_\\' 中文", "d04_literal", 1]]) await makeRole(name, code, enabled, [readUser]);
    for (let i = 0; i < 15; i++) await makeRole(`D04 filler ${i}`, `d04_filler_${i}`, 1, [readUser]);
    readerRole = await makeRole("D04 Reader", "d04_reader_role", 1, [readRole]);
    const nonReaderRole = await makeRole("D04 Nonreader", "d04_nonreader_role", 1, [readUser]);
    const reader = await makeUser("d04_reader", readerRole);
    await makeUser("d04_nonreader", nonReaderRole);
    assert.equal((await api("/api/system/modules")).data.length, 0, "Expected core-only binary");
    browser = await chromium.launch({ headless: true }); result.browserVersion = browser.version();
    const context = await browser.newContext({ viewport: { width: 1920, height: 1080 } });
    page = await context.newPage(); page.setDefaultTimeout(12000);
    await page.clock.install();
    page.on("request", (r) => { const u = new URL(r.url()); if (u.pathname === "/api/system/roles" && r.method() === "GET") requests.push({ ...Object.fromEntries(u.searchParams), injected }); });
    page.on("pageerror", (e) => result.console.push({ type: "pageerror", message: e.message }));
    page.on("console", (m) => { if (["error", "warning"].includes(m.type())) result.console.push({ type: m.type(), message: m.text().slice(0, 600), injected }); });
    await login(page, "owner", "rustzen@123");
    await check("desktop-pagination-status", async () => {
        await openRoles(); assert.equal(new URL(page.url()).pathname, "/system/role");
        assert((await page.title()).includes("Rustzen"));
        assert.equal(await page.locator("vite-error-overlay").count(), 0);
        const next = waitList(page, (s) => s.get("current") === "2");
        await page.locator(".ant-pagination-item-2").click(); await next;
        const startIndex = requests.length;
        const data = await search("  NeEdLe  "); assert.equal(data.total, 4);
        assert(requests.slice(startIndex).every((q) => q.current === "1"));
        assert.equal(await keyword().evaluate((e) => document.activeElement === e), true, "Search retains focus");
        const enabled = await chooseStatus("Enabled", "1"); assert.equal(enabled.total, 2);
        assert(enabled.data.every((r) => r.status === 1));
        assert.deepEqual(enabled.data.map((r) => r.code).sort(), ["d04_alpha", "d04_needle_code"]);
        await shot("desktop-filtered");
        const disabled = await chooseStatus("Disabled", "2"); assert.equal(disabled.total, 2);
        assert(disabled.data.every((r) => r.status === 2));
    });
    await check("debounce-clear-composition", async () => {
        await openRoles(); await page.clock.pauseAt(new Date(Date.now() + 1000));
        try {
            const before = requests.length;
            await keyword().fill("needle"); await delay(30);
            await page.clock.runFor(299); assert.equal(requests.length, before, "No query before 300 ms");
            const pending = waitList(page, (s) => s.get("keyword") === "needle");
            await page.clock.runFor(1); await pending;
            const clear = waitList(page, (s) => !s.has("keyword"));
            // Clearing executes without advancing the paused browser clock.
            await keyword().locator("..").locator(".ant-input-clear-icon").click({ force: true }); await clear;
            const baseline = requests.length;
            await keyword().dispatchEvent("compositionstart", { data: "中" });
            await keyword().fill("中文"); await delay(30); await page.clock.runFor(1000);
            assert.equal(requests.length, baseline, "Composition does not query");
            await keyword().dispatchEvent("compositionend", { data: "中文" }); await delay(30);
            const composed = waitList(page, (s) => s.get("keyword") === "中文");
            await page.clock.runFor(300); const payload = await (await composed).json(); assert.equal(payload.total, 1);
            result.compositionBoundary = "Synthetic DOM composition events in Chromium; no operating-system IME claim";
        } finally { await page.clock.resume(); }
    });
    await check("initial-error-retry-empty", async () => {
        await openRoles(); injected = true;
        const match = (u) => u.pathname === "/api/system/roles" && u.searchParams.get("keyword") === "d04_no_match";
        const handler = (r) => r.fulfill({ status: 503, contentType: "application/json", body: JSON.stringify({ code: 40001, message: "D04 injected list failure", data: null }) });
        await page.route(match, handler);
        try {
            await keyword().fill("d04_no_match");
            await page.getByText("Failed to load roles", { exact: true }).waitFor();
            assert.equal(await page.getByText("No roles", { exact: true }).count(), 0);
            assert.equal(await keyword().inputValue(), "d04_no_match");
            await shot("initial-error");
            await page.unroute(match, handler); injected = false;
            const retry = waitList(page, (s, r) => s.get("keyword") === "d04_no_match" && r.status() === 200);
            await page.getByRole("button", { name: "Reload", exact: true }).click();
            assert.equal((await (await retry).json()).total, 0);
            await page.getByText("No roles", { exact: true }).waitFor();
            assert.equal(await keyword().inputValue(), "d04_no_match");
        } finally { await page.unroute(match, handler); injected = false; }
    });
    await check("cached-error-visible-retry", async () => {
        await openRoles(); await search("needle"); await search("Other");
        injected = true;
        const match = (u) => u.pathname === "/api/system/roles" && u.searchParams.get("keyword") === "needle";
        const handler = (r) => r.fulfill({ status: 503, contentType: "application/json", body: JSON.stringify({ code: 40001, message: "D04 injected cached failure", data: null }) });
        await page.route(match, handler);
        try {
            await keyword().fill("needle");
            await page.getByText("Background refresh failed. The last successfully loaded data remains visible.", { exact: true }).waitFor();
            assert.equal(await keyword().inputValue(), "needle");
            assert.equal(await page.locator(".ant-table-tbody tr[data-row-key]").count(), 4);
            await shot("cached-refresh-error");
            await page.unroute(match, handler); injected = false;
            const recovered = waitList(page, (s, r) => s.get("keyword") === "needle" && r.status() === 200);
            await page.getByRole("button", { name: "Retry", exact: true }).click(); await recovered;
            await page.getByText("Background refresh failed. The last successfully loaded data remains visible.", { exact: true }).waitFor({ state: "hidden" });
            assert.equal(await keyword().inputValue(), "needle");
        } finally { await page.unroute(match, handler); injected = false; }
    });
    await check("bilingual-narrow", async () => {
        await openRoles(); await page.getByRole("button", { name: "Switch to Chinese", exact: true }).click();
        await page.getByRole("textbox", { name: "角色名称或编码", exact: true }).waitFor();
        assert.equal((await search("中文")).total, 1);
        await page.setViewportSize({ width: 390, height: 844 });
        await page.waitForTimeout(200);
        const b = await keyword().boundingBox(); assert(b && b.x >= 0 && b.x + b.width <= 390);
        assert.equal(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth), false, "No document overflow");
        await shot("narrow-chinese");
        await page.reload(); await page.getByRole("textbox", { name: "角色名称或编码", exact: true }).waitFor();
        await page.getByRole("button", { name: "切换为英文", exact: true }).click();
        await page.getByRole("textbox", { name: "Role name or code", exact: true }).waitFor();
        assert.equal((await search("d04_literal")).total, 1);
        await shot("narrow-english"); await page.setViewportSize({ width: 1920, height: 1080 });
    });
    await check("live-identity-revocation-audit", async () => {
        const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } });
        const p = await ctx.newPage();
        try {
            readerToken = await login(p, reader.username, password);
            await openRoles(p); assert.equal((await search("needle", p)).total, 4);
            assert.equal(await p.getByRole("button", { name: "New role", exact: true }).count(), 0);
            const before = (await roles()).total;
            await api("/api/system/roles", { method: "POST", token: readerToken, body: { name: "Forbidden", code: "d04_forbidden", status: 1, menuIds: [readRole] }, expected: 403 });
            assert.equal((await roles()).total, before);
            await api(`/api/system/roles/${readerRole.id}`, { method: "PUT", body: { ...readerRole.body, menuIds: [readUser] } });
            await api("/api/system/roles?keyword=needle", { token: readerToken, expected: 403 });
            const revokedResponse = waitList(p, (_, r) => r.status() === 403);
            await p.reload(); await revokedResponse;
            // An already-open route may present its explicit load error rather than redirect.
            await p.getByText("Failed to load roles", { exact: true }).or(p.getByText("Access denied", { exact: true })).first().waitFor();
            assert.equal(await p.locator(".ant-table-tbody tr[data-row-key]").count(), 0, "Revoked view must not retain role rows");
            await shot("reader-permission-denied", p);
            await api(`/api/system/roles/${readerRole.id}`, { method: "PUT", body: readerRole.body });
            await api("/api/system/roles?keyword=needle", { token: readerToken });
            const deniedToken = (await api("/api/auth/login", { method: "POST", token: null, body: { username: "d04_nonreader", password } })).data.token;
            await api("/api/system/roles?keyword=needle", { token: deniedToken, expected: 403 });
            await api("/api/system/roles?keyword=needle", { token: null, expected: 401 });
            const audit = db("SELECT description,status FROM operation_logs WHERE username='d04_reader' AND description LIKE '%/api/system/roles%'");
            for (const text of ["GET /api/system/roles - 200", "GET /api/system/roles - 403", "POST /api/system/roles - 403"]) assert(audit.some((a) => a.description === text), `Missing audit: ${text}`);
            result.audit = audit;
        } finally {
            // Restore owned fixture authority even when a UI assertion fails.
            try { await api(`/api/system/roles/${readerRole.id}`, { method: "PUT", body: readerRole.body }); }
            finally { await ctx.close(); }
        }
    });
    await check("restart-persistence-cleanup", async () => {
        const before = (await roles()).data.map((r) => r.id).sort((a,b) => a-b);
        await stop(); await start();
        ownerToken = (await api("/api/auth/login", { method: "POST", token: null, body: { username: "owner", password: "rustzen@123" } })).data.token;
        const fresh = (await api("/api/auth/login", { method: "POST", token: null, body: { username: reader.username, password } })).data;
        assert.equal(fresh.userInfo.id, reader.id);
        assert.deepEqual((await roles()).data.map((r) => r.id).sort((a,b) => a-b), before);
        assert.equal((await api("/api/system/roles?keyword=needle", { token: fresh.token })).total, 4);
        for (const user of fixtureUsers) await api(`/api/system/users/${user.id}`, { method: "DELETE" });
        for (const role of fixtureRoles) await api(`/api/system/roles/${role.id}`, { method: "DELETE" });
        assert.equal((await roles()).total, 3);
        assert.equal((await api("/api/system/users?pageSize=100")).total, 1);
        result.cleanup.businessRecords = "Only the owner and three built-in roles remain";
    });
    assert.equal(result.console.filter((e) => e.type === "pageerror").length, 0, "Unexpected page exception");
    result.requests = requests;
    result.status = result.checks.some((c) => c.status === "failed") ? "failed" : "passed";
} catch (error) { result.status = "failed"; result.fatal = String(error.message).slice(0, 1600); console.error(result.fatal); }
finally {
    await browser?.close().catch(() => {}); await stop();
    try { await freePort(); result.cleanup.listener = "closed"; } catch { result.cleanup.listener = "not verified"; result.status = "failed"; }
    rmSync(runtime, { recursive: true, force: true }); result.cleanup.ownedRuntime = "removed";
    result.requests = requests;
    persist();
}
if (result.status !== "passed") process.exitCode = 1;
