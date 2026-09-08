import { appendFileSync, chmodSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, readlinkSync, rmSync, symlinkSync, unlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

export const innerReceipts=["steps.log","browser-results.json","proxy-receipt.json","sse-preflight.json","realtime-timing.json","forbidden-timing.json","selected-state.json","pure-absence.json","message-center-desktop-en.png","message-center-mobile-en.png"];
export const receipts=[...innerReceipts,"phase-status.tsv","source-tests.log","web-build.log","build.log","verifier-helper.log","runtime.log","browser-steps.json","build-provenance.tsv"];
const cases=["emptyDesktop","realtimeInvalidation","loading","populatedDetail","unreadFilterPaging","singleRead","readAll","forbiddenClears","unauthorized","incidentDeepLink","mobile"];
const hash=(value)=>new Bun.CryptoHasher("sha256").update(value).digest("hex");
const executable=(path,body)=>{writeFileSync(path,body);chmodSync(path,0o755);};
const envs=(args)=>{const result={};for(let index=0;index<args.length;index+=1)if(args[index]==="--env"){const [key,...value]=args[++index].split("=");result[key]=value.join("=");}return result;};
function runtime(args){
    const evidence=args.find((arg)=>arg.includes("dst=/verify/evidence")).match(/src=([^,]+)/)[1],values=envs(args);
    const json=(name,value)=>writeFileSync(join(evidence,name),JSON.stringify(value));
    writeFileSync(join(evidence,"steps.log"),"STEP manifest\n");
    const steps=JSON.parse(readFileSync(join(evidence,"browser-steps.json"),"utf8"));
    const browser=cases.map((name)=>({case:name,steps:steps[name].map(({action})=>({action,status:"succeeded",message:null}))}));
    const proxy=[{case:"preflight",method:"GET",path:"/api/notifications/stream",query:false,tokenInUrl:false,bearer:true,accept:"text/event-stream",status:200,atNs:1}];
    for(const name of cases)proxy.push({case:name,method:"GET",path:"/api/notifications/stream",query:false,tokenInUrl:false,bearer:true,accept:"text/event-stream",status:name==="realtimeInvalidation"?200:name==="unauthorized"?401:204,atNs:10});
    proxy.push({case:"realtimeInvalidation",method:"GET",path:"/api/notifications/unread-count",query:false,tokenInUrl:false,bearer:true,accept:"*/*",status:200,atNs:20});
    proxy.push({case:"realtimeInvalidation",method:"GET",path:"/api/notifications",query:true,tokenInUrl:false,bearer:true,accept:"*/*",status:200,atNs:21});
    proxy.push({case:"realtimeInvalidation",method:"GET",path:"/api/notifications/unread-count",query:false,tokenInUrl:false,bearer:true,accept:"*/*",status:200,atNs:40});
    proxy.push({case:"realtimeInvalidation",method:"GET",path:"/api/notifications",query:true,tokenInUrl:false,bearer:true,accept:"*/*",status:200,atNs:41});
    proxy.push({case:"forbiddenClears",method:"GET",path:"/api/notifications",query:false,tokenInUrl:false,bearer:true,accept:"application/json",status:200,atNs:20});
    proxy.push({case:"forbiddenClears",method:"GET",path:"/api/notifications",query:false,tokenInUrl:false,bearer:true,accept:"application/json",status:403,atNs:40});
    const tamper=process.env.FAKE_TAMPER;
    if(tamper==="browser")browser.pop(); if(tamper==="token")proxy[0].tokenInUrl=true;
    if(tamper==="actions")browser[0].steps[0].action="click";
    if(tamper==="forbidden")proxy.pop();
    if(tamper==="sse-status")proxy[0].status=204;
    if(tamper==="unauthorized")proxy.find((row)=>row.case==="unauthorized"&&row.path.endsWith("stream")).status=200;
    if(tamper==="browser-sse-missing")proxy.splice(proxy.findIndex((row)=>row.case==="realtimeInvalidation"&&row.path.endsWith("stream")),1);
    if(tamper==="browser-sse-duplicate")proxy.push({...proxy.find((row)=>row.case==="realtimeInvalidation"&&row.path.endsWith("stream"))});
    if(tamper==="browser-sse-case")proxy.find((row)=>row.case==="realtimeInvalidation"&&row.path.endsWith("stream")).case="loading";
    if(tamper==="realtime-refetch")proxy.splice(proxy.findIndex((row)=>row.case==="realtimeInvalidation"&&row.path==="/api/notifications"&&row.atNs>30),1);
    json("browser-results.json",browser);json("proxy-receipt.json",proxy);
    const stream={status:200,contentType:"text/event-stream",path:"/api/notifications/stream",query:false,bearer:true,frames:["reconcile.required","inbox.changed"]};
    json("sse-preflight.json",{direct:{label:"direct",...stream},proxy:{label:"proxy",...stream}});
    json("realtime-timing.json",{case:"realtimeInvalidation",streamReadyAtNs:10,listReadyAtNs:21,admissionStartedAtNs:30,notificationCount:1});
    json("forbidden-timing.json",{case:"forbiddenClears",listStatus:200,listReadyAtNs:tamper==="pre-ready"?3:1,admissionStartedAtNs:2,eventId:"browser-page-22",admissionStatus:201,admissionCode:"stored"});
    json("selected-state.json",{messages:22,recipients:22,reads:22,incidents:1});
    json("pure-absence.json",tamper==="pure"?{schemaObjects:["notifications"],notificationModules:[],preset:"monitor",routeStatus:404,ingressListener:false,notificationOwners:false}:{schemaObjects:[],notificationModules:[],preset:"monitor",routeStatus:404,ingressListener:false,notificationOwners:false});
    writeFileSync(join(evidence,"message-center-desktop-en.png"),"desktop");writeFileSync(join(evidence,"message-center-mobile-en.png"),"mobile");
    const rows=innerReceipts.map((file)=>{const bytes=readFileSync(join(evidence,file));return{file,sha256:hash(bytes),bytes:bytes.length};});
    const manifest={schemaVersion:1,status:"passed",gitHead:values.RUSTZEN_VERIFY_HEAD,sourceTreeState:values.RUSTZEN_VERIFY_SOURCE_TREE_STATE,sourceTreeSha256:values.RUSTZEN_VERIFY_SOURCE_TREE_SHA256,platform:{architecture:values.RUSTZEN_VERIFY_ARCHITECTURE,name:values.RUSTZEN_VERIFY_PLATFORM},binaryHashes:JSON.parse(values.RUSTZEN_VERIFY_BINARY_HASHES),web:{selectedInventorySha256:values.RUSTZEN_VERIFY_WEB_INVENTORY_SHA256,pureInventorySha256:values.RUSTZEN_VERIFY_PURE_INVENTORY_SHA256},buildProvenanceSha256:values.RUSTZEN_VERIFY_BUILD_PROVENANCE_SHA256,verifier:{imageId:values.RUSTZEN_VERIFY_VERIFIER_IMAGE_ID,key:values.RUSTZEN_VERIFY_VERIFIER_KEY,provenanceSha256:values.RUSTZEN_VERIFY_VERIFIER_PROVENANCE_SHA256},cases:{browser:11,sseBearer:true,sseUrlSecret:false,browserSseLifecycle:true,singleLifecycle:true,durableReconcile:true,forbiddenClears:true,unauthorizedLogin:true,incidentDeepLink:true,pureAbsent:true},screenshots:[{file:"message-center-desktop-en.png",dimensions:"1440 x 900"},{file:"message-center-mobile-en.png",dimensions:"390 x 844"}],receipts:rows,limitations:["fake"]};
    if(tamper==="duplicate")manifest.receipts.push(rows[0]);if(tamper==="path")manifest.receipts[0].file="../steps.log";if(tamper==="badge")manifest.cases.browser=9;
    json("manifest.json",manifest);
}
export async function fakeDocker(args){
    process.on("SIGINT",()=>process.exit(130));process.on("SIGTERM",()=>process.exit(143));mkdirSync(process.env.FAKE_STATE_ROOT,{recursive:true});appendFileSync(process.env.FAKE_CALL_LOG,`${args.join(" ")}\n`);
    if(args[0]==="info")return console.log("aarch64");if(args[0]==="logs")return console.log("fake container log");
    if(args[0]==="rm"){if(process.env.FAKE_RM_FAIL==="1")process.exit(1);const state=join(process.env.FAKE_STATE_ROOT,args.at(-1));if(existsSync(state))unlinkSync(state);return;}
    if(args[0]==="container"&&args[1]==="inspect")process.exit(existsSync(join(process.env.FAKE_STATE_ROOT,args.at(-1)))?0:1);if(args[0]!=="run")process.exit(2);
    const name=args[args.indexOf("--name")+1],build=args.includes("rust:1.95-bookworm");writeFileSync(join(process.env.FAKE_STATE_ROOT,name),String(process.pid));if(process.env.FAKE_BLOCK===(build?"build":"runtime"))await Bun.sleep(30_000);
    if(build){if(process.env.FAKE_BUILD_EXIT)process.exit(Number(process.env.FAKE_BUILD_EXIT));const bin=join(process.env.FAKE_EVIDENCE_ROOT,"build/aarch64/bin");mkdirSync(bin,{recursive:true});for(const name of ["rz-admin-notify","rz-monitor-notify","rz-admin-pure","rz-monitor-pure","rz-admin-verifier","rz-reports-verifier"]){writeFileSync(join(bin,name),name);chmodSync(join(bin,name),0o755);}return;}
    if(process.env.FAKE_RUNTIME_EXIT){const evidence=args.find((arg)=>arg.includes("dst=/verify/evidence")).match(/src=([^,]+)/)[1];writeFileSync(join(evidence,"failed-case-emptyDesktop-steps.json"),JSON.stringify({case:"emptyDesktop",runId:"fake-run",steps:[{stepIndex:6,action:"waitFor",status:"failed",message:"fake selector failure"}]}));writeFileSync(join(evidence,"failed-case-emptyDesktop-artifacts.json"),'{"data":[]}');process.exit(Number(process.env.FAKE_RUNTIME_EXIT));}runtime(args);
}
export function fixture(outer,options={}){
    const root=mkdtempSync(join(tmpdir(),"rz-message-center-gate-")),evidence=join(root,"evidence"),states=join(root,"states"),calls=join(root,"calls");mkdirSync(join(evidence,"runs/previous"),{recursive:true});mkdirSync(states);writeFileSync(join(evidence,"runs/previous/manifest.json"),'{"status":"previous"}\n');symlinkSync("runs/previous",join(evidence,"current"));
    const source=join(root,"source.sh"),sourceTest=join(root,"source-test.sh"),verifier=join(root,"verifier.sh"),file=join(root,"file.sh"),docker=join(root,"docker.sh"),web=join(root,"web.sh");
    executable(source,"#!/bin/sh\nprintf 'fakehead\\tclean\\tfakesource\\n'\n");executable(sourceTest,"#!/bin/sh\necho 'source tests passed'\n");executable(verifier,"#!/bin/sh\necho helper >&2\nprintf 'sha256:fake\\tfakekey\\tfakeprovenance\\n'\n");executable(file,"#!/bin/sh\ncase \"$1\" in *desktop*) echo 'PNG image data, 1440 x 900';; *mobile*) echo 'PNG image data, 390 x 844';; *) echo 'ELF 64-bit LSB executable, ARM aarch64';; esac\n");
    executable(web,"#!/bin/sh\nset -eu\nmkdir -p \"$1/notify/dist\" \"$1/pure/dist\"\nprintf notify >\"$1/notify/dist/index.html\"; printf pure >\"$1/pure/dist/index.html\"\nprintf '{\"preset\":\"monitor-notify\"}' >\"$1/notify/inventory.json\"; printf '{\"preset\":\"monitor\"}' >\"$1/pure/inventory.json\"\necho staged\n");executable(docker,`#!/bin/sh\nexec '${process.execPath}' '${import.meta.filename}' --fake-docker "$@"\n`);
    const env={...process.env,RUSTZEN_MESSAGE_CENTER_DOCKER:docker,RUSTZEN_MESSAGE_CENTER_SOURCE_IDENTITY:source,RUSTZEN_MESSAGE_CENTER_SOURCE_TESTER:sourceTest,RUSTZEN_MESSAGE_CENTER_VERIFIER_HELPER:verifier,RUSTZEN_MESSAGE_CENTER_WEB_BUILDER:web,RUSTZEN_MESSAGE_CENTER_FILE:file,RUSTZEN_MESSAGE_CENTER_EVIDENCE_ROOT:evidence,RUSTZEN_MESSAGE_CENTER_TIMEOUT:options.timeout??"5",RUSTZEN_MESSAGE_CENTER_CLEANUP_TIMEOUT:"1",RUSTZEN_MESSAGE_CENTER_KILL_GRACE:"1",RUSTZEN_MESSAGE_CENTER_HELPER_TIMEOUT:"5",RUSTZEN_MESSAGE_CENTER_HELPER_CLEANUP_GRACE:"2",FAKE_EVIDENCE_ROOT:evidence,FAKE_STATE_ROOT:states,FAKE_CALL_LOG:calls,FAKE_TAMPER:options.tamper??"",FAKE_BLOCK:options.block??"",FAKE_RUNTIME_EXIT:options.runtimeExit??"",FAKE_BUILD_EXIT:options.buildExit??"",FAKE_RM_FAIL:options.rmFail?"1":""};
    return{root,evidence,env,outer,cleanup:()=>rmSync(root,{recursive:true,force:true}),current:()=>readlinkSync(join(evidence,"current")),active:()=>readdirSync(states),failed:()=>existsSync(join(evidence,"failed-runs"))?readdirSync(join(evidence,"failed-runs")):[],calls:()=>existsSync(calls)?readFileSync(calls,"utf8"):""};
}
export const run=(item)=>Bun.spawnSync(["bash",item.outer],{env:item.env,stdout:"pipe",stderr:"pipe"});
export async function signal(item,name){const child=Bun.spawn(["bash",item.outer],{env:item.env,stdout:"pipe",stderr:"pipe"});for(let i=0;i<500&&item.active().length===0;i++)await Bun.sleep(10);child.kill(name);return child.exited;}
if(process.argv[2]==="--fake-docker")await fakeDocker(process.argv.slice(3));
