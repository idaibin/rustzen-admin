#!/usr/bin/env python3
"""Route-exact Monitoring read fixture for the Linux Chromium state gate.

The four Monitoring collection reads are synthetic. Every other request is
proxied to Admin so login, permissions, Reports, and the application shell stay
inside the real disposable runtime.
"""

from http.client import HTTPConnection
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from threading import Event, Lock
import time
from urllib.parse import parse_qs, urlparse

from monitoring_ui_state_fixture_payloads import route_payload


UPSTREAM_HOST = os.environ.get("RUSTZEN_MONITORING_FIXTURE_UPSTREAM", "127.0.0.1")
UPSTREAM_PORT = int(os.environ.get("RUSTZEN_MONITORING_FIXTURE_UPSTREAM_PORT", "19801"))
PORT = int(os.environ.get("RUSTZEN_MONITORING_FIXTURE_PORT", "19806"))
SLOW_SECONDS = float(os.environ.get("RUSTZEN_MONITORING_FIXTURE_SLOW_SECONDS", "5"))
HOLD_TIMEOUT_SECONDS = float(
    os.environ.get("RUSTZEN_MONITORING_FIXTURE_HOLD_TIMEOUT_SECONDS", "30")
)
if not 0 < HOLD_TIMEOUT_SECONDS <= 120:
    raise ValueError("monitor fixture hold timeout must be in (0, 120]")


def safe_response_header(key, value):
    if "\r" in key or "\n" in key or "\r" in value or "\n" in value:
        raise ValueError("upstream response header contains a line break")
    return key, value


ROUTES = {
    "/api/monitor/overview": "overview",
    "/api/monitor/nodes": "nodes",
    "/api/monitor/incidents": "incidents",
    "/api/monitor/daily-summaries": "summaries",
}
MODES = {"slow", "success", "403", "500"}
FAIL_AFTER_STATUSES = {"403", "500"}
lock = Lock()
state = {
    name: {
        "mode": "success",
        "fail_after_first_status": None,
        "reads": 0,
        "hold_slow": False,
        "release_event": Event(),
    }
    for name in ROUTES.values()
}
requests = []
control_events = []


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *_args):
        return

    def send_json(self, status, body):
        raw = json.dumps(body, separators=(",", ":")).encode()
        self.send_response(status)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(raw)))
        self.end_headers()
        self.wfile.write(raw)

    def proxy(self):
        size = int(self.headers.get("content-length", "0"))
        body = self.rfile.read(size) if size else None
        headers = {
            key: value
            for key, value in self.headers.items()
            if key.lower() not in {"host", "connection", "content-length"}
        }
        if body is not None:
            headers["content-length"] = str(len(body))
        connection = HTTPConnection(UPSTREAM_HOST, UPSTREAM_PORT, timeout=20)
        connection.request(self.command, self.path, body=body, headers=headers)
        response = connection.getresponse()
        payload = response.read()
        self.send_response(response.status)
        for key, value in response.getheaders():
            if key.lower() not in {"connection", "transfer-encoding", "content-length"}:
                self.send_header(*safe_response_header(key, value))
        self.send_header("content-length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)
        connection.close()

    def monitoring_response(self, route, name, query):
        with lock:
            entry = state[name]
            mode = entry["mode"]
            hold_slow = mode == "slow" and entry["hold_slow"]
            release_event = entry["release_event"]
            entry["hold_slow"] = False
            entry["reads"] += 1
            request = {
                "route": route,
                "name": name,
                "mode": mode,
                "query": query,
                "read": entry["reads"],
                "receivedAtUtc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
                "receivedAtEpochMs": time.time_ns() // 1_000_000,
                "holdRequested": hold_slow,
            }
            requests.append(request)
            next_status = entry["fail_after_first_status"]
            if mode == "success" and next_status is not None:
                entry["mode"] = next_status
                entry["fail_after_first_status"] = None
        if mode == "slow":
            if hold_slow:
                released = release_event.wait(HOLD_TIMEOUT_SECONDS)
                request["holdOutcome"] = "released" if released else "timeout"
                request["holdFinishedAtUtc"] = time.strftime(
                    "%Y-%m-%dT%H:%M:%SZ", time.gmtime()
                )
                request["holdFinishedAtEpochMs"] = time.time_ns() // 1_000_000
            else:
                time.sleep(SLOW_SECONDS)
        if mode in FAIL_AFTER_STATUSES:
            self.send_json(
                int(mode),
                {"code": 40002, "message": f"fixture {mode}", "data": None},
            )
            return
        self.send_json(
            200,
            {"code": 0, "message": "Success", "data": route_payload(name, query)},
        )

    def handle_request(self):
        parsed = urlparse(self.path)
        if parsed.path == "/__monitoring_fixture/health":
            self.send_json(200, {"status": "ok"})
            return
        if parsed.path == "/__monitoring_fixture/receipt":
            with lock:
                receipt = {
                    "requests": list(requests),
                    "controlEvents": list(control_events),
                }
            self.send_json(200, receipt)
            return
        name = ROUTES.get(parsed.path)
        if name is not None and self.command == "GET":
            self.monitoring_response(parsed.path, name, parse_qs(parsed.query))
            return
        self.proxy()

    do_GET = handle_request
    do_POST = handle_request
    do_PUT = handle_request
    do_DELETE = handle_request

    def do_PATCH(self):
        if self.path not in {
            "/__monitoring_fixture/mode",
            "/__monitoring_fixture/hold",
            "/__monitoring_fixture/release",
        }:
            self.proxy()
            return
        size = int(self.headers.get("content-length", "0"))
        try:
            payload = json.loads(self.rfile.read(size) or b"{}")
        except json.JSONDecodeError:
            self.send_json(400, {"error": "invalid JSON"})
            return
        if self.path in {
            "/__monitoring_fixture/hold",
            "/__monitoring_fixture/release",
        }:
            route_name = payload.get("route")
            trigger = payload.get("trigger", "test")
            if (
                set(payload) - {"route", "trigger"}
                or "route" not in payload
                or route_name not in state
                or trigger not in {"test", "assertText-succeeded", "cleanup"}
            ):
                self.send_json(400, {"error": "invalid fixture route"})
                return
            with lock:
                entry = state[route_name]
                if self.path.endswith("/hold"):
                    if entry["hold_slow"]:
                        self.send_json(409, {"error": "fixture hold already pending"})
                        return
                    entry["hold_slow"] = True
                    entry["release_event"] = Event()
                    event = "hold"
                else:
                    entry["release_event"].set()
                    event = "release"
                control_events.append(
                    {
                        "event": event,
                        "route": route_name,
                        "atUtc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
                        "atEpochMs": time.time_ns() // 1_000_000,
                        "trigger": trigger,
                    }
                )
            self.send_json(200, {"status": "ok", "event": event})
            return
        allowed = {
            name for route_name in ROUTES.values() for name in (
                route_name,
                f"{route_name}FailAfterFirstStatus",
            )
        }
        if not payload or set(payload) - allowed:
            self.send_json(400, {"error": "invalid fixture key"})
            return
        for key, value in payload.items():
            route_name = key.removesuffix("FailAfterFirstStatus")
            allowed_values = FAIL_AFTER_STATUSES if key != route_name else MODES
            if value is not None and value not in allowed_values:
                self.send_json(400, {"error": "invalid fixture mode"})
                return
        with lock:
            for key, value in payload.items():
                route_name = key.removesuffix("FailAfterFirstStatus")
                field = "fail_after_first_status" if key != route_name else "mode"
                state[route_name][field] = value
        self.send_json(200, {"status": "ok"})


ThreadingHTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
