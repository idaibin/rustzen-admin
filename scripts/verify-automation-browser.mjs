import { createHmac, randomUUID } from "node:crypto";
import { createServer } from "node:http";

const reportsPort = Number(process.env.RUSTZEN_REPORTS_PORT);
const fixturePort = Number(process.env.RUSTZEN_AUTOMATION_FIXTURE_PORT);
const token = process.env.RUSTZEN_IPC_TOKEN;
if (!reportsPort || !fixturePort || !token) throw new Error("Automation verification environment is incomplete");
const reportsBase = `http://127.0.0.1:${reportsPort}`;
const verifierId = "1";
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
  response.writeHead(200, { "content-type": "text/html" });
  response.end(`<form method="post"><input id="title" name="title"><button id="submit" type="submit">Submit</button></form>`);
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

try {
  const system = await call("/api/reports/systems", "reports:system:manage", "POST", { name: "Browser fixture", baseUrl: `http://127.0.0.1:${fixturePort}` });
  const flow = await call("/api/reports/flows", "reports:flow:manage", "POST", { systemId: system.id, name: "Real form submission", steps: [
    { action: "goto", url: "/" },
    { action: "fill", selector: "#title", value: "{{input.title}}" },
    { action: "click", selector: "#submit" },
    { action: "waitFor", selector: "#received" },
    { action: "assertText", selector: "#received", text: "RustZen+MVP" },
    { action: "screenshot", name: "submitted" },
  ] });
  const run = await call("/api/reports/runs", "reports:run:manage", "POST", { flowId: flow.id, input: { title: "RustZen MVP" } });
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
  if (!submitted.includes("title=RustZen+MVP")) throw new Error(`Fixture received unexpected form: ${submitted}`);
  const steps = await call(`/api/reports/runs/${run.id}/steps`, "reports:run:view");
  if (steps.length !== 6 || steps.some(step => step.status !== "succeeded")) throw new Error(`Unexpected step audit: ${JSON.stringify(steps)}`);
  const artifacts = await call(`/api/reports/runs/${run.id}/artifacts`, "reports:run:view");
  if (!artifacts.some(artifact => artifact.kind === "screenshot") || !artifacts.some(artifact => artifact.kind === "live-frame")) {
    throw new Error(`Expected screenshot and live-frame artifacts: ${JSON.stringify(artifacts)}`);
  }
  const livePath = `/api/reports/runs/${run.id}/live-frame`;
  const liveResponse = await fetch(`${reportsBase}${livePath}`, { headers: headers(livePath, "reports:run:view") });
  if (!liveResponse.ok || !liveResponse.headers.get("content-type")?.startsWith("image/png") || (await liveResponse.arrayBuffer()).byteLength === 0) {
    throw new Error(`Live frame endpoint failed: ${liveResponse.status}`);
  }

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
  console.log(
    `Reports browser verification passed: run=${run.id}, steps=${steps.length}, artifacts=${artifacts.length}, cancelled=${cancellationRun.id}`,
  );
} finally {
  await new Promise(resolve => fixture.close(resolve));
}
