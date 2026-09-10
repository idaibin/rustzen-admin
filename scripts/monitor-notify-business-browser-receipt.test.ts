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
    expect(wrapper).toContain('kill -KILL "$chrome_pid"');
    expect(wrapper).toContain("for _ in $(seq 1 100)");
});
