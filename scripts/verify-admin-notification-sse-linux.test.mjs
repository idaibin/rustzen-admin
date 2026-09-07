import { afterEach, describe, expect, test } from "bun:test";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { fixture, receipts, run, signal } from "./verify-admin-notification-sse-linux.test-support.mjs";

const scripts=dirname(fileURLToPath(import.meta.url));
const outer=join(scripts,"verify-admin-notification-sse-linux.sh");
const inner=join(scripts,"verify-admin-notification-sse-linux-inner.sh");
const validator=join(scripts,"verify-admin-notification-sse-evidence.jq");
const fixtures=[];
afterEach(()=>{while(fixtures.length)fixtures.pop().cleanup();});
const make=(options)=>{const item=fixture(outer,options);fixtures.push(item);return item;};

describe("Admin notification SSE Linux gate",()=>{
    test("source fixes selected build, real SSE cases, process ownership and bounded publication",()=>{
        const outerText=readFileSync(outer,"utf8"),innerText=readFileSync(inner,"utf8"),jqText=readFileSync(validator,"utf8");
        for(const value of ["--no-default-features --features monitor-distribution,notifications","failed-runs","atomic_replace_symlink","remove_container","verify-admin-notification-sse-linux-inner.sh"])expect(outerText).toContain(value);
        for(const value of ["initial","Last-Event-ID","live-change","retention","quota-fifth","authority-change","expiry","authority-outage","started_pid=$!"])expect(innerText).toContain(value);
        expect(innerText).toContain("'content':{'title':'Runtime incident opened','summary':'Runtime SSE evidence'}");
        expect(innerText).toContain('capture logout GET "$public/api/auth/logout"');
        expect(innerText).toContain("'operationLogTablePresent':bool(has_operation_logs)");
        for(const value of ["'status':status","'contentType':headers.get('content-type'","'cacheControl':headers.get('cache-control'","'accelBuffering':headers.get('x-accel-buffering'"])expect(innerText).toContain(value);
        for(const value of ["exact_stream($initial[0])","exact_stream($live[0])","exact_stream($reconnect[0])"])expect(jqText).toContain(value);
        expect(innerText).not.toContain("=$(sse_start"); expect(jqText).toContain("($allowlist|sort)");
        expect(outerText.split("\n").length).toBeLessThan(300);expect(innerText.split("\n").length).toBeLessThan(300);expect(jqText.split("\n").length).toBeLessThan(300);
    });

    test("valid evidence publishes only after both containers are absent",()=>{
        const item=make(),result=run(item); expect(result.exitCode).toBe(0); expect(item.current()).toMatch(/^runs\//); expect(item.active()).toEqual([]);
        expect(item.callsText()).toContain("container inspect rz-admin-sse-build-");expect(item.callsText()).toContain("container inspect rz-admin-sse-runtime-");
        const manifest=JSON.parse(readFileSync(join(item.evidence,item.current(),"manifest.json"),"utf8"));expect(manifest.receipts.map(({file})=>file).sort()).toEqual([...receipts].sort());
    });

    for(const tamper of ["revision","replay","authority","redaction","quota","expiry","stream-status","stream-content-type","stream-cache","stream-buffering","duplicate","path"]){
        test(`validator rejects ${tamper} tamper and preserves current`,()=>{const item=make({tamper}),result=run(item);expect(result.exitCode).not.toBe(0);expect(item.current()).toBe("runs/previous");expect(item.failed()).toHaveLength(1);expect(item.active()).toEqual([]);});
    }

    test("runtime failure retains diagnostics without replacing current",()=>{const item=make({runtimeExit:"7"}),result=run(item);expect(result.exitCode).toBe(7);expect(item.current()).toBe("runs/previous");expect(item.failed()).toHaveLength(1);expect(existsSync(join(item.evidence,"failed-runs",item.failed()[0],"runtime-container.log"))).toBe(true);expect(item.active()).toEqual([]);});
    test("build failure retains diagnostics and cleans the build container",()=>{const item=make({buildExit:"8"}),result=run(item);expect(result.exitCode).toBe(8);const failed=join(item.evidence,"failed-runs",item.failed()[0]);expect(readFileSync(join(failed,"phase-status.tsv"),"utf8")).toContain("build\t8");expect(readFileSync(join(failed,"build-container.log"),"utf8").length).toBeGreaterThan(0);expect(item.current()).toBe("runs/previous");expect(item.active()).toEqual([]);});
    test("runtime timeout returns 124 and releases resources",()=>{const item=make({block:"runtime",timeout:"1"}),result=run(item);expect(result.exitCode).toBe(124);expect(item.current()).toBe("runs/previous");expect(item.failed()).toHaveLength(1);expect(item.active()).toEqual([]);});
    test("container removal failure cannot publish",()=>{const item=make({rmFail:true}),result=run(item);expect(result.exitCode).not.toBe(0);expect(item.current()).toBe("runs/previous");expect(item.failed()).toHaveLength(1);});
    for(const [name,code] of [["SIGINT",130],["SIGTERM",143]])test(`${name} returns ${code} and cleans the owned child`,async()=>{const item=make({block:"runtime",timeout:"10"});expect(await signal(item,name)).toBe(code);expect(item.current()).toBe("runs/previous");expect(item.failed()).toHaveLength(1);expect(item.active()).toEqual([]);});
});
