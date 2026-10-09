#!/usr/bin/env python3
import http.client
import importlib.util
import contextlib
import io
import json
import pathlib
import socket
import tempfile
import threading
import time
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from unittest import mock

SCRIPT = pathlib.Path(__file__).with_name("message-center-web-proxy.py")
SPEC = importlib.util.spec_from_file_location("message_center_web_proxy", SCRIPT)
PROXY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PROXY)
JSON_BODY = b'{"code":0,"message":"ok","data":{"token":"complete"}}'


class Upstream(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *_args):
        return

    def do_GET(self):
        if self.path in {"/api/folded", "/api/folded-stream"}:
            body = b"ok"
            self.send_response(200)
            self.send_header("content-type", "text/event-stream" if self.path.endswith("stream") else "text/plain")
            self.send_header("x-upstream", "safe\r\n injected: yes")
            self.send_header("content-length", str(len(body)))
            self.end_headers(); self.wfile.write(body)
            return
        if self.path == "/api/json":
            self.send_response(200)
            self.send_header("content-type", "application/json")
            self.send_header("content-length", str(len(JSON_BODY)))
            self.end_headers(); self.wfile.write(JSON_BODY)
            return
        self.send_response(200)
        self.send_header("content-type", "text/event-stream")
        self.end_headers()
        self.wfile.write(b"event: reconcile.required\ndata: {}\n\n")
        self.wfile.write(b"event: inbox.changed\ndata: {\"revision\":1}\n\n"); self.wfile.flush()
        self.server.release.wait(2)
        self.wfile.write(b": done\n\n"); self.wfile.flush()
        self.close_connection = True


class ProxyIntegrationTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        root = pathlib.Path(cls.temp.name)
        (root / "index.html").write_text("gate")
        cls.mode = root / "mode.json"; cls.mode.write_text("{}")
        cls.receipt = root / "receipt.jsonl"; cls.receipt.write_text("")
        cls.upstream = ThreadingHTTPServer(("127.0.0.1", 0), Upstream)
        cls.upstream.release = threading.Event()
        cls.proxy = ThreadingHTTPServer(("127.0.0.1", 0), PROXY.Handler)
        cls.proxy.web_root = str(root); cls.proxy.receipt = str(cls.receipt)
        cls.proxy.mode_file = str(cls.mode); cls.proxy.backend_port = cls.upstream.server_port
        cls.proxy.counts = {}
        cls.proxy.handle_error = lambda *_args: None
        cls.threads = [threading.Thread(target=server.serve_forever, daemon=True)
                       for server in (cls.upstream, cls.proxy)]
        for thread in cls.threads: thread.start()

    @classmethod
    def tearDownClass(cls):
        cls.upstream.release.set()
        for server in (cls.proxy, cls.upstream): server.shutdown(); server.server_close()
        cls.temp.cleanup()

    def connect(self):
        return http.client.HTTPConnection("127.0.0.1", self.proxy.server_port, timeout=1)

    def test_json_response_has_exact_length_and_completes_without_eof(self):
        connection = self.connect(); connection.request("GET", "/api/json")
        response = connection.getresponse()
        self.assertEqual(response.headers.get("content-length"), str(len(JSON_BODY)))
        self.assertEqual(response.read(), JSON_BODY)
        connection.close()

    def test_folded_upstream_header_is_not_forwarded(self):
        for path in ("/api/folded", "/api/folded-stream"):
            with self.subTest(path=path):
                connection = self.connect(); connection.request("GET", path)
                with self.assertRaises(http.client.RemoteDisconnected):
                    connection.getresponse()
                connection.close()

    def test_static_root_keeps_assets_inside_root(self):
        root = pathlib.Path(self.temp.name)
        (root / "app.js").write_text("window.fixture = true")
        for path, expected in (("/app.js", b"window.fixture = true"),
                               ("/missing", b"gate"),
                               ("/../outside", b"gate")):
            with self.subTest(path=path):
                connection = self.connect(); connection.request("GET", path)
                response = connection.getresponse()
                self.assertEqual(response.status, 200)
                content_type = response.getheader("content-type")
                self.assertIn(content_type, {"text/javascript", "application/javascript"}
                              if path == "/app.js" else {"text/html"})
                self.assertEqual(response.read(), expected)
                connection.close()
        with tempfile.TemporaryDirectory() as outside:
            (pathlib.Path(outside) / "secret.txt").write_text("outside-secret")
            (root / "escape").symlink_to(outside, target_is_directory=True)
            connection = self.connect(); connection.request("GET", "/escape/secret.txt")
            response = connection.getresponse()
            self.assertEqual(response.status, 200)
            self.assertEqual(response.read(), b"gate")
            connection.close()
            index = root / "index.html"
            index.unlink(); index.symlink_to(pathlib.Path(outside) / "secret.txt")
            try:
                connection = self.connect(); connection.request("GET", "/missing")
                response = connection.getresponse()
                self.assertEqual(response.status, 404)
                self.assertNotIn(b"outside-secret", response.read())
                connection.close()
            finally:
                index.unlink(); index.write_text("gate")
                (root / "escape").unlink()

    def test_static_serves_nested_member_and_in_root_symlink(self):
        root = pathlib.Path(self.temp.name)
        (root / "assets").mkdir(exist_ok=True)
        (root / "assets" / "nested.js").write_text("nested-member")
        (root / "alias.js").symlink_to(root / "assets" / "nested.js")
        try:
            for path in ("/assets/nested.js", "/alias.js"):
                with self.subTest(path=path):
                    connection = self.connect(); connection.request("GET", path)
                    response = connection.getresponse()
                    self.assertEqual(response.status, 200)
                    self.assertIn(response.getheader("content-type"),
                                  {"text/javascript", "application/javascript"})
                    self.assertEqual(response.read(), b"nested-member")
                    connection.close()
        finally:
            (root / "alias.js").unlink()

    def test_static_read_is_bound_to_the_opened_descriptor(self):
        # Deterministic interleave: swap the served name for a symlink to an
        # outside file between the open and the read. The response must still
        # carry the bytes of the already-opened member.
        root = pathlib.Path(self.temp.name)
        bound = root / "bound.js"; bound.write_text("bound-original")
        real_fstat = PROXY.os.fstat
        with tempfile.TemporaryDirectory() as outside:
            secret = pathlib.Path(outside) / "secret.txt"; secret.write_text("outside-secret")
            def swap_then_fstat(fd):
                bound.unlink(); bound.symlink_to(secret)
                return real_fstat(fd)
            try:
                with mock.patch.object(PROXY.os, "fstat", side_effect=swap_then_fstat):
                    connection = self.connect(); connection.request("GET", "/bound.js")
                    response = connection.getresponse()
                    self.assertEqual(response.status, 200)
                    self.assertEqual(response.read(), b"bound-original")
                    connection.close()
            finally:
                if bound.is_symlink(): bound.unlink()
                bound.write_text("bound-original")

    def test_static_swap_between_resolve_and_open_needs_local_write(self):
        # The remaining window is a local rename between resolve() and open().
        # The proxy has no HTTP write surface, so this forced interleave is the
        # minimal primitive the residual race requires; kept open by the
        # in-root symlink compatibility (no per-component O_NOFOLLOW walk).
        root = pathlib.Path(self.temp.name)
        window = root / "window.js"; window.write_text("window-original")
        real_resolve = pathlib.Path.resolve
        with tempfile.TemporaryDirectory() as outside:
            secret = pathlib.Path(outside) / "secret.txt"; secret.write_text("outside-secret")
            def swap_after_resolve(self, *args, **kwargs):
                result = real_resolve(self, *args, **kwargs)
                if self.name == "window.js" and not self.is_symlink():
                    window.unlink(); window.symlink_to(secret)
                return result
            try:
                with mock.patch.object(pathlib.Path, "resolve", swap_after_resolve):
                    connection = self.connect(); connection.request("GET", "/window.js")
                    response = connection.getresponse()
                    self.assertEqual(response.status, 200)
                    self.assertEqual(response.read(), b"outside-secret")
                    connection.close()
                # Without the interleave the swapped name is just a statically
                # present outside symlink: resolve() confines it and the SPA
                # fallback is served instead.
                connection = self.connect(); connection.request("GET", "/window.js")
                response = connection.getresponse()
                self.assertEqual(response.status, 200)
                self.assertEqual(response.read(), b"gate")
                connection.close()
            finally:
                if window.is_symlink(): window.unlink()
                window.write_text("window-original")

    def test_sse_is_streamed_without_content_length(self):
        connection = self.connect(); started = time.monotonic()
        diagnostics = io.StringIO()
        with contextlib.redirect_stdout(diagnostics):
            connection.request("GET", "/api/notifications/stream", headers={
                "accept": "text/event-stream", "authorization": "Bearer never-log-this-secret"})
            response = connection.getresponse()
            self.assertIsNone(response.headers.get("content-length"))
            self.assertEqual(response.readline(), b"event: reconcile.required\n")
        self.assertLess(time.monotonic() - started, 0.75)
        log = diagnostics.getvalue()
        self.assertIn('"proxyEvent":"request-start"', log)
        self.assertIn('"proxyEvent":"sse.request"', log)
        self.assertIn('"proxyEvent":"sse.response"', log)
        self.assertNotIn("never-log-this-secret", log)
        self.upstream.release.set(); connection.close()

    def test_preflight_closes_after_two_complete_frames(self):
        self.mode.write_text('{"name":"preflight"}')
        connection = self.connect(); started = time.monotonic()
        connection.request("GET", "/api/notifications/stream", headers={
            "accept": "text/event-stream", "authorization": "Bearer preflight-token"})
        response = connection.getresponse(); payload = response.read(); connection.close()
        self.assertEqual(payload.count(b"\n\n"), 2)
        self.assertLess(time.monotonic() - started, 0.75)
        self.mode.write_text("{}")

    def test_browser_stream_faults_are_receipted_without_upstream(self):
        for mode, status in (({"name": "stopped", "sse204": True}, 204),
                             ({"name": "expired", "sse401": True}, 401)):
            self.mode.write_text(json.dumps(mode)); connection = self.connect()
            connection.request("GET", "/api/notifications/stream", headers={
                "accept": "text/event-stream", "authorization": "Bearer browser-token"})
            response = connection.getresponse(); response.read(); connection.close()
            self.assertEqual(response.status, status)
            deadline = time.monotonic() + 1
            while True:
                lines = self.receipt.read_text().splitlines()
                receipt = json.loads(lines[-1]) if lines else None
                if receipt and receipt["case"] == mode["name"] and receipt["status"] == status:
                    break
                if time.monotonic() >= deadline:
                    self.fail(f"missing receipt for {mode['name']}: {receipt}")
                time.sleep(0.01)
            self.assertEqual((receipt["case"], receipt["status"]), (mode["name"], status))
            self.assertGreater(receipt["atNs"], 0)
        self.mode.write_text("{}")

    def test_upstream_error_is_flushed_without_authorization(self):
        unavailable = socket.socket(); unavailable.bind(("127.0.0.1", 0))
        unavailable_port = unavailable.getsockname()[1]; unavailable.close()
        previous = self.proxy.backend_port; self.proxy.backend_port = unavailable_port
        diagnostics = io.StringIO(); connection = self.connect()
        try:
            with contextlib.redirect_stdout(diagnostics):
                connection.request("GET", "/api/failure", headers={
                    "authorization": "Bearer upstream-secret-must-not-appear"})
                with self.assertRaises(http.client.RemoteDisconnected): connection.getresponse()
        finally:
            self.proxy.backend_port = previous; connection.close()
        log = diagnostics.getvalue()
        self.assertIn('"proxyEvent":"request-start"', log)
        self.assertIn('"proxyEvent":"upstream-error"', log)
        self.assertNotIn("upstream-secret-must-not-appear", log)


if __name__ == "__main__":
    unittest.main()
