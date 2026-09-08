import { afterEach, describe, expect, test } from "bun:test";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { fixture, receipts, run, signal } from "./verify-message-center-browser-linux.test-support.mjs";

const scripts=dirname(fileURLToPath(import.meta.url));const outer=join(scripts,"verify-message-center-browser-linux.sh");const inner=join(scripts,"verify-message-center-browser-linux-inner.sh");const steps=join(scripts,"message-center-browser-steps.mjs");const validator=join(scripts,"verify-message-center-browser-evidence.jq");const fixtures=[];
afterEach(()=>{while(fixtures.length)fixtures.pop().cleanup();});const make=(options)=>{const item=fixture(outer,options);fixtures.push(item);return item;};

describe("message center browser Linux gate",()=>{
    test("binds selected/pure artifacts, real browser states and bounded publication",()=>{
        const outside=readFileSync(outer,"utf8"),inside=readFileSync(inner,"utf8"),matrix=readFileSync(steps,"utf8"),jq=readFileSync(validator,"utf8");
        for(const value of ["monitor-distribution,notifications","rz-monitor-notify","rz-admin-pure","stage-message-center-web.sh","atomic_replace_symlink","remove_container","failed-runs"])expect(outside).toContain(value);
        for(const value of ["selected-sse-preflight","sse-preflight.json","realtimeInvalidation","realtime-timing.json","browser-empty-loading","browser-populated-detail","browser-reconcile-and-auth","forbidden-list-ready","forbidden-timing.json","pure-absence.json","RUSTZEN_NOTIFICATION_INGRESS_PORT=19911","RUSTZEN_REPORTS_NOTIFICATION_INGRESS_URL=http://127.0.0.1:19911","19800 19810 19811 19812 19820 19821 19822 19901 19904 19911","capture_logs","case_diagnostics","{stepIndex,action,status,message}","$prefix-artifact-$index.bin"])expect(inside).toContain(value);
        for(const path of ["$harness/data/reports/db","$harness/logs/reports","$harness/output","$harness/.config","$harness/.cache"])expect(inside).toContain(path);
        expect(inside.match(/as_reports \/verify\/staged\/rz-reports-verifier [^\n]+/g)).toEqual(["as_reports /verify/staged/rz-reports-verifier serve >$harness/logs/reports/verifier.log 2>&1 & pids+=(\"$!\")"]);
        expect(inside).not.toContain("rz-reports-verifier init-db");expect(inside).not.toContain("rz-reports-verifier bind-database");
        expect(inside).toContain("-d '{\"username\":\"owner\",\"password\":\"rustzen@123\"}'");expect(inside).not.toContain("\"username\":\"rustzen\"");
        for(const marker of ["step harness-login","step harness-system-create","step \"run-case-$name\""])expect(inside).toContain(marker);
        for(const value of ["subject:{kind:\"monitor-incident\",id:$subject,revision:1}","date -u -d '6 days'",".status == 201 and .body.code == \"stored\"","run_case forbiddenClears '{\"list403After\":1,\"readyFile\":\"/verify/evidence/forbidden-list-ready\"}' forbidden 22","run_case unauthorized '{\"sse401\":true}'"])expect(inside).toContain(value);
        expect(inside).toContain("report_epoch=$(($(date +%s)-30))");expect(inside).toContain('int(sys.argv[1])+int(sys.argv[2])');
        expect(inside).toContain("select count(*) from monitor_incidents");expect(inside).not.toContain("select count(*) from incidents");
        expect(inside).toContain("owners=('apps/web/src/notifications/','apps/web/src/api/notifications/')");expect(inside).not.toContain("if 'notification' in x.lower()");
        expect(inside).toContain('"/verify/evidence/admission-$delayed.json"');
        expect(inside).toContain("rm -f /verify/evidence/browser-results.jsonl /verify/evidence/proxy.jsonl /verify/evidence/proxy-mode.json");
        for(const value of [".ant-badge-count","Mark all read","realtimeInvalidation","forbiddenClears","incidentDeepLink","message-center-mobile-en"])expect(matrix).toContain(value);
        expect(matrix).toContain("browser-page-21");expect(jq).toContain("$expected[0][$result.case]");expect(jq).toContain("$preflight[0]");expect(jq).toContain("$browserStreams|length == 11");expect(jq).toContain("$realtimeReads");expect(jq).toContain(".status == 403");expect(jq).toContain("tokenInUrl == false");expect(jq).toContain("$files|unique");
        for(const file of [outer,inner,steps,validator])expect(readFileSync(file,"utf8").split("\n").length).toBeLessThan(300);
    });
    test("valid evidence publishes after owned containers are absent",()=>{const item=make(),result=run(item);expect(result.exitCode).toBe(0);expect(item.current()).toMatch(/^runs\//);expect(item.active()).toEqual([]);const manifest=JSON.parse(readFileSync(join(item.evidence,item.current(),"manifest.json")));expect(manifest.receipts.map(({file})=>file).sort()).toEqual([...receipts].sort());expect(item.calls()).toContain("container inspect rz-message-center-runtime-");});
    for(const tamper of ["browser","actions","token","sse-status","unauthorized","browser-sse-missing","browser-sse-duplicate","browser-sse-case","realtime-refetch","forbidden","pre-ready","pure","duplicate","path","badge"])test(`rejects ${tamper} tamper without replacing current`,()=>{const item=make({tamper}),result=run(item);expect(result.exitCode).not.toBe(0);expect(item.current()).toBe("runs/previous");expect(item.failed()).toHaveLength(1);expect(item.active()).toEqual([]);});
    test("runtime failure keeps browser steps, artifacts and container diagnostics",()=>{const item=make({runtimeExit:"7"}),result=run(item),failed=join(item.evidence,"failed-runs",item.failed()[0]);expect(result.exitCode).toBe(7);expect(item.current()).toBe("runs/previous");expect(JSON.parse(readFileSync(join(failed,"failed-case-emptyDesktop-steps.json"),"utf8")).steps).toEqual([{stepIndex:6,action:"waitFor",status:"failed",message:"fake selector failure"}]);expect(existsSync(join(failed,"failed-case-emptyDesktop-artifacts.json"))).toBe(true);expect(existsSync(join(failed,"runtime-container.log"))).toBe(true);expect(item.active()).toEqual([]);});
    test("build timeout returns 124 and cleans resources",()=>{const item=make({block:"build",timeout:"1"}),result=run(item);expect(result.exitCode).toBe(124);expect(item.current()).toBe("runs/previous");expect(item.active()).toEqual([]);});
    test("container cleanup failure cannot publish",()=>{const item=make({rmFail:true}),result=run(item);expect(result.exitCode).not.toBe(0);expect(item.current()).toBe("runs/previous");});
    for(const [name,code] of [["SIGINT",130],["SIGTERM",143]])test(`${name} returns ${code} and retains failure evidence`,async()=>{const item=make({block:"runtime",timeout:"10"});expect(await signal(item,name)).toBe(code);expect(item.current()).toBe("runs/previous");expect(item.failed()).toHaveLength(1);expect(item.active()).toEqual([]);});
});
