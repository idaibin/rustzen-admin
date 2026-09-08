#!/usr/bin/env python3
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
    assert gate.receive(right) == {"method": "fragmented"}
finally: left.close(); right.close()

listener = socket.socket(); listener.bind(("127.0.0.1", 0)); listener.listen(1); request_line = []
def accept_handshake():
    connection, _ = listener.accept(); data = b""
    while b"\r\n\r\n" not in data: data += connection.recv(1024)
    request_line.append(data.split(b"\r\n", 1)[0]); connection.sendall(b"HTTP/1.1 101 Switching Protocols\r\n\r\n"); connection.close()
threading.Thread(target=accept_handshake, daemon=True).start()
client = gate.websocket(f"ws://127.0.0.1:{listener.getsockname()[1]}/devtools/page/target")
client.close(); listener.close()
assert request_line == [b"GET /devtools/page/target HTTP/1.1"]

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
        sys.argv = ["gate", "--admin-url", "http://example.test:80", "--password-file", str(secret), "--output", str(output), "--export-root", "x", "--release-result", "x", "--certificate", "x", "--public-key", "x", "--expected-source-identity", "x", "--admin-bin", "x", "--runtime-container", "x"]
        try: gate.main(); raise AssertionError("invalid preflight accepted")
        except SystemExit: pass
        assert not output.exists()
    finally: sys.argv = previous

# Admission failures are terminal before Chromium or evidence publication. The
# certificate verifier remains the authority; these doubles exercise its boundary.
class AdmissionArgs:
    export_root = release_result = certificate = public_key = expected_source_identity = admin_bin = "ignored"
def complete_admission(source="source"):
    return {"admission":{"certificateSha256":"4" * 64, "manifestSha256":"5" * 64, "archiveSha256":"6" * 64, "envelopeSha256":"7" * 64, "buildId":"2" * 64, "selection":{"compositionId":"3" * 64}, "binaryDigests":[{"path":"bin/rz-admin", "sha256":"8" * 64}]}, "webDigest":"1" * 64, "expectedSourceIdentity":"source", "currentSourceIdentity":source}

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
    mismatch = complete_admission("different-source")
    gate.subprocess.run = lambda *_, **__: gate.subprocess.CompletedProcess([], 0, json.dumps(mismatch), "")
    try: gate.admission(AdmissionArgs()); raise AssertionError("current source mismatch accepted")
    except SystemExit as error: assert "incomplete" in str(error)
finally: gate.subprocess.run = original_run

with tempfile.TemporaryDirectory() as directory:
    secret = Path(directory) / "secret"; secret.write_text("owner-secret\n"); secret.chmod(0o600)
    admin_bin = Path(directory) / "rz-admin"; admin_bin.write_bytes(b"different-admin")
    output = Path(directory) / "must-not-exist"
    original_admission, previous = gate.admission, sys.argv
    try:
        gate.admission = lambda _: {"adminSha256":"0" * 64, "webDigest":"1" * 64, "admission":{"buildId":"2" * 64, "selection":{"compositionId":"3" * 64}}}
        sys.argv = ["gate", "--admin-url", "http://127.0.0.1:9", "--password-file", str(secret), "--output", str(output), "--export-root", "x", "--release-result", "x", "--certificate", "x", "--public-key", "x", "--expected-source-identity", "x", "--admin-bin", str(admin_bin), "--runtime-container", "x"]
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
        sys.argv = ["gate", "--admin-url", "http://127.0.0.1:9", "--password-file", str(secret), "--output", str(output), "--export-root", "x", "--release-result", "x", "--certificate", "x", "--public-key", "x", "--expected-source-identity", "x", "--admin-bin", "x", "--runtime-container", "x"]
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
