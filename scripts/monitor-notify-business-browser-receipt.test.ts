import { expect, test } from "bun:test";

const parser = await Bun.file(new URL("./monitor-notify-business-browser-receipt.ts", import.meta.url)).text();
const driver = await Bun.file(new URL("./monitor-notify-business-browser-driver.ts", import.meta.url)).text();

test("P8f-B receipt requires accepted, consecutive witness reports", () => {
    expect(parser).toContain('item.status === "accepted"');
    expect(parser).toContain("item.sequence === api.deterministicReports[index - 1].sequence + 1");
    expect(parser).toContain("ui.monitorNode.nodeId === value.witnessAgent.nodeId");
    expect(driver).toContain("witnessNode.sequence+1");
    expect(driver).toContain("witness report was not accepted");
});

test("P8f-B stops the verified witness before issuing business reports", () => {
    expect(driver.indexOf("witness Agent did not stop")).toBeLessThan(driver.indexOf("const reportReceipts"));
    expect(driver).toContain("sha256(witnessArtifact.bytes)");
});

test("P8f-B diagnostics never echo evaluated expressions or wait unbounded for Chrome", async () => {
    const wrapper = await Bun.file(new URL("./verify-monitor-notify-business-browser-linux.sh", import.meta.url)).text();
    expect(driver).not.toContain("expression.slice");
    expect(driver).not.toContain("CDP evaluate failed: ${detail}");
    expect(driver).not.toContain("value.error.message");
    expect(driver).toContain("Page.addScriptToEvaluateOnNewDocument");
    expect(driver).not.toContain("Network.requestWillBeSentExtraInfo");
    expect(driver).toContain("streamDiagnostics(streamItems)");
    expect(driver).toContain("wallTime: value.params.wallTime");
    expect(driver).toContain('"subjectRevision"');
    expect(driver).toContain("detailDiagnostics");
    expect(driver).toContain("readAtPresent");
    expect(driver).toContain("sseFetchProbeSource()")
    expect(driver).toContain('throw Error("browser document generation did not change")');
    expect(driver.indexOf('await wait(".shell-content"); await call("Page.addScriptToEvaluateOnNewDocument"')).toBeGreaterThan(driver.indexOf("document.querySelector('button[type=submit]').click()"));
    expect(driver.indexOf('await call("Page.reload"); await waitDocument(firstDocument); await wait(".shell-content");')).toBeGreaterThan(driver.indexOf("Page.addScriptToEvaluateOnNewDocument"));
    expect(driver).toContain('await call("Page.reload"); await waitDocument(finalDocument); await wait(".shell-content");');
    expect(driver).toContain("[aria-label='Message details']");
    expect(wrapper).toContain('kill -KILL "$chrome_pid"');
    expect(wrapper).toContain("for _ in $(seq 1 100)");
});

test("P8f-B locates the selected notification shell in either supported locale", () => {
    expect(driver).toContain('localStorage.setItem("rustzen-admin-locale","en-US"); location.reload()');
    expect(driver).toContain("button[aria-label='Open message center'],button[aria-label='打开消息中心']");
    expect(driver).toContain("message center control is ambiguous");
});
