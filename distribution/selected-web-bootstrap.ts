import { createHash } from "node:crypto";

const ENTRY_PATH = /^assets\/index-[A-Za-z0-9_-]+\.js$/;

export type SelectedWebBootstrap = {
    html: string;
    integrity: string;
};

export function createSelectedWebBootstrap(input: {
    entryPath: string;
    entryBytes: Uint8Array;
}): SelectedWebBootstrap {
    if (!ENTRY_PATH.test(input.entryPath))
        throw new Error("selected Web bootstrap entry path is invalid");
    if (!input.entryBytes.length) throw new Error("selected Web bootstrap entry is empty");
    const integrity = `sha256-${createHash("sha256").update(input.entryBytes).digest("base64")}`;
    const entryPath = JSON.stringify(`/${input.entryPath}`);
    const entryIntegrity = JSON.stringify(integrity);
    const source = `(()=>{
const endpoint="/__web-binding",markerKey="rustzen:web-binding-reload",reloadParam="__rz_web_reload";
const root=document.getElementById("root"),bindingName=["rustzen","web","binding"].join("-"),meta=document.querySelector('meta[name="'+bindingName+'"]');
const stamp=meta&&meta.getAttribute("content")||"";
const canonical=()=>{const value=new URL(location.href);value.searchParams.delete(reloadParam);return value;};
const attemptKey=()=>{const value=canonical();return stamp+"|"+value.pathname+value.search+value.hash;};
const showFailure=()=>{root.innerHTML='<main role="alert"><h1>Unable to start Rustzen Monitor</h1><p>The application files do not match this server.</p><button id="rustzen-web-retry" type="button">Retry</button></main>';document.getElementById("rustzen-web-retry").onclick=()=>{try{sessionStorage.removeItem(markerKey)}catch{}location.reload()};};
const fail=()=>{let attempted=true;try{attempted=sessionStorage.getItem(markerKey)===attemptKey();if(!attempted)sessionStorage.setItem(markerKey,attemptKey())}catch{}if(attempted){showFailure();return}const next=new URL(location.href);next.searchParams.set(reloadParam,Date.now().toString(36));location.replace(next.href);};
const validDigest=value=>typeof value==="string"&&/^[a-f0-9]{64}$/.test(value);
const start=async()=>{try{
if(!validDigest(stamp))throw new Error("invalid HTML binding");
const response=await fetch(endpoint,{cache:"no-store",credentials:"omit",redirect:"error",headers:{accept:"application/json"}});
const contentType=response.headers.get("content-type")||"",declared=Number(response.headers.get("content-length")||0);
if(!response.ok||!/^application\\/json(?:\\s*;|$)/i.test(contentType)||declared>256)throw new Error("invalid binding response");
const text=await response.text();if(new TextEncoder().encode(text).length>256)throw new Error("oversized binding response");
const binding=JSON.parse(text),keys=binding&&typeof binding==="object"&&!Array.isArray(binding)?Object.keys(binding).sort():[];
if(keys.join(",")!=="bindingVersion,webDigest"||binding.bindingVersion!==1||!validDigest(binding.webDigest)||binding.webDigest!==stamp)throw new Error("binding mismatch");
const entry=document.createElement("script");entry.type="module";entry.src=${entryPath};entry.integrity=${entryIntegrity};entry.crossOrigin="anonymous";
entry.onload=()=>{try{sessionStorage.removeItem(markerKey)}catch{}};entry.onerror=fail;document.head.append(entry);
}catch{fail()}};start();
})()`;
    if (source.toLowerCase().includes("</script"))
        throw new Error("selected Web bootstrap contains an unsafe script terminator");
    return { html: `<script>${source}</script>`, integrity };
}
