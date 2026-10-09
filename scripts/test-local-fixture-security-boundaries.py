#!/usr/bin/env python3
"""Loopback checks for the verifier fixtures' upstream and TLS boundaries."""

import http.client
import importlib.util
import os
import pathlib
import socket
import subprocess
import tempfile
import threading
import time
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from unittest import mock


ROOT = pathlib.Path(__file__).resolve().parent


def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / (name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def free_port():
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


class Upstream(BaseHTTPRequestHandler):
    def log_message(self, *_args):
        pass

    def do_GET(self):
        body = b"upstream-ok"
        self.send_response(200)
        value = "safe\r\n injected: yes" if self.path == "/header-folded" else "safe"
        self.send_header("x-upstream", value)
        self.send_header("content-length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


class BoundaryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.upstream = ThreadingHTTPServer(("127.0.0.1", 0), Upstream)
        cls.thread = threading.Thread(target=cls.upstream.serve_forever, daemon=True)
        cls.thread.start()

    @classmethod
    def tearDownClass(cls):
        cls.upstream.shutdown()
        cls.upstream.server_close()

    def check_proxy(self, port):
        connection = http.client.HTTPConnection("127.0.0.1", port, timeout=3)
        try:
            connection.request("GET", "/header-normal")
            response = connection.getresponse()
            self.assertEqual((response.status, response.getheader("x-upstream"), response.read()),
                             (200, "safe", b"upstream-ok"))
        finally:
            connection.close()
        connection = http.client.HTTPConnection("127.0.0.1", port, timeout=3)
        try:
            connection.request("GET", "/header-folded")
            with self.assertRaises(http.client.RemoteDisconnected):
                connection.getresponse()
        finally:
            connection.close()

    def test_selected_web_proxy_forwards_normal_and_rejects_folded(self):
        module = load("selected-web-bootstrap-browser-fixture")
        server, _fixture = module.serve(f"http://127.0.0.1:{self.upstream.server_port}", "success")
        server.handle_error = lambda *_args: None
        try:
            self.check_proxy(server.server_port)
            for key, value in (("bad\r\nkey", "safe"), ("x-test", "bad\nvalue")):
                with self.assertRaises(ValueError):
                    module.safe_response_header(key, value)
        finally:
            server.shutdown()
            server.server_close()

    def test_analytics_and_monitoring_forwarding_boundaries(self):
        for name, prefix in (("analytics-ui-state-fixture", "ANALYTICS"),
                             ("monitoring-ui-state-fixture", "MONITORING")):
            with self.subTest(name=name):
                port = free_port()
                environment = os.environ | {
                    f"RUSTZEN_{prefix}_FIXTURE_PORT": str(port),
                    f"RUSTZEN_{prefix}_FIXTURE_UPSTREAM": "127.0.0.1",
                    f"RUSTZEN_{prefix}_FIXTURE_UPSTREAM_PORT": str(self.upstream.server_port),
                }
                process = subprocess.Popen(["python3", "-B", str(ROOT / (name + ".py"))],
                                           env=environment, stdout=subprocess.DEVNULL,
                                           stderr=subprocess.DEVNULL)
                try:
                    for _attempt in range(100):
                        if process.poll() is not None:
                            self.fail(f"{name} exited before becoming ready")
                        try:
                            self.check_proxy(port)
                            break
                        except (ConnectionRefusedError, ConnectionResetError):
                            time.sleep(0.02)
                    else:
                        self.fail(f"{name} did not become ready")
                finally:
                    process.terminate()
                    try:
                        process.wait(timeout=3)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=5)

    def test_agent_tls_context_requires_tls_1_2(self):
        module = load("monitor-agent-pairing-fixture")
        with tempfile.TemporaryDirectory() as directory:
            certificate = pathlib.Path(directory) / "certificate.pem"
            private_key = pathlib.Path(directory) / "private-key.pem"
            subprocess.run(["openssl", "req", "-x509", "-newkey", "rsa:2048", "-nodes",
                            "-keyout", str(private_key), "-out", str(certificate), "-days", "1",
                            "-subj", "/CN=localhost"], check=True, stdout=subprocess.DEVNULL,
                           stderr=subprocess.DEVNULL)
            context = module.tls_server_context(certificate, private_key)
            self.assertGreaterEqual(context.minimum_version, module.ssl.TLSVersion.TLSv1_2)
            class OldDefaultContext:
                minimum_version = module.ssl.TLSVersion.TLSv1

                def load_cert_chain(self, cert, key):
                    self.loaded = (cert, key)

            old_default = OldDefaultContext()
            with mock.patch.object(module.ssl, "SSLContext", return_value=old_default):
                self.assertIs(module.tls_server_context(certificate, private_key), old_default)
            self.assertEqual(old_default.minimum_version, module.ssl.TLSVersion.TLSv1_2)
            self.assertEqual(old_default.loaded, (certificate, private_key))


if __name__ == "__main__":
    unittest.main()
