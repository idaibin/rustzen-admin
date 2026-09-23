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
            receipt = json.loads(self.receipt.read_text().splitlines()[-1])
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
