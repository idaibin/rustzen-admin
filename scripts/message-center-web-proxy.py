#!/usr/bin/env python3
"""Pinned verifier Python provides a dependency-free same-origin static/API proxy."""
import argparse
import http.client
import json
import mimetypes
import os
import pathlib
import threading
import time
import urllib.parse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

HOP = {"connection", "keep-alive", "proxy-authenticate", "proxy-authorization", "te", "trailers", "transfer-encoding", "upgrade"}
lock = threading.Lock()


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *_args):
        return

    def mode(self):
        try:
            return json.loads(pathlib.Path(self.server.mode_file).read_text())
        except (OSError, ValueError):
            return {}

    def receipt(self, status):
        parsed = urllib.parse.urlsplit(self.path)
        row = {"case": self.mode().get("name", "default"), "method": self.command, "path": parsed.path, "query": bool(parsed.query),
               "tokenInUrl": "token=" in parsed.query.lower(),
               "bearer": self.headers.get("authorization", "").startswith("Bearer "),
               "accept": self.headers.get("accept", ""), "status": status, "atNs": time.time_ns()}
        with lock:
            with open(self.server.receipt, "a", encoding="utf-8") as output:
                output.write(json.dumps(row, separators=(",", ":")) + "\n")

    def diagnostic(self, event, **fields):
        parsed = urllib.parse.urlsplit(self.path)
        print(json.dumps({"proxyEvent": event, "case": self.mode().get("name", "default"),
                          "method": self.command, "path": parsed.path, **fields},
                         separators=(",", ":")), flush=True)

    def json_error(self, status, code):
        body = json.dumps({"code": code, "message": "gate fault", "data": None}).encode()
        self.send_response(status); self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(body))); self.end_headers(); self.wfile.write(body)
        self.receipt(status)

    def empty(self, status):
        self.send_response(status); self.send_header("content-length", "0"); self.end_headers()
        self.receipt(status)

    def proxy(self):
        mode = self.mode(); parsed = urllib.parse.urlsplit(self.path)
        is_stream_request = parsed.path == "/api/notifications/stream"
        if is_stream_request:
            self.diagnostic("sse.request", bearer=self.headers.get("authorization", "").startswith("Bearer "))
        count_key = (mode.get("name", "default"), parsed.path)
        with lock:
            self.server.counts[count_key] = self.server.counts.get(count_key, 0) + 1
            count = self.server.counts[count_key]
        if parsed.path == "/api/notifications/stream" and mode.get("sse401"):
            return self.json_error(401, 40101)
        if parsed.path == "/api/notifications/stream" and mode.get("sse204"):
            return self.empty(204)
        if parsed.path == "/api/notifications/unread-count" and mode.get("auth401"):
            return self.json_error(401, 40101)
        if parsed.path == "/api/notifications" and count > mode.get("list403After", 10**9):
            return self.json_error(403, 40301)
        if parsed.path == "/api/notifications" and count == 1:
            time.sleep(mode.get("listDelayMs", 0) / 1000)
        body = self.rfile.read(int(self.headers.get("content-length", 0)))
        connection = http.client.HTTPConnection("127.0.0.1", self.server.backend_port, timeout=60)
        headers = {key: value for key, value in self.headers.items() if key.lower() not in HOP}
        try:
            connection.request(self.command, self.path, body=body, headers=headers)
            response = connection.getresponse()
        except (OSError, http.client.HTTPException) as error:
            self.diagnostic("upstream-error", errorType=type(error).__name__, stream=is_stream_request)
            connection.close(); raise
        is_sse = response.headers.get_content_type() == "text/event-stream"
        if is_stream_request:
            self.diagnostic("sse.response", status=response.status,
                            contentType=response.headers.get_content_type())
        if is_sse:
            self.send_response(response.status)
            for key, value in response.getheaders():
                if key.lower() not in HOP and key.lower() != "content-length": self.send_header(key, value)
            self.send_header("connection", "close"); self.end_headers(); self.receipt(response.status)
            ready_file = mode.get("streamReadyFile")
            if ready_file:
                pathlib.Path(ready_file).write_text(json.dumps({"case": mode.get("name"),
                    "path": parsed.path, "status": response.status, "readyAtNs": time.time_ns()}, separators=(",", ":")))
            try:
                preflight_payload = b""
                while True:
                    chunk = response.read1(4096)
                    if not chunk: break
                    self.wfile.write(chunk); self.wfile.flush()
                    if mode.get("name") == "preflight":
                        preflight_payload += chunk
                        normalized = preflight_payload.replace(b"\r\n", b"\n")
                        if normalized.count(b"\n\n") >= 2: break
            except (OSError, http.client.HTTPException) as error:
                self.diagnostic("stream-error", errorType=type(error).__name__)
            connection.close(); self.close_connection = True
            self.diagnostic("sse.closed", case=mode.get("name", "default"))
            return
        try:
            payload = response.read()
        except (OSError, http.client.HTTPException) as error:
            self.diagnostic("upstream-error", errorType=type(error).__name__, stream=False)
            connection.close(); raise
        connection.close(); self.send_response(response.status)
        for key, value in response.getheaders():
            if key.lower() not in HOP and key.lower() != "content-length": self.send_header(key, value)
        self.send_header("content-length", str(len(payload))); self.end_headers(); self.receipt(response.status)
        self.wfile.write(payload); self.wfile.flush()
        ready_file = mode.get("readyFile")
        if parsed.path == "/api/notifications" and count == 1 and response.status == 200 and ready_file:
            pathlib.Path(ready_file).write_text(json.dumps({"case": mode.get("name"), "path": parsed.path,
                                                           "status": 200, "readyAtNs": time.time_ns()}, separators=(",", ":")))

    def static(self):
        path = urllib.parse.urlsplit(self.path).path.lstrip("/") or "index.html"
        root = pathlib.Path(self.server.web_root).resolve(); target = (root / path).resolve()
        if root not in target.parents or not target.is_file(): target = root / "index.html"
        body = target.read_bytes(); self.send_response(200)
        self.send_header("content-type", mimetypes.guess_type(target.name)[0] or "application/octet-stream")
        self.send_header("content-length", str(len(body))); self.end_headers(); self.wfile.write(body)

    def route(self):
        parsed = urllib.parse.urlsplit(self.path)
        self.diagnostic("request-start", query=bool(parsed.query), api=parsed.path.startswith("/api/"))
        if parsed.path.startswith("/api/"): self.proxy()
        else: self.static()

    do_GET = route
    do_POST = route
    do_PUT = route
    do_PATCH = route
    do_DELETE = route


def main():
    parser = argparse.ArgumentParser(); parser.add_argument("--web-root", required=True)
    parser.add_argument("--receipt", required=True); parser.add_argument("--mode-file", required=True)
    parser.add_argument("--port", type=int, default=19800); parser.add_argument("--backend-port", type=int, default=19810)
    args = parser.parse_args(); pathlib.Path(args.receipt).write_text("")
    server = ThreadingHTTPServer(("127.0.0.1", args.port), Handler)
    server.web_root=args.web_root; server.receipt=args.receipt; server.mode_file=args.mode_file
    server.backend_port=args.backend_port; server.counts={}; server.serve_forever()


if __name__ == "__main__": main()
