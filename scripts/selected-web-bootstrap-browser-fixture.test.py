#!/usr/bin/env python3
import base64
import hashlib
import http.server
import importlib.util
import threading
import urllib.request
import socket
import tempfile
import os
import sys
import json
from pathlib import Path

spec = importlib.util.spec_from_file_location("fixture", Path(__file__).with_name("selected-web-bootstrap-browser-fixture.py"))
module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
gate_spec = importlib.util.spec_from_file_location("gate", Path(__file__).with_name("verify-selected-web-bootstrap-browser.py"))
gate = importlib.util.module_from_spec(gate_spec); gate_spec.loader.exec_module(gate)
import selected_web_bootstrap_cdp as cdp

class Admin(http.server.BaseHTTPRequestHandler):
    def log_message(self, *_): pass
    def do_GET(self):
        body = b'<script>entry.integrity="sha256-good";</script>' if self.path.startswith("/monitoring/nodes") else b"original entry"
        self.send_response(200); self.send_header("content-type", "text/html"); self.send_header("content-length", str(len(body))); self.end_headers(); self.wfile.write(body)

admin = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Admin)
thread = threading.Thread(target=admin.serve_forever, daemon=True); thread.start()
fixture, receipt = module.serve(f"http://127.0.0.1:{admin.server_port}", "sriIntegrityRemoved")
try:
    index = urllib.request.urlopen(f"http://127.0.0.1:{fixture.server_port}/monitoring/nodes").read()
    entry = urllib.request.urlopen(f"http://127.0.0.1:{fixture.server_port}/assets/index-test.js").read()
    assert b"entry.integrity=undefined" in index and b"sha256-good" not in index
    assert entry == b"globalThis.__rz_sri_tamper_executed = true;"
    assert [item["path"] for item in receipt.requests] == ["/monitoring/nodes", "/assets/index-test.js"]
finally:
    fixture.shutdown(); fixture.server_close(); admin.shutdown(); admin.server_close()

left, right = socket.socketpair()
try:
    payload = b'{"method":"fragmented"}'
    frame = bytes([129, len(payload)]) + payload
    for byte in frame: left.send(bytes([byte]))
    assert cdp.receive(right) == {"method": "fragmented"}
finally: left.close(); right.close()

def frame(opcode, payload, final=True):
    return bytes([(128 if final else 0) | opcode, len(payload)]) + payload

left, right = socket.socketpair()
try:
    left.sendall(frame(1, b'{"method":', final=False) + frame(0, b'"fragmented"}'))
    assert cdp.receive(right) == {"method": "fragmented"}
finally: left.close(); right.close()

left, right = socket.socketpair()
try:
    left.sendall(frame(9, b"ping") + frame(1, b'{"method":"control"}'))
    assert cdp.receive(right) == {"method": "control"}
    pong = left.recv(10); assert pong[:2] == bytes([138, 132])
    assert bytes(value ^ pong[2 + index % 4] for index, value in enumerate(pong[6:])) == b"ping"
finally: left.close(); right.close()

left, right = socket.socketpair()
try:
    left.sendall(bytes([129, 127]) + (cdp.MAX_CDP_FRAME_BYTES + 1).to_bytes(8, "big"))
    try: cdp.receive(right); raise AssertionError("oversized frame accepted")
    except RuntimeError as error: assert "exceeds limit" in str(error)
finally: left.close(); right.close()

def handshake(mode):
    listener = socket.socket(); listener.bind(("127.0.0.1", 0)); listener.listen(1); request_line = []
    def accept_handshake():
        connection, _ = listener.accept(); data = b""
        while b"\r\n\r\n" not in data: data += connection.recv(1024)
        request_line.append(data.split(b"\r\n", 1)[0])
        key = next(line[19:] for line in data.decode().split("\r\n") if line.startswith("Sec-WebSocket-Key: "))
        accept = base64.b64encode(hashlib.sha1((key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").encode()).digest()).decode()
        headers = "Upgrade: websocket\r\nConnection: keep-alive, Upgrade\r\n"
        if mode in ("valid", "alternate"): headers += f"Sec-WebSocket-Accept: {accept}\r\n"
        if mode == "wrong": headers += "Sec-WebSocket-Accept: wrong\r\n"
        reason = "WebSocket Protocol Handshake" if mode == "alternate" else "Switching Protocols"
        connection.sendall(f"HTTP/1.1 101 {reason}\r\n{headers}\r\n".encode()); connection.close()
    thread = threading.Thread(target=accept_handshake, daemon=True); thread.start()
    return listener, request_line

for mode in ("valid", "alternate"):
    listener, request_line = handshake(mode)
    client = cdp.websocket(f"ws://127.0.0.1:{listener.getsockname()[1]}/devtools/page/target", listener.getsockname()[1])
    client.close(); listener.close()
    assert request_line == [b"GET /devtools/page/target HTTP/1.1"]
for mode in ("missing", "wrong"):
    listener, _ = handshake(mode)
    try: cdp.websocket(f"ws://127.0.0.1:{listener.getsockname()[1]}/devtools/page/target", listener.getsockname()[1]); raise AssertionError(f"{mode} accept accepted")
    except RuntimeError as error: assert "upgrade differs" in str(error)
    finally: listener.close()
for url, port in (("ws://localhost:1/devtools/page/target", 1), ("ws://127.0.0.1:1/devtools/page/target", 2)):
    try: cdp.websocket(url, port); raise AssertionError("unpinned WebSocket accepted")
    except RuntimeError as error: assert "pinned loopback" in str(error)
with tempfile.TemporaryDirectory() as directory:
    profile = Path(directory)
    for value in ("not-a-port\n/devtools/browser/id\n", "1234\n/devtools/browser/id?query\n", "1234\n/devtools/browser/id\nextra\n"):
        (profile / "DevToolsActivePort").write_text(value)
        try: cdp.devtools_active_port(profile); raise AssertionError("invalid DevToolsActivePort accepted")
        except RuntimeError as error: assert "invalid" in str(error)
    class Alive: poll = lambda self: None
    (profile / "DevToolsActivePort").write_text("1234\n")
    def complete(_): (profile / "DevToolsActivePort").write_text("1234\n/devtools/browser/id\n")
    assert cdp.wait_devtools_active_port(profile, Alive(), timeout=1, sleep=complete) == (1234, "/devtools/browser/id")
    (profile / "DevToolsActivePort").write_text("bad\n")
    try: cdp.wait_devtools_active_port(profile, Alive(), timeout=0, sleep=lambda _: None); raise AssertionError("persistent invalid port accepted")
    except RuntimeError as error: assert "remained invalid" in str(error)
    class Exited: poll = lambda self: 1
    try: cdp.wait_devtools_active_port(profile, Exited(), timeout=0); raise AssertionError("early Chromium exit accepted")
    except RuntimeError as error: assert "exited" in str(error)

class FakeSocket:
    def __init__(self, calls, broken=False): self.calls, self.broken = calls, broken
    def close(self):
        self.calls.append("socket.close")
        if self.broken: raise OSError("closed")
class FakeProcess:
    def __init__(self, calls, hung=False): self.calls, self.hung = calls, hung; self.waits = 0
    def poll(self): return None
    def terminate(self): self.calls.append("terminate")
    def kill(self): self.calls.append("kill")
    def wait(self, timeout):
        self.calls.append(f"wait:{timeout}"); self.waits += 1
        if self.hung and self.waits == 1: raise cdp.subprocess.TimeoutExpired("chrome", timeout)
with tempfile.TemporaryDirectory() as directory:
    calls = []; client = object.__new__(cdp.CDP); client.profile = Path(directory) / "profile"; client.profile.mkdir(); client.sock = FakeSocket(calls, broken=True); client.process = FakeProcess(calls, hung=True)
    client.close()
    assert calls == ["socket.close", "terminate", "wait:10", "kill", "wait:10"] and not client.profile.exists()
with tempfile.TemporaryDirectory() as directory:
    calls = []; client = object.__new__(cdp.CDP); client.profile = Path(directory) / "profile"; client.profile.mkdir(); client.sock = FakeSocket(calls); client.process = FakeProcess(calls)
    client.close(); assert calls == ["socket.close", "terminate", "wait:10"] and not client.profile.exists()
with tempfile.TemporaryDirectory() as directory:
    profile = Path(directory) / "profile"; profile.mkdir(); previous_mkdtemp, previous_popen = cdp.tempfile.mkdtemp, cdp.subprocess.Popen
    try:
        cdp.tempfile.mkdtemp = lambda **_: str(profile)
        cdp.subprocess.Popen = lambda *_args, **_kwargs: (_ for _ in ()).throw(ValueError("launch failed"))
        try: cdp.CDP("chromium"); raise AssertionError("constructor failure accepted")
        except ValueError as error: assert "launch failed" in str(error)
        assert not profile.exists()
    finally: cdp.tempfile.mkdtemp, cdp.subprocess.Popen = previous_mkdtemp, previous_popen

class SensitivityTerminal:
    def evaluate(self, expression):
        return expression == "!!globalThis.__rz_sri_tamper_executed"
assert gate.terminal_state(SensitivityTerminal(), "sriIntegrityRemoved")
try: gate.admin_url("http://localhost:8080"); raise AssertionError("localhost accepted")
except SystemExit: pass

# An eager or speculative business entry is forbidden for both binding failures,
# across every navigation because the helper receives the cumulative receipt.
for case in ("bindingMismatch", "bindingNetworkFailure"):
    try:
        gate.assert_entry_boundary(case, [{"path":"/__web-binding"}, {"path":"/assets/eager.js"}])
        raise AssertionError(f"{case} eager entry accepted")
    except RuntimeError as error:
        assert "business entry" in str(error)
gate.assert_entry_boundary("sriEntryFailure", [{"path":"/__web-binding"}, {"path":"/assets/entry.js"}])

with tempfile.TemporaryDirectory() as directory:
    secret = Path(directory) / "secret"; secret.write_text("owner-secret\n"); secret.chmod(0o600)
    assert gate.owner_password(secret) == "owner-secret"
    secret.chmod(0o644)
    try: gate.owner_password(secret); raise AssertionError("mode accepted")
    except SystemExit: pass
    secret.chmod(0o600); link = Path(directory) / "link"; link.symlink_to(secret)
    try: gate.owner_password(link); raise AssertionError("symlink accepted")
    except SystemExit: pass
    # The reader validates and consumes one descriptor, so path replacement cannot
    # cause it to validate one object then reopen another.
    assert "os.open(path, flags)" in Path(gate.__file__).read_text()

with tempfile.TemporaryDirectory() as directory:
    secret = Path(directory) / "secret"; secret.write_text("owner-secret\n"); secret.chmod(0o600)
    output = Path(directory) / "must-not-exist"
    previous = sys.argv
    try:
        sys.argv = ["gate", "--admin-url", "http://example.test:80", "--password-file", str(secret), "--output", str(output), "--selection", "x", "--export-root", "x", "--release-result", "x", "--certificate", "x", "--public-key", "x", "--expected-source-identity", "x", "--admin-bin", "x", "--runtime-container", "x"]
        try: gate.main(); raise AssertionError("invalid preflight accepted")
        except SystemExit: pass
        assert not output.exists()
    finally: sys.argv = previous

# Admission failures are terminal before Chromium or evidence publication. The
# certificate verifier remains the authority; these doubles exercise its boundary.
class AdmissionArgs:
    selection = export_root = release_result = certificate = public_key = expected_source_identity = admin_bin = "ignored"
def complete_admission(verifier="verifier"):
    return {"admission":{"certificateSha256":"4" * 64, "manifestSha256":"5" * 64, "archiveSha256":"6" * 64, "envelopeSha256":"7" * 64, "buildId":"2" * 64, "selection":{"preset":"monitor-notify", "target":"x86_64-unknown-linux-musl", "artifactClass":"server", "compositionId":"3" * 64}, "binaryDigests":[{"path":"bin/rz-admin", "sha256":"8" * 64}]}, "webDigest":"1" * 64, "productSourceIdentity":"source", "verifierSourceIdentity":verifier}

original_run = gate.subprocess.run
try:
    for reason in ("release identity differs", "source identity differs"):
        gate.subprocess.run = lambda *_, **__: gate.subprocess.CompletedProcess([], 1, "", reason)
        try: gate.admission(AdmissionArgs()); raise AssertionError("admission failure accepted")
        except SystemExit as error: assert reason in str(error)
    # Matching build/composition values alone are not evidence: the full P8e
    # certificate tuple and certified rz-admin member are mandatory.
    pseudo = {"admission":{"buildId":"2" * 64, "selection":{"compositionId":"3" * 64}}, "webDigest":"1" * 64}
    gate.subprocess.run = lambda *_, **__: gate.subprocess.CompletedProcess([], 0, json.dumps(pseudo), "")
    try: gate.admission(AdmissionArgs()); raise AssertionError("pseudo self-consistent admission accepted")
    except SystemExit as error: assert "incomplete" in str(error)
    mismatch = complete_admission("")
    gate.subprocess.run = lambda *_, **__: gate.subprocess.CompletedProcess([], 0, json.dumps(mismatch), "")
    try: gate.admission(AdmissionArgs()); raise AssertionError("missing verifier identity accepted")
    except SystemExit as error: assert "incomplete" in str(error)
finally: gate.subprocess.run = original_run

with tempfile.TemporaryDirectory() as directory:
    secret = Path(directory) / "secret"; secret.write_text("owner-secret\n"); secret.chmod(0o600)
    admin_bin = Path(directory) / "rz-admin"; admin_bin.write_bytes(b"different-admin")
    output = Path(directory) / "must-not-exist"
    original_admission, previous = gate.admission, sys.argv
    try:
        gate.admission = lambda _: {"adminSha256":"0" * 64, "webDigest":"1" * 64, "admission":{"buildId":"2" * 64, "selection":{"compositionId":"3" * 64}}}
        sys.argv = ["gate", "--admin-url", "http://127.0.0.1:9", "--password-file", str(secret), "--output", str(output), "--selection", "x", "--export-root", "x", "--release-result", "x", "--certificate", "x", "--public-key", "x", "--expected-source-identity", "x", "--admin-bin", str(admin_bin), "--runtime-container", "x"]
        try: gate.main(); raise AssertionError("admin digest mismatch accepted")
        except SystemExit as error: assert "admin binary changed" in str(error)
        assert not output.exists()
    finally: gate.admission, sys.argv = original_admission, previous

with tempfile.TemporaryDirectory() as directory:
    secret = Path(directory) / "secret"; secret.write_text("owner-secret\n"); secret.chmod(0o600)
    output = Path(directory) / "must-not-exist"
    previous, original_run = sys.argv, gate.subprocess.run
    try:
        gate.subprocess.run = lambda *_, **__: gate.subprocess.CompletedProcess([], 1, "", "source identity differs")
        sys.argv = ["gate", "--admin-url", "http://127.0.0.1:9", "--password-file", str(secret), "--output", str(output), "--selection", "x", "--export-root", "x", "--release-result", "x", "--certificate", "x", "--public-key", "x", "--expected-source-identity", "x", "--admin-bin", "x", "--runtime-container", "x"]
        try: gate.main(); raise AssertionError("source admission failure accepted")
        except SystemExit as error: assert "source identity differs" in str(error)
        assert not output.exists()
    finally: gate.subprocess.run, sys.argv = original_run, previous

with tempfile.TemporaryDirectory() as directory:
    output = Path(directory) / "must-not-exist"
    before, after = complete_admission(), complete_admission(); after["webDigest"] = "9" * 64
    try: gate.assert_admission_stable(before, after); raise AssertionError("post-run admission change accepted")
    except RuntimeError as error: assert "admission changed" in str(error)
    assert not output.exists()
