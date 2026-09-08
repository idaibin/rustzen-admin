#!/usr/bin/env python3
"""Run the selected-Web bootstrap closure in a disposable headless Chromium.

Python is used because the pinned verifier image already provides it; this avoids
adding a browser-driver dependency to the selected Monitor artifact.
"""
import argparse, base64, hashlib, importlib.util, json, os, shutil, socket, stat, struct, subprocess, time, urllib.parse, urllib.request
from pathlib import Path

fixture_spec = importlib.util.spec_from_file_location(
    "selected_web_bootstrap_browser_fixture",
    Path(__file__).with_name("selected-web-bootstrap-browser-fixture.py"),
)
fixture_module = importlib.util.module_from_spec(fixture_spec)
fixture_spec.loader.exec_module(fixture_module)
serve = fixture_module.serve

CASES = ("success", "bindingMismatch", "bindingNetworkFailure", "sriEntryFailure")
DEEP_LINK = "/monitoring/nodes?q=web#node-1"

def argument():
    parser = argparse.ArgumentParser()
    parser.add_argument("--admin-url", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--chromium", default="chromium")
    parser.add_argument("--username", default="owner")
    parser.add_argument("--password-file", required=True)
    parser.add_argument("--export-root", required=True)
    parser.add_argument("--release-result", required=True)
    parser.add_argument("--certificate", required=True)
    parser.add_argument("--public-key", required=True)
    parser.add_argument("--expected-source-identity", required=True)
    parser.add_argument("--admin-bin", required=True)
    parser.add_argument("--runtime-container", required=True)
    return parser.parse_args()

def sha256(path):
    with open(path, "rb") as handle: return hashlib.file_digest(handle, "sha256").hexdigest()

def admission(args):
    command = ["pnpm", "dlx", "bun@1.3.14", "scripts/verify-selected-web-browser-admission.ts", "--export-root", args.export_root, "--release-result", args.release_result, "--certificate", args.certificate, "--public-key", args.public_key, "--expected-source-identity", args.expected_source_identity, "--admin-bin", args.admin_bin]
    result = subprocess.run(command, cwd=Path(__file__).parent.parent, capture_output=True, text=True)
    if result.returncode: raise SystemExit(f"browser admission failed: {result.stderr.strip()}")
    value = json.loads(result.stdout)
    identity = value.get("admission", {})
    hashes = ("certificateSha256", "manifestSha256", "archiveSha256", "envelopeSha256", "buildId")
    certified_admin = next((row.get("sha256") for row in identity.get("binaryDigests", []) if row.get("path") == "bin/rz-admin"), "")
    if not all(len(identity.get(key, "")) == 64 for key in hashes) or len(identity.get("selection", {}).get("compositionId", "")) != 64 or len(value.get("webDigest", "")) != 64 or len(certified_admin) != 64 or value.get("expectedSourceIdentity") != args.expected_source_identity or value.get("currentSourceIdentity") != args.expected_source_identity: raise SystemExit("browser admission identity is incomplete")
    return value

def assert_admission_stable(before, after):
    if json.dumps(before, sort_keys=True, separators=(",",":")) != json.dumps(after, sort_keys=True, separators=(",",":")):
        raise RuntimeError("browser admission changed during browser gate")

def runtime_attestation(args, verified):
    command = ["pnpm", "dlx", "bun@1.3.14", "scripts/verify-selected-web-runtime-attestation.ts", "--runtime-container", args.runtime_container, "--admin-url", args.admin_url, "--expected-sha256", verified["adminSha256"]]
    result = subprocess.run(command, cwd=Path(__file__).parent.parent, capture_output=True, text=True)
    if result.returncode: raise RuntimeError(f"runtime attestation failed: {result.stderr.strip()}")
    return json.loads(result.stdout)

def entry_requests(receipt):
    return [row for row in receipt if row["path"].startswith("/assets/") and row["path"].endswith(".js")]

def assert_entry_boundary(case, receipt):
    if case in {"bindingMismatch", "bindingNetworkFailure"} and entry_requests(receipt):
        raise RuntimeError(f"{case}: binding failure requested business entry")

def port():
    sock = socket.socket(); sock.bind(("127.0.0.1", 0)); value = sock.getsockname()[1]; sock.close(); return value

def owner_password(path):
    flags = os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW
    try: descriptor = os.open(path, flags)
    except OSError as error: raise SystemExit(f"--password-file cannot be opened: {error.strerror}")
    try:
        state = os.fstat(descriptor)
        if not stat.S_ISREG(state.st_mode) or state.st_uid != os.getuid(): raise SystemExit("--password-file must be a current-user regular file")
        if stat.S_IMODE(state.st_mode) != 0o600: raise SystemExit("--password-file must have mode 0600")
        value = b""
        while chunk := os.read(descriptor, 4096): value += chunk
    finally: os.close(descriptor)
    value = value.decode().strip()
    if not value: raise SystemExit("--password-file is empty")
    return value

def admin_url(value):
    parsed = urllib.parse.urlsplit(value)
    try: port = parsed.port
    except ValueError: port = None
    if parsed.scheme != "http" or parsed.hostname != "127.0.0.1" or not port or parsed.username or parsed.password or parsed.path or parsed.query or parsed.fragment:
        raise SystemExit("--admin-url must be http://127.0.0.1:<port>")
    return value

def websocket(url):
    parsed = urllib.parse.urlsplit(url)
    sock = socket.create_connection((parsed.hostname, parsed.port), timeout=10)
    key = base64.b64encode(os.urandom(16)).decode()
    target = parsed.path + (f"?{parsed.query}" if parsed.query else "")
    sock.sendall((f"GET {target} HTTP/1.1\r\nHost: {parsed.netloc}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n").encode())
    response = b""
    while b"\r\n\r\n" not in response: response += recv_exact(sock, 1)
    if b" 101 " not in response.split(b"\r\n", 1)[0]: raise RuntimeError("Chromium CDP WebSocket upgrade failed")
    return sock

def recv_exact(sock, length):
    data = b""
    while len(data) < length:
        chunk = sock.recv(length - len(data))
        if not chunk: raise RuntimeError("Chromium CDP socket closed")
        data += chunk
    return data

def send(sock, value):
    payload = json.dumps(value, separators=(",", ":")).encode(); mask = os.urandom(4)
    header = bytes([129, 128 | len(payload)]) if len(payload) < 126 else bytes([129, 254]) + struct.pack("!H", len(payload))
    sock.sendall(header + mask + bytes(byte ^ mask[i % 4] for i, byte in enumerate(payload)))

def receive(sock):
    first, second = recv_exact(sock, 2); length = second & 127
    if length == 126: length = struct.unpack("!H", recv_exact(sock, 2))[0]
    if length == 127: length = struct.unpack("!Q", recv_exact(sock, 8))[0]
    data = recv_exact(sock, length)
    return json.loads(data.decode())

class CDP:
    def __init__(self, chromium):
        self.debug_port = port(); self.profile = Path("/tmp") / f"rz-selected-web-{os.getpid()}-{self.debug_port}"
        self.process = subprocess.Popen([chromium, "--headless=new", "--no-sandbox", "--remote-allow-origins=*", f"--remote-debugging-port={self.debug_port}", f"--user-data-dir={self.profile}", "about:blank"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        deadline = time.time() + 15
        while time.time() < deadline:
            try:
                info = json.load(urllib.request.urlopen(f"http://127.0.0.1:{self.debug_port}/json/version", timeout=1)); break
            except Exception: time.sleep(.1)
        else: raise RuntimeError("Chromium did not expose CDP")
        self.sock = websocket(info["webSocketDebuggerUrl"]); self.id = 0
        self.call("Target.createTarget", {"url":"about:blank"})
        tabs = json.load(urllib.request.urlopen(f"http://127.0.0.1:{self.debug_port}/json/list"))
        self.sock.close(); self.sock = websocket(next(tab["webSocketDebuggerUrl"] for tab in tabs if tab["type"] == "page")); self.id = 0
        self.call("Page.enable"); self.call("Runtime.enable"); self.call("Network.enable")
    def call(self, method, params=None):
        self.id += 1; call_id = self.id; send(self.sock, {"id":call_id, "method":method, "params":params or {}})
        while True:
            value = receive(self.sock)
            if value.get("id") == call_id:
                if "error" in value: raise RuntimeError(value["error"])
                return value.get("result", {})
    def evaluate(self, expression):
        result = self.call("Runtime.evaluate", {"expression":expression, "awaitPromise":True, "returnByValue":True})
        return result["result"].get("value")
    def close(self):
        self.sock.close(); self.process.terminate(); self.process.wait(timeout=10); shutil.rmtree(self.profile, ignore_errors=True)

def terminal_state(cdp, case):
    return cdp.evaluate("!!document.querySelector('#login_username') || !!document.querySelector('[role=alert]')") or (case == "sriIntegrityRemoved" and cdp.evaluate("!!globalThis.__rz_sri_tamper_executed"))

def browser_case(cdp, base, case, username, password):
    cdp.call("Page.navigate", {"url":base + DEEP_LINK})
    deadline = time.time() + 12
    while time.time() < deadline:
        if terminal_state(cdp, case): break
        time.sleep(.05)
    else: raise RuntimeError(f"{case}: bootstrap did not reach a terminal state")
    if case == "success":
        login = json.dumps({"username":username, "password":password})
        value = cdp.evaluate(f"fetch('/api/auth/login',{{method:'POST',headers:{{'content-type':'application/json'}},body:{login!r}}}).then(r=>r.json()).then(async r=>{{const i=await fetch('/api/installation',{{headers:{{Authorization:'Bearer '+r.data.token}}}}).then(v=>v.json());return {{login:!!r.data.token,buildId:i.data.buildId,compositionId:i.data.compositionId,webDigest:i.data.webDigest}}}})")
    else: value = None
    return {"value":value, "stamp":cdp.evaluate("document.querySelector('meta[name=rustzen-web-binding]').content"), "url":cdp.evaluate("location.href"), "alert":cdp.evaluate("!!document.querySelector('[role=alert]')"), "entryExecuted": cdp.evaluate("!!document.querySelector('#login_username')")}

def main():
    args = argument(); password = owner_password(args.password_file); url = admin_url(args.admin_url); output = Path(args.output); verified = admission(args); admin_before = sha256(args.admin_bin)
    if admin_before != verified["adminSha256"]: raise SystemExit("admin binary changed after admission")
    runtime_before = runtime_attestation(args, verified)
    if output.exists() or output.is_symlink(): raise SystemExit("--output must be a fresh non-symlink path")
    if not shutil.which(args.chromium) and not Path(args.chromium).is_file(): raise SystemExit("--chromium is unavailable")
    health = json.load(urllib.request.urlopen(url + "/health", timeout=10))
    binding = health.get("selectedBinding") or {}
    if binding.get("buildId") != verified["admission"]["buildId"] or binding.get("compositionId") != verified["admission"]["selection"]["compositionId"]:
        raise SystemExit("Admin health differs from verified release identity")
    if len(binding.get("buildId", "")) != 64 or len(binding.get("compositionId", "")) != 64:
        raise SystemExit("Admin health does not expose selected build/composition identity")
    results = []; cdp = CDP(args.chromium)
    try:
        for case in CASES:
            health_before = json.load(urllib.request.urlopen(url + "/health", timeout=10))
            if health_before.get("selectedBinding") != binding: raise RuntimeError(f"{case}: Admin health identity changed before run")
            server, fixture = serve(url, case)
            try:
                result = browser_case(cdp, f"http://127.0.0.1:{server.server_port}", case, args.username, password)
                receipt = fixture.requests; binding_request = next((row for row in receipt if row["path"].startswith("/__web-binding")), None)
                active = [row for row in receipt if row["path"].startswith("/__web-binding") or row["path"].startswith("/api/") or (row["path"].startswith("/assets/") and row["path"].endswith(".js"))]
                if not binding_request or not active or active[0] is not binding_request or binding_request["authorization"] or binding_request["cookie"]: raise RuntimeError(f"{case}: binding was not first anonymous API/JS request")
                failures = case != "success"; entry = entry_requests(receipt)
                reloadCount = max(0, len([row for row in receipt if row["path"].startswith("/monitoring/nodes")]) - 1)
                deep_link = urllib.parse.urlsplit(result["url"])
                api_requests = [row for row in receipt if row["path"].startswith("/api/")]
                if failures and (not result["alert"] or deep_link.path != "/monitoring/nodes" or urllib.parse.parse_qs(deep_link.query).get("q") != ["web"] or deep_link.fragment != "node-1" or result["entryExecuted"] or api_requests): raise RuntimeError(f"{case}: failure did not fail closed")
                assert_entry_boundary(case, receipt)
                if failures and reloadCount != 1: raise RuntimeError(f"{case}: expected exactly one recovery reload")
                if case == "sriEntryFailure" and (not entry or cdp.evaluate("!!globalThis.__rz_sri_tamper_executed")): raise RuntimeError("SRI entry executed despite integrity")
                if case == "success" and (not result["entryExecuted"] or not result["value"] or result["value"]["webDigest"] != result["stamp"] or result["value"]["webDigest"] != verified["webDigest"] or result["value"]["buildId"] != binding["buildId"] or result["value"]["compositionId"] != binding["compositionId"] or not any(row["authorization"] and row["cookie"] for row in api_requests)): raise RuntimeError("success installation identity or credential proof differs")
                manualRetry = None
                if failures:
                    documents_before = len([row for row in receipt if row["path"].startswith("/monitoring/nodes")])
                    cdp.evaluate("document.getElementById('rustzen-web-retry').click()")
                    deadline = time.time() + 10
                    while time.time() < deadline:
                        documents = [row for row in receipt if row["path"].startswith("/monitoring/nodes")]
                        if len(documents) >= documents_before + 2 and cdp.evaluate("!!document.querySelector('[role=alert]')"): break
                        time.sleep(.1)
                    documents = [row for row in receipt if row["path"].startswith("/monitoring/nodes")]
                    next_document = documents[documents_before] if len(documents) > documents_before else None
                    if not next_document or "__rz_web_reload" in next_document["path"] or "q=web" not in next_document["path"]:
                        raise RuntimeError(f"{case}: Retry did not first navigate to canonical deep link")
                    postRetryAutomaticReloadCount = len(documents) - documents_before - 1
                    if postRetryAutomaticReloadCount != 1 or not cdp.evaluate("!!document.querySelector('[role=alert]')") or cdp.evaluate("!!document.querySelector('#login_username')") or cdp.evaluate("!!globalThis.__rz_sri_tamper_executed") or [row for row in receipt if row["path"].startswith("/api/")]: raise RuntimeError(f"{case}: Retry budget or failure boundary changed")
                    assert_entry_boundary(case, receipt)
                    manualRetry = {"canonicalDocument":next_document["path"], "postRetryAutomaticReloadCount":postRetryAutomaticReloadCount}
                health_after = json.load(urllib.request.urlopen(url + "/health", timeout=10))
                if health_after.get("selectedBinding") != binding: raise RuntimeError(f"{case}: Admin health identity changed after run")
                results.append({"case":case, "health":{"before":health_before,"after":health_after}, "browser":result | {"reloadCount":reloadCount, "manualRetry":manualRetry}, "requests":receipt, "assertions":{"bindingFirstActive":True,"bindingCredentialOmitted":True,"failureApiAbsent":not failures or not api_requests,"entryExecuted":result["entryExecuted"]}})
            finally: server.shutdown(); server.server_close()
        health_before = json.load(urllib.request.urlopen(url + "/health", timeout=10))
        server, fixture = serve(url, "sriIntegrityRemoved")
        try:
            browser_case(cdp, f"http://127.0.0.1:{server.server_port}", "sriIntegrityRemoved", args.username, password)
            if not cdp.evaluate("!!globalThis.__rz_sri_tamper_executed"):
                raise RuntimeError("integrity-removal negative fixture did not execute its entry")
            integritySensitivityPassed = health_before.get("selectedBinding") == binding
        finally: server.shutdown(); server.server_close()
        health_after = json.load(urllib.request.urlopen(url + "/health", timeout=10))
        sensitivity = {"requests":fixture.requests, "marker":True, "health":{"before":health_before,"after":health_after}}
        if not integritySensitivityPassed or health_after.get("selectedBinding") != binding: raise RuntimeError("SRI sensitivity health identity changed")
        chromiumVersion = cdp.evaluate("navigator.userAgent")
        final_health = json.load(urllib.request.urlopen(url + "/health", timeout=10))
        if final_health != health: raise RuntimeError("Admin health identity changed during browser gate")
        admin_after = sha256(args.admin_bin)
        if admin_after != admin_before or admin_after != verified["adminSha256"]: raise RuntimeError("admin binary changed during browser gate")
        assert_admission_stable(verified, admission(args))
        runtime_after = runtime_attestation(args, verified)
        if runtime_after != runtime_before: raise RuntimeError("runtime attestation changed during browser gate")
    finally: cdp.close()
    success = next(item for item in results if item["case"] == "success")["browser"]
    manifest = {"schemaVersion":1,"status":"passed","chromiumVersion":chromiumVersion,"adminHealth":{"initial":health,"final":final_health},"digests":{"buildId":binding["buildId"],"compositionId":binding["compositionId"],"html":success["stamp"],"binding":success["stamp"],"installation":success["value"]["webDigest"],"verified":verified["webDigest"],"adminBinary":{"before":admin_before,"after":admin_after}},"release":verified["admission"],"sourceIdentity":{"expected":verified["expectedSourceIdentity"],"current":verified["currentSourceIdentity"]},"runtime":{"before":runtime_before,"after":runtime_after},"verifier":{"sources":verified["provenance"]},"integritySensitivityPassed":integritySensitivityPassed,"sensitivity":sensitivity,"cases":results}
    receipt = json.dumps(manifest, sort_keys=True, separators=(",", ":")).encode()
    if json.dumps(json.loads(receipt), sort_keys=True, separators=(",", ":")).encode() != receipt: raise RuntimeError("browser receipt is not canonical")
    output.mkdir(parents=False)
    path = output / "manifest.json"; path.write_bytes(receipt)

if __name__ == "__main__": main()
