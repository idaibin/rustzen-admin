import { createHmac, randomUUID } from "node:crypto";
import { createServer } from "node:http";
import { readdir } from "node:fs/promises";
import { join } from "node:path";
import { execFileSync } from "node:child_process";

const reportsPort = Number(process.env.RUSTZEN_REPORTS_PORT);
const fixturePort = Number(process.env.RUSTZEN_AUTOMATION_FIXTURE_PORT);
const token = process.env.RUSTZEN_IPC_TOKEN;
if (!reportsPort || !fixturePort || !token) throw new Error("Automation verification environment is incomplete");
const reportsBase = `http://127.0.0.1:${reportsPort}`;
const verifierId = "1";
const expectedTimezone = process.env.RUSTZEN_EXPECT_TIMEZONE;
let submitted = "";
const fixture = createServer((request, response) => {
  if (request.method === "POST") {
    const chunks = [];
    request.on("data", chunk => chunks.push(chunk));
    request.on("end", () => {
      submitted = Buffer.concat(chunks).toString();
      response.writeHead(200, { "content-type": "text/html" });
      response.end(`<main id="received">Received ${submitted}</main>`);
    });
    return;
  }
  if (request.url === "/tall") {
    response.writeHead(200, { "content-type": "text/html" });
    response.end('<main style="width:2000px;height:2500px">Too tall to capture</main>');
    return;
  }
  response.writeHead(200, { "content-type": "text/html" });
  response.end(`<form method="post">
    <input id="title" name="title">
    <textarea id="notes" name="notes"></textarea>
    <select id="choice" name="choice"><option value="alpha">Alpha</option><option value="beta">Beta</option></select>
    <output id="events"></output>
    <button id="submit" type="submit">Submit</button>
  </form>
  <script>
    for (const id of ["title", "notes", "choice"]) {
      for (const type of ["input", "change"]) {
        document.getElementById(id).addEventListener(type, () => {
          document.getElementById("events").textContent += id + ":" + type + ";";
        });
      }
    }
  </script>`);
});
await new Promise(resolve => fixture.listen(fixturePort, "127.0.0.1", resolve));

function headers(path, capability, method = "GET") {
  const timestamp = Math.floor(Date.now() / 1000).toString();
  const requestId = randomUUID();
  const payload = ["1", timestamp, requestId, verifierId, "reports", method, path, capability].join("\n");
  return {
    "content-type": "application/json",
    "x-rustzen-contract-version": "1",
    "x-rustzen-ipc-timestamp": timestamp,
    "x-rustzen-request-id": requestId,
    "x-rustzen-user-id": verifierId,
    "x-rustzen-module": "reports",
    "x-rustzen-ipc-capability": capability,
    "x-rustzen-ipc-signature": createHmac("sha256", token).update(payload).digest("hex"),
  };
}
async function call(path, capability, method = "GET", body) {
  const response = await fetch(`${reportsBase}${path}`, { method, headers: headers(new URL(`${reportsBase}${path}`).pathname, capability, method), ...(body ? { body: JSON.stringify(body) } : {}) });
  if (!response.ok) throw new Error(`${method} ${path}: ${response.status} ${await response.text()}`);
  const payload = await response.json();
  return payload.data;
}
async function expectStatus(path, capability, expectedStatus, method = "GET", body) {
  const response = await fetch(`${reportsBase}${path}`, { method, headers: headers(new URL(`${reportsBase}${path}`).pathname, capability, method), ...(body ? { body: JSON.stringify(body) } : {}) });
  if (response.status !== expectedStatus) {
    throw new Error(`${method} ${path} with ${capability}: expected ${expectedStatus}, got ${response.status}: ${await response.text()}`);
  }
  return response;
}

try {
  if (expectedTimezone) {
    const settings = await call("/api/reports/settings", "reports:schedule:view");
    if (settings.timezone !== expectedTimezone) {
      throw new Error(`Reports timezone mismatch: expected ${expectedTimezone}, got ${settings.timezone}`);
    }
  }
  const system = await call("/api/reports/systems", "reports:system:manage", "POST", { name: "Browser fixture", baseUrl: `http://127.0.0.1:${fixturePort}` });
  const flow = await call("/api/reports/flows", "reports:flow:manage", "POST", { systemId: system.id, name: "Real form submission", steps: [
    { action: "goto", url: "/" },
    { action: "guardExists", selector: "#nonexistent-modal", onMissing: "skipNext" },
    { action: "click", selector: "#nonexistent-modal" },
    { action: "fill", selector: "#title", value: "{{input.title}}" },
    { action: "fill", selector: "#notes", value: "controlled textarea" },
    { action: "fill", selector: "#choice", value: "beta" },
    { action: "assertText", selector: "#events", text: "title:input;title:change;notes:input;notes:change;choice:input;choice:change;" },
    { action: "pause", durationMs: 100 },
    { action: "click", selector: "#submit" },
    { action: "waitFor", selector: "#received" },
    { action: "assertText", selector: "#received", text: "Rustzen+MVP" },
    { action: "screenshot", name: "submitted" },
  ] });
  const scheduleOnlyInput = { flowId: flow.id, cadence: "daily", weekday: null, dueTime: "23:59", input: {}, enabled: false };
  await expectStatus("/api/reports/schedules", "reports:schedule:view", 200);
  await expectStatus("/api/reports/flow-options", "reports:schedule:view", 200);
  await expectStatus("/api/reports/schedules", "reports:schedule:view", 403, "POST", scheduleOnlyInput);
  const scheduleManager = await call("/api/reports/schedules", "reports:schedule:manage", "POST", scheduleOnlyInput);
  await expectStatus(`/api/reports/schedules/${scheduleManager.id}`, "reports:schedule:manage", 200, "DELETE");
  console.log("Schedule-only viewer/manager delegated permissions passed");
  const run = await call("/api/reports/runs", "reports:run:manage", "POST", { flowId: flow.id, input: { title: "Rustzen MVP" } });
  let current;
  for (let attempt = 0; attempt < 600; attempt += 1) {
    current = await call(`/api/reports/runs/${run.id}`, "reports:run:view");
    if (!["queued", "running", "cancelling"].includes(current.status)) break;
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  if (current?.status !== "succeeded") {
    const failedSteps = await call(`/api/reports/runs/${run.id}/steps`, "reports:run:view");
    throw new Error(`Browser run failed: ${JSON.stringify(current)}, steps=${JSON.stringify(failedSteps)}`);
  }
  if (!submitted.includes("title=Rustzen+MVP&notes=controlled+textarea&choice=beta")) {
    throw new Error(`Fixture received unexpected form: ${submitted}`);
  }
  const steps = await call(`/api/reports/runs/${run.id}/steps`, "reports:run:view");
  if (steps.length !== 12) throw new Error(`Unexpected step count: ${steps.length}`);
  if (steps[2].status !== "skipped") throw new Error(`Expected step 2 to be skipped: ${JSON.stringify(steps[2])}`);
  if (steps.filter(step => step.status !== "skipped").some(step => step.status !== "succeeded")) {
    throw new Error(`Unexpected step audit: ${JSON.stringify(steps)}`);
  }
  const artifacts = await call(`/api/reports/runs/${run.id}/artifacts`, "reports:run:view");
  if (!artifacts.some(artifact => artifact.kind === "screenshot") || !artifacts.some(artifact => artifact.kind === "live-frame")) {
    throw new Error(`Expected screenshot and live-frame artifacts: ${JSON.stringify(artifacts)}`);
  }
  const livePath = `/api/reports/runs/${run.id}/live-frame`;
  const liveResponse = await fetch(`${reportsBase}${livePath}`, { headers: headers(livePath, "reports:run:view") });
  if (!liveResponse.ok || !liveResponse.headers.get("content-type")?.startsWith("image/png") || (await liveResponse.arrayBuffer()).byteLength === 0) {
    throw new Error(`Live frame endpoint failed: ${liveResponse.status}`);
  }

  const tallFlow = await call("/api/reports/flows", "reports:flow:manage", "POST", {
    systemId: system.id,
    name: "Tall screenshot rejection",
    steps: [
      { action: "goto", url: "/tall" },
      { action: "screenshot", name: "too-tall" },
    ],
  });
  const tallRun = await call("/api/reports/runs", "reports:run:manage", "POST", {
    flowId: tallFlow.id,
    input: {},
  });
  let tallResult;
  for (let attempt = 0; attempt < 200; attempt += 1) {
    tallResult = await call(`/api/reports/runs/${tallRun.id}`, "reports:run:view");
    if (!["queued", "running", "cancelling"].includes(tallResult.status)) break;
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  if (tallResult?.status !== "failed" || !tallResult.error?.includes("screenshot exceeds pixel limit")) {
    throw new Error(`Tall screenshot was not rejected by pixel preflight: ${JSON.stringify(tallResult)}`);
  }
  const tallArtifacts = await call(`/api/reports/runs/${tallRun.id}/artifacts`, "reports:run:view");
  if (tallArtifacts.some(artifact => artifact.kind === "screenshot")) {
    throw new Error(`Tall screenshot persisted an artifact: ${JSON.stringify(tallArtifacts)}`);
  }
  const tallFiles = await readdir(join(process.env.RUSTZEN_RUNTIME_ROOT, "data", "reports", tallRun.id));
  if (tallFiles.some(name => name.endsWith(".tmp") || name.startsWith("too-tall-") || name.startsWith("browser-"))) {
    throw new Error(`Tall screenshot left a temporary, screenshot, or profile file: ${JSON.stringify(tallFiles)}`);
  }
  console.log(`Tall screenshot preflight rejection passed: ${tallRun.id}`);

  submitted = "";
  const cancellationFlow = await call("/api/reports/flows", "reports:flow:manage", "POST", {
    systemId: system.id,
    name: "Cancellation stops later actions",
    steps: [
      { action: "goto", url: "/" },
      { action: "waitFor", selector: "#never-appears" },
      { action: "click", selector: "#submit" },
    ],
  });
  const cancellationRun = await call("/api/reports/runs", "reports:run:manage", "POST", {
    flowId: cancellationFlow.id,
    input: {},
  });
  let cancellationCurrent;
  for (let attempt = 0; attempt < 100; attempt += 1) {
    cancellationCurrent = await call(`/api/reports/runs/${cancellationRun.id}`, "reports:run:view");
    if (cancellationCurrent.status === "running") break;
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  if (cancellationCurrent?.status !== "running") {
    throw new Error(`Cancellation run did not start: ${JSON.stringify(cancellationCurrent)}`);
  }
  await new Promise(resolve => setTimeout(resolve, 300));
  const cancelling = await call(
    `/api/reports/runs/${cancellationRun.id}/cancel`,
    "reports:run:manage",
    "POST",
  );
  if (cancelling.status !== "cancelling") {
    throw new Error(`Running cancellation was prematurely terminal: ${JSON.stringify(cancelling)}`);
  }
  for (let attempt = 0; attempt < 100; attempt += 1) {
    cancellationCurrent = await call(`/api/reports/runs/${cancellationRun.id}`, "reports:run:view");
    if (cancellationCurrent.status === "cancelled") break;
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  const cancellationSteps = await call(
    `/api/reports/runs/${cancellationRun.id}/steps`,
    "reports:run:view",
  );
  const laterActionRan = cancellationSteps.some(
    step => step.action === "click" && step.status === "succeeded",
  );
  if (
    cancellationCurrent?.status !== "cancelled" ||
    laterActionRan ||
    submitted !== ""
  ) {
    throw new Error(
      `Cancellation did not stop execution: run=${JSON.stringify(cancellationCurrent)}, steps=${JSON.stringify(cancellationSteps)}, submitted=${submitted}`,
    );
  }
  const concurrentRuns = await Promise.all([1, 2].map(() => call(
    "/api/reports/runs", "reports:run:manage", "POST",
    { flowId: flow.id, input: { title: "Rustzen MVP" } },
  )));
  const concurrentResults = await Promise.all(concurrentRuns.map(async concurrentRun => {
    let latest;
    for (let attempt = 0; attempt < 600; attempt += 1) {
      latest = await call(`/api/reports/runs/${concurrentRun.id}`, "reports:run:view");
      if (!["queued", "running", "cancelling"].includes(latest.status)) break;
      await new Promise(resolve => setTimeout(resolve, 100));
    }
    if (latest?.status !== "succeeded") throw new Error(`Concurrent run failed: ${JSON.stringify(latest)}`);
    return latest;
  }));
  if (Math.max(...concurrentResults.map(item => Date.parse(item.startedAt))) >=
      Math.min(...concurrentResults.map(item => Date.parse(item.finishedAt)))) {
    throw new Error("Concurrent runs did not overlap");
  }
  const runtimeRoot = process.env.RUSTZEN_RUNTIME_ROOT;
  if (!runtimeRoot) throw new Error("Runtime root required for profile cleanup verification");
  // This verifier owns a fresh disposable database. Shorten only its run budget
  // to exercise process/profile cleanup when the overall deadline expires.
  execFileSync("sqlite3", [join(runtimeRoot, "data", "reports", "db", "reports.db"),
    "PRAGMA busy_timeout=5000; UPDATE automation_settings SET max_run_timeout_seconds=2;"]);
  const timedRun = await call("/api/reports/runs", "reports:run:manage", "POST", {
    flowId: cancellationFlow.id, input: {},
  });
  let timedResult;
  for (let attempt = 0; attempt < 200; attempt += 1) {
    timedResult = await call(`/api/reports/runs/${timedRun.id}`, "reports:run:view");
    if (!["queued", "running", "cancelling"].includes(timedResult.status)) break;
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  if (timedResult?.status !== "failed" || timedResult.error !== "run timed out") {
    throw new Error(`Run deadline failed: ${JSON.stringify(timedResult)}`);
  }
  for (const completed of [run, cancellationRun, timedRun, ...concurrentRuns]) {
    const files = await readdir(join(runtimeRoot, "data", "reports", completed.id));
    if (files.some(name => name.startsWith("browser-"))) throw new Error(`Browser profile leaked: ${completed.id}`);
  }
  console.log(`Overlapping runs, timeout and profile cleanup passed: ${concurrentRuns.map(item => item.id).join(", ")}, timeout=${timedRun.id}`);
  console.log(
    `Reports browser verification passed: run=${run.id}, steps=${steps.length}, artifacts=${artifacts.length}, cancelled=${cancellationRun.id}`,
  );
} finally {
  await new Promise(resolve => fixture.close(resolve));
}
