#!/usr/bin/env python3
"""Pinned verifier Python provides a dependency-free same-origin static/API proxy."""
import argparse
import http.client
import json
import mimetypes
import os
import pathlib
import stat
import threading
import time
import urllib.parse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

HOP = {"connection", "keep-alive", "proxy-authenticate", "proxy-authorization", "te", "trailers", "transfer-encoding", "upgrade"}
lock = threading.Lock()


def safe_response_header(key, value):
    if "\r" in key or "\n" in key or "\r" in value or "\n" in value:
        raise ValueError("upstream response header contains a line break")
    return key, value


def read_static_member(root, member):
    # Walk from the root directory descriptor: every component is opened
    # exactly once, relative to a pinned directory, with symlink following
    # disabled, and the bytes are read from the opened descriptor. No name is
    # resolved again after the open, so swapping directory entries between
    # checks cannot redirect what is served; only objects physically reachable
    # from the root directory without following links can ever be served.
    directory = os.open(root, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        for name in member[len(root) + 1:].split("/"):
            if name in ("", ".", ".."):
                return None
            following = os.open(name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=directory)
            os.close(directory)
            directory = following
        if not stat.S_ISREG(os.fstat(directory).st_mode):
            return None
        handle = os.fdopen(directory, "rb")
        directory = None
        try:
            return handle.read(), member.rsplit("/", 1)[-1]
        finally:
            handle.close()
    except (OSError, ValueError):
        return None
    finally:
        if directory is not None:
            os.close(directory)


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
                if key.lower() not in HOP and key.lower() != "content-length": self.send_header(*safe_response_header(key, value))
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
            if key.lower() not in HOP and key.lower() != "content-length": self.send_header(*safe_response_header(key, value))
        self.send_header("content-length", str(len(payload))); self.end_headers(); self.receipt(response.status)
        self.wfile.write(payload); self.wfile.flush()
        ready_file = mode.get("readyFile")
        if parsed.path == "/api/notifications" and count == 1 and response.status == 200 and ready_file:
            pathlib.Path(ready_file).write_text(json.dumps({"case": mode.get("name"), "path": parsed.path,
                                                           "status": 200, "readyAtNs": time.time_ns()}, separators=(",", ":")))

    def static(self):
        root = os.path.realpath(self.server.web_root)
        requested = urllib.parse.urlsplit(self.path).path.lstrip("/") or "index.html"
        body = None; served = None
        # The realpath-plus-prefix screen resolves legitimate in-root symlinks
        # and rejects paths escaping the root; the descriptor walk inside
        # read_static_member is the enforcement that survives concurrent name
        # swaps. Symlinks resolving outside the root, or swapped in after the
        # screen, fall back to the SPA index.
        for component in (requested, "index.html"):
            try:
                candidate = os.path.realpath(os.path.join(root, component))
                if not candidate.startswith(root + os.sep):
                    continue
                member = read_static_member(root, candidate)
            except (OSError, ValueError):
                member = None
            if member is not None:
                body, served = member
                break
        if body is None:
            self.send_error(404)
            return
        self.send_response(200)
        self.send_header(*safe_response_header("content-type", mimetypes.guess_type(served)[0] or "application/octet-stream"))
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
