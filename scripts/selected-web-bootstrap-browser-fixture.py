#!/usr/bin/env python3
"""Loopback-only proxy faults for the selected-Web Chromium gate.

Python is used here because the pinned verifier image already contains it and the
standard library can provide a no-dependency HTTP proxy/receipt server.
"""
import http.client
import json
import re
import socket
import threading
import time
import urllib.parse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


class Fixture:
    def __init__(self, admin_url, case):
        self.admin = urllib.parse.urlsplit(admin_url)
        self.case = case
        self.requests = []

    def handler(self):
        fixture = self

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *_): pass

            def _record(self):
                fixture.requests.append({
                    "method": self.command, "path": self.path,
                    "authorization": bool(self.headers.get("authorization")),
                    "cookie": bool(self.headers.get("cookie")),
                })

            def do_GET(self): self._handle()
            def do_POST(self): self._handle()

            def _handle(self):
                self._record()
                path = urllib.parse.urlsplit(self.path).path
                if (fixture.case == "bindingDelayed" and path == "/__web-binding") or (fixture.case == "entryDelayed" and path.startswith("/assets/")):
                    time.sleep(3)
                if fixture.case == "bindingNetworkFailure" and path == "/__web-binding":
                    self.connection.shutdown(socket.SHUT_RDWR)
                    self.connection.close()
                    return
                length = int(self.headers.get("content-length", "0"))
                body = self.rfile.read(length) if length else None
                headers = {key: value for key, value in self.headers.items()
                           if key.lower() not in {"host", "connection", "content-length"}}
                connection = http.client.HTTPConnection(fixture.admin.hostname, fixture.admin.port or 80, timeout=15)
                connection.request(self.command, self.path, body=body, headers=headers)
                response = connection.getresponse()
                response_body = response.read()
                response_headers = response.getheaders()
                connection.close()
                if fixture.case == "bindingMismatch" and path == "/__web-binding" and response.status == 200:
                    response_body = json.dumps({"bindingVersion": 1, "webDigest": "b" * 64}).encode()
                    response_headers = [("content-type", "application/json"), ("cache-control", "no-store")]
                if fixture.case in {"sriEntryFailure", "sriIntegrityRemoved"} and path.startswith("/assets/") and path.endswith(".js"):
                    response_body = b"globalThis.__rz_sri_tamper_executed = true;"
                    response_headers = [("content-type", "text/javascript")]
                if fixture.case == "sriIntegrityRemoved" and path == "/monitoring/nodes":
                    response_body, count = re.subn(rb"entry\.integrity=[^;]+;", b"entry.integrity=undefined;", response_body)
                    if count != 1: raise RuntimeError("bootstrap integrity slot is missing")
                self.send_response(response.status)
                for key, value in response_headers:
                    if key.lower() not in {"connection", "content-length", "transfer-encoding"}:
                        self.send_header(key, value)
                self.send_header("content-length", str(len(response_body)))
                if self.command == "GET" and not path.startswith("/api/") and path != "/__web-binding":
                    self.send_header("set-cookie", "rz_bootstrap_proof=1; Path=/; SameSite=Lax")
                self.end_headers()
                try:
                    self.wfile.write(response_body)
                except BrokenPipeError:
                    pass

        return Handler


def serve(admin_url, case):
    fixture = Fixture(admin_url, case)
    server = ThreadingHTTPServer(("127.0.0.1", 0), fixture.handler())
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    return server, fixture
