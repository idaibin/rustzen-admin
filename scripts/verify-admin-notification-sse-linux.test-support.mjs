import { appendFileSync, chmodSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, readlinkSync, rmSync, symlinkSync, unlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const inner = ["steps.log","initial.json","event.json","ingest.json","live-change.json","inbox.json","read.json","retention.json","reconnect.json","auth-401.json","auth-403.json","query-rejection.json","redaction.json","quota.json","authority-change.json","expiry.json","authority-outage.json","selected-state.json"];
export const receipts = [...inner,"phase-status.tsv","build.log","verifier-helper.log","runtime.log"];
const hash = (bytes) => new Bun.CryptoHasher("sha256").update(bytes).digest("hex");
const executable = (path, body) => { writeFileSync(path, body); chmodSync(path, 0o755); };
function envValues(args) {
    const values = {};
    for (let i=0;i<args.length;i+=1) if (args[i] === "--env") { const [key,...value]=args[++i].split("="); values[key]=value.join("="); }
    return values;
}
const envelope = (status,code,message="denied",data=null) => ({status,contentType:"application/json",body:{code,message,data}});
const stream = (body) => ({status:200,contentType:"text/event-stream",cacheControl:"no-cache, no-store, must-revalidate",accelBuffering:"no",body});
function runtimeEvidence(args) {
    const candidate=args.find((arg)=>arg.includes("dst=/verify/evidence")).match(/src=([^,]+)/)[1];
    const values=envValues(args); const json=(name,value)=>writeFileSync(join(candidate,name),JSON.stringify(value));
    writeFileSync(join(candidate,"steps.log"),"STEP manifest\n");
    const files={
        "initial.json":stream('event: reconcile.required\ndata: {"reason":"connected"}\n\nid: 0\n'),
        "event.json":{schemaVersion:1,eventId:"sse-runtime-open",producer:"monitor",topic:"monitor.incident.opened",subject:{kind:"monitor-incident",id:"subject",revision:1}},
        "ingest.json":{status:201,contentType:"application/json",body:{code:"stored"}},
        "live-change.json":stream("id: 0\nid: 1\nid: 2\n"),
        "inbox.json":envelope(200,0,"Success",{items:[{id:"message",topic:"monitor.incident.opened"}]}),
        "read.json":envelope(200,0,"Success",{id:"message",revision:2}),
        "retention.json":envelope(200,0,"Success",{count:0,revision:3}),
        "reconnect.json":stream('event: reconcile.required\ndata: {"reason":"connected"}\n\nid: 3\n'),
        "auth-401.json":envelope(401,401), "auth-403.json":envelope(403,403), "query-rejection.json":envelope(400,400),
        "redaction.json":{secretAbsent:true,queryAbsent:true,operationDescriptions:[]},
        "quota.json":{opened:4,fifth:{status:429,code:42901,retryAfter:60},released:true,reopened:200},
        "authority-change.json":{connected:true,closed:true}, "expiry.json":{connected:true,closed:true},
        "authority-outage.json":envelope(503,50302),
        "selected-state.json":{receipts:0,messages:0,recipients:0,revision:3,operationLogTablePresent:false},
    };
    const tamper=process.env.FAKE_TAMPER;
    if(tamper==="revision") files["retention.json"].body.data.revision=2;
    if(tamper==="replay") files["reconnect.json"].body+="id: 2\n";
    if(tamper==="authority") files["authority-outage.json"]=envelope(401,401);
    if(tamper==="redaction") files["redaction.json"].secretAbsent=false;
    if(tamper==="quota") files["quota.json"].fifth.status=200;
    if(tamper==="expiry") files["expiry.json"].closed=false;
    if(tamper==="stream-status") files["initial.json"].status=204;
    if(tamper==="stream-content-type") files["initial.json"].contentType="application/json";
    if(tamper==="stream-cache") files["initial.json"].cacheControl="no-store";
    if(tamper==="stream-buffering") files["initial.json"].accelBuffering="yes";
    for(const [name,value] of Object.entries(files)) json(name,value);
    const rows=inner.map((file)=>{const bytes=readFileSync(join(candidate,file));return{file,sha256:hash(bytes),bytes:bytes.byteLength};});
    const manifest={schemaVersion:1,status:"passed",gitHead:values.RUSTZEN_VERIFY_HEAD,sourceTreeState:values.RUSTZEN_VERIFY_SOURCE_TREE_STATE,sourceTreeSha256:values.RUSTZEN_VERIFY_SOURCE_TREE_SHA256,platform:{architecture:values.RUSTZEN_VERIFY_ARCHITECTURE,name:values.RUSTZEN_VERIFY_PLATFORM},binarySha256:values.RUSTZEN_VERIFY_BINARY_SHA256,buildProvenanceSha256:values.RUSTZEN_VERIFY_BUILD_PROVENANCE_SHA256,verifier:{imageId:values.RUSTZEN_VERIFY_VERIFIER_IMAGE_ID,key:values.RUSTZEN_VERIFY_VERIFIER_KEY,provenanceSha256:values.RUSTZEN_VERIFY_VERIFIER_PROVENANCE_SHA256},cases:{initialRevision:true,durableAdmission:true,durableRead:true,retention:true,lastEventIdNoReplay:true,unauthorized:true,forbidden:true,queryRejectedAndRedacted:true,perUserQuota:true,authorityChangeEof:true,expiryEof:true,authorityOutage:true},limitations:["fake"],receipts:rows};
    if(tamper==="duplicate") manifest.receipts.push(rows[0]); if(tamper==="path") manifest.receipts[0].file="../steps.log"; json("manifest.json",manifest);
}

export async function fakeDocker(args) {
    process.on("SIGINT",()=>process.exit(130)); process.on("SIGTERM",()=>process.exit(143)); mkdirSync(process.env.FAKE_STATE_ROOT,{recursive:true}); appendFileSync(process.env.FAKE_CALL_LOG,`${args.join(" ")}\n`);
    if(args[0]==="info") return console.log("aarch64");
    if(args[0]==="logs") return console.log("fake container log");
    if(args[0]==="rm") { if(process.env.FAKE_BLOCK==="rm") await Bun.sleep(30_000); if(process.env.FAKE_RM_FAIL==="1") process.exit(1); const state=join(process.env.FAKE_STATE_ROOT,args.at(-1)); if(existsSync(state)) unlinkSync(state); return; }
    if(args[0]==="container"&&args[1]==="inspect") process.exit(existsSync(join(process.env.FAKE_STATE_ROOT,args.at(-1)))?0:1);
    if(args[0]!=="run") process.exit(2); const name=args[args.indexOf("--name")+1]; writeFileSync(join(process.env.FAKE_STATE_ROOT,name),String(process.pid)); const build=args.includes("rust:1.95-bookworm");
    if(process.env.FAKE_BLOCK===(build?"build":"runtime")) await Bun.sleep(30_000);
    if(build){if(process.env.FAKE_BUILD_EXIT)process.exit(Number(process.env.FAKE_BUILD_EXIT));const bin=join(process.env.FAKE_EVIDENCE_ROOT,"build/aarch64/bin");mkdirSync(bin,{recursive:true});writeFileSync(join(bin,"rz-admin"),"fake-admin");chmodSync(join(bin,"rz-admin"),0o755);return;}
    if(process.env.FAKE_RUNTIME_EXIT)process.exit(Number(process.env.FAKE_RUNTIME_EXIT)); runtimeEvidence(args);
}

export function fixture(outer,options={}) {
    const root=mkdtempSync(join(tmpdir(),"rz-admin-sse-gate-")),evidence=join(root,"evidence"),states=join(root,"states"),calls=join(root,"calls");mkdirSync(join(evidence,"runs/previous"),{recursive:true});mkdirSync(states);writeFileSync(join(evidence,"runs/previous/manifest.json"),'{"status":"previous"}\n');symlinkSync("runs/previous",join(evidence,"current"));
    const source=join(root,"source.sh"),verifier=join(root,"verifier.sh"),file=join(root,"file.sh"),docker=join(root,"docker.sh");
    executable(source,"#!/bin/sh\nprintf 'fakehead\\tclean\\tfakesource\\n'\n"); executable(verifier,"#!/bin/sh\nset -eu\necho 'fake verifier helper' >&2\nprintf 'sha256:fake\\tfakekey\\tfakeprovenance\\n'\n"); executable(file,"#!/bin/sh\necho 'ELF 64-bit LSB executable, ARM aarch64'\n"); executable(docker,`#!/bin/sh\nexec '${process.execPath}' '${import.meta.filename}' --fake-docker "$@"\n`);
    const env={...process.env,RUSTZEN_ADMIN_SSE_DOCKER:docker,RUSTZEN_ADMIN_SSE_SOURCE_IDENTITY:source,RUSTZEN_ADMIN_SSE_VERIFIER_HELPER:verifier,RUSTZEN_ADMIN_SSE_FILE:file,RUSTZEN_ADMIN_SSE_EVIDENCE_ROOT:evidence,RUSTZEN_ADMIN_SSE_TIMEOUT:options.timeout??"5",RUSTZEN_ADMIN_SSE_CLEANUP_TIMEOUT:"1",RUSTZEN_ADMIN_SSE_KILL_GRACE:"1",RUSTZEN_ADMIN_SSE_HELPER_TIMEOUT:"5",RUSTZEN_ADMIN_SSE_HELPER_CLEANUP_GRACE:"2",FAKE_EVIDENCE_ROOT:evidence,FAKE_STATE_ROOT:states,FAKE_CALL_LOG:calls,FAKE_TAMPER:options.tamper??"",FAKE_BLOCK:options.block??"",FAKE_RUNTIME_EXIT:options.runtimeExit??"",FAKE_BUILD_EXIT:options.buildExit??"",FAKE_RM_FAIL:options.rmFail?"1":""};
    return{root,evidence,states,calls,env,outer,cleanup:()=>rmSync(root,{recursive:true,force:true}),current:()=>readlinkSync(join(evidence,"current")),active:()=>readdirSync(states),failed:()=>existsSync(join(evidence,"failed-runs"))?readdirSync(join(evidence,"failed-runs")):[],callsText:()=>existsSync(calls)?readFileSync(calls,"utf8"):""};
}
export const run=(item)=>Bun.spawnSync(["bash",item.outer],{env:item.env,stdout:"pipe",stderr:"pipe"});
export async function signal(item,name){const child=Bun.spawn(["bash",item.outer],{env:item.env,stdout:"pipe",stderr:"pipe"});for(let i=0;i<500&&item.active().length===0;i+=1)await Bun.sleep(10);if(item.active().length===0)throw new Error("fake Docker never started");child.kill(name);return await child.exited;}
if(process.argv[2]==="--fake-docker") await fakeDocker(process.argv.slice(3));
