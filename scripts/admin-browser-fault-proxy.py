#!/usr/bin/env python3
"""Container-only HTTP forward proxy for deterministic browser failure cases."""
import http.client
import json
import os
import signal
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

UPSTREAM_HOST = "127.0.0.1"
UPSTREAM_PORT = int(os.environ.get("RUSTZEN_VERIFY_PROXY_UPSTREAM_PORT", "19801"))
LISTEN_PORT = int(os.environ.get("RUSTZEN_VERIFY_PROXY_LISTEN_PORT", "19805"))
MODE = os.environ["RUSTZEN_VERIFY_FAULT_MODE"]
ROUTE = os.environ["RUSTZEN_VERIFY_FAULT_ROUTE"]
METHOD = os.environ["RUSTZEN_VERIFY_FAULT_METHOD"]
RECEIPT = os.environ["RUSTZEN_VERIFY_FAULT_RECEIPT"]
HITS = 0
HITS_LOCK = threading.Lock()
TRANSITION_RUN_ID = None
TRANSITION_READS = 0


def write_receipt(*_args):
    with HITS_LOCK:
        hits = HITS
    with open(RECEIPT, "w", encoding="utf-8") as receipt:
        json.dump({"method": METHOD, "mode": MODE, "route": ROUTE, "hitCount": hits, "transitionRunId": TRANSITION_RUN_ID, "transitionReads": TRANSITION_READS}, receipt)
    raise SystemExit(0)


class Proxy(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *_args):
        pass

    def _forward(self):
        length = int(self.headers.get("content-length", "0"))
        body = self.rfile.read(length) if length else None
        headers = {key: value for key, value in self.headers.items() if key.lower() not in {"host", "connection"}}
        connection = http.client.HTTPConnection(UPSTREAM_HOST, UPSTREAM_PORT, timeout=20)
        connection.request(self.command, self.path, body=body, headers=headers)
        response = connection.getresponse()
        return response.status, response.reason, response.getheaders(), response.read()

    def _respond(self, status, reason, headers, body):
        self.send_response(status, reason)
        for key, value in headers:
            if key.lower() not in {"connection", "transfer-encoding", "content-length"}:
                self.send_header(key, value)
        self.send_header("content-length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _transition(self, path):
        global HITS, TRANSITION_RUN_ID, TRANSITION_READS
        status, reason, headers, body = self._forward()
        with HITS_LOCK:
            if self.command == "POST" and path == f"{ROUTE}/cleanup-operation-logs-retention/run":
                HITS += 1
                TRANSITION_RUN_ID = json.loads(body).get("data", {}).get("id")
            elif self.command == "GET" and path == ROUTE and TRANSITION_RUN_ID and TRANSITION_READS == 0:
                HITS += 1
                TRANSITION_READS += 1
                payload = json.loads(body)
                for task in payload.get("data", []):
                    if task.get("taskKey") == "cleanup-operation-logs-retention":
                        task["running"] = True
                        task["lastStatus"] = "running"
                body = json.dumps(payload, separators=(",", ":")).encode("utf-8")
            elif self.command == "GET" and path == ROUTE and TRANSITION_RUN_ID:
                HITS += 1
                TRANSITION_READS += 1
        self._respond(status, reason, headers, body)

    def _handle(self):
        path = self.path.split("?", 1)[0]
        if path == "/__verify_proxy_health":
            self.send_response(204)
            self.end_headers()
            return
        if MODE == "task-transition" and (
            path == ROUTE or path == f"{ROUTE}/cleanup-operation-logs-retention/run"
        ):
            self._transition(path)
            return
        if self.command == METHOD and path == ROUTE:
            global HITS
            with HITS_LOCK:
                HITS += 1
            if MODE == "network":
                # Send headers and only one byte of a longer body. Chromium has
                # received a response, so it will not transparently replay the
                # mutation, while body decoding still fails as a network error.
                self.connection.sendall(
                    b"HTTP/1.1 200 OK\r\n"
                    b"content-type: application/json\r\n"
                    b"content-length: 64\r\n"
                    b"connection: close\r\n\r\n{"
                )
                self.close_connection = True
                return
            if MODE == "http":
                body = b'{"code":500,"message":"browser fault injection"}'
                self.send_response(500)
                self.send_header("content-type", "application/json")
                self.send_header("content-length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if MODE == "empty":
                body = b'{"code":0,"message":"Success","data":[],"total":0}'
                self.send_response(200)
                self.send_header("content-type", "application/json")
                self.send_header("content-length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
        self._respond(*self._forward())

    do_GET = _handle
    do_POST = _handle
    do_PUT = _handle
    do_DELETE = _handle


if __name__ == "__main__":
    signal.signal(signal.SIGTERM, write_receipt)
    signal.signal(signal.SIGINT, write_receipt)
    ThreadingHTTPServer(("127.0.0.1", LISTEN_PORT), Proxy).serve_forever()
