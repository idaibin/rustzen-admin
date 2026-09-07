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
from threading import Lock
import time
from urllib.parse import parse_qs, urlparse


UPSTREAM_HOST = os.environ.get("RUSTZEN_MONITORING_FIXTURE_UPSTREAM", "127.0.0.1")
UPSTREAM_PORT = int(os.environ.get("RUSTZEN_MONITORING_FIXTURE_UPSTREAM_PORT", "19801"))
PORT = int(os.environ.get("RUSTZEN_MONITORING_FIXTURE_PORT", "19806"))
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
    name: {"mode": "success", "fail_after_first_status": None, "reads": 0}
    for name in ROUTES.values()
}
requests = []


def overview_payload():
    return {
        "registeredNodes": 1,
        "onlineNodes": 1,
        "offlineNodes": 0,
        "activeIncidents": 21,
        "latestResource": {
            "nodeId": "fixture-overview-node",
            "cpuPercent": 21.0,
            "memoryPercent": 34.0,
            "disks": [
                {
                    "mountPoint": "/",
                    "usedBytes": 55,
                    "totalBytes": 100,
                    "usagePercent": 55.0,
                }
            ],
            "collectedAt": "2026-09-07T00:00:00Z",
            "lastReceivedAt": "2026-09-07T00:00:01Z",
        },
    }


def nodes_payload():
    return [
        {
            "nodeId": "fixture-node",
            "hostname": "fixture-node",
            "agentVersion": "verify",
            "bootId": "fixture-boot",
            "sequence": 1,
            "lastReportAt": "2026-09-07T00:00:00Z",
            "lastReceivedAt": "2026-09-07T00:00:01Z",
            "status": "online",
            "alertPolicySource": "global",
            "cpuPercent": 10.0,
            "memory": {
                "usedBytes": 10,
                "totalBytes": 100,
                "usagePercent": 10.0,
            },
            "disks": [
                {
                    "mountPoint": "/",
                    "collectedAt": "2026-09-07T00:00:00Z",
                    "usedBytes": 55,
                    "totalBytes": 100,
                    "usagePercent": 55.0,
                }
            ],
            "createdAt": "2026-09-07T00:00:00Z",
            "updatedAt": "2026-09-07T00:00:00Z",
        }
    ]


def incident_rows():
    return [
        {
            "id": f"incident-{index}",
            "nodeId": "fixture-node",
            "kind": "cpuHigh",
            "target": f"cpu-{index}",
            "status": "active",
            "title": f"Fixture incident {index}",
            "thresholdPercent": 90.0,
            "observedPercent": 95.0,
            "openedAt": "2026-09-07T00:00:00Z",
            "lastObservedAt": "2026-09-07T00:00:00Z",
            "resolvedAt": None,
            "resolutionReason": None,
            "details": {},
        }
        for index in range(1, 22)
    ]


def summary_rows():
    return [
        {
            "nodeId": "fixture-node",
            "date": f"2026-08-{index:02d}",
            "sampleCount": 3,
            "coverage": 100.0,
            "coveragePercent": 100.0,
            "cpu": {"min": 10.0, "avg": 20.0, "max": 30.0},
            "memory": {"min": 11.0, "avg": 21.0, "max": 31.0},
            "diskSummary": {
                "/": {"min": 12.0, "avg": 22.0, "max": 32.0},
            },
            "offlineSeconds": 0,
            "incidentCount": index,
        }
        for index in range(1, 22)
    ]


def page_payload(rows, query):
    current = int(query.get("current", ["1"])[0])
    page_size = int(query.get("pageSize", ["20"])[0])
    start = (current - 1) * page_size
    return {
        "data": rows[start : start + page_size],
        "total": len(rows),
        "success": True,
    }


def route_payload(name, query):
    if name == "overview":
        return overview_payload()
    if name == "nodes":
        return nodes_payload()
    if name == "incidents":
        return page_payload(incident_rows(), query)
    return page_payload(summary_rows(), query)


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
                self.send_header(key, value)
        self.send_header("content-length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)
        connection.close()

    def monitoring_response(self, route, name, query):
        with lock:
            entry = state[name]
            mode = entry["mode"]
            entry["reads"] += 1
            request = {
                "route": route,
                "name": name,
                "mode": mode,
                "query": query,
                "read": entry["reads"],
            }
            requests.append(request)
            next_status = entry["fail_after_first_status"]
            if mode == "success" and next_status is not None:
                entry["mode"] = next_status
                entry["fail_after_first_status"] = None
        if mode == "slow":
            time.sleep(1)
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
                receipt = {"requests": list(requests)}
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
        if self.path != "/__monitoring_fixture/mode":
            self.proxy()
            return
        size = int(self.headers.get("content-length", "0"))
        try:
            payload = json.loads(self.rfile.read(size) or b"{}")
        except json.JSONDecodeError:
            self.send_json(400, {"error": "invalid JSON"})
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
