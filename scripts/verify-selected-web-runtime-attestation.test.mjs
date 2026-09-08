import { expect, test } from "bun:test";
import { image, inspected, process, same } from "./verify-selected-web-runtime-attestation.ts";
const digest = "a".repeat(64);
const inspect = [{ Id: "c".repeat(64), Image: "i".repeat(64), State: { Running: true }, Platform: "linux", NetworkSettings: { Ports: { "8080/tcp": [{ HostIp: "127.0.0.1", HostPort: "3000" }] } } }];
test("attests a unique certified listener and rejects substitutes", () => {
    const base = inspected(inspect, "127.0.0.1", 3000); image([{ Os: "linux", Architecture: "amd64" }]);
    expect(() => image([{ Os: "linux", Architecture: "arm64" }])).toThrow("amd64");
    const good = process(`41\t1\t2\t${digest}\n`, base, digest);
    expect(() => inspected([{ ...inspect[0], NetworkSettings: { Ports: { "8080/tcp": [{ HostIp: "192.168.1.2", HostPort: "3000" }] } } }], "127.0.0.1", 3000)).toThrow("map uniquely");
    expect(() => inspected([{ ...inspect[0], NetworkSettings: { Ports: { "8080/tcp": [{ HostIp: "::", HostPort: "3000" }] } } }], "127.0.0.1", 3000)).toThrow("map uniquely");
    expect(() => inspected([{ ...inspect[0], NetworkSettings: { Ports: { "8080/tcp": [{ HostIp: "127.0.0.1", HostPort: "3000" }, { HostIp: "0.0.0.0", HostPort: "3000" }] } } }], "127.0.0.1", 3000)).toThrow("map uniquely");
    expect(good.pid).toBe(41); expect(good.containerPort).toBe(8080);
    expect(() => process(`41\t1\t2\t${digest}\n42\t3\t4\t${digest}\n`, base, digest)).toThrow("uniquely");
    expect(() => process(`41\t1\t2\t${"b".repeat(64)}\n`, base, digest)).toThrow("certified");
    expect(same(good, { ...good })).toBe(true); expect(same(good, { ...good, pid: 42 })).toBe(false);
});
