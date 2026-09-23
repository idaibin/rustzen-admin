#!/usr/bin/env python3
"""Route-exact, in-process fixture for the Analytics UI browser gate.

Only the two Insights reads are synthetic.  Every other Admin UI/API request is
proxied to the real Admin service, so authentication and route permissions stay
in the exercised application.
"""

from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from http.client import HTTPConnection
import json
import os
import time
from urllib.parse import parse_qs, urlparse


UPSTREAM_HOST = os.environ.get("RUSTZEN_ANALYTICS_FIXTURE_UPSTREAM", "127.0.0.1")
UPSTREAM_PORT = int(os.environ.get("RUSTZEN_ANALYTICS_FIXTURE_UPSTREAM_PORT", "19801"))
PORT = int(os.environ.get("RUSTZEN_ANALYTICS_FIXTURE_PORT", "19805"))
SLOW_SECONDS = float(os.environ.get("RUSTZEN_ANALYTICS_FIXTURE_SLOW_SECONDS", "5"))
state = {
    "overview": "success",
    "events": "success",
    "fail_after_first_status": {"overview": None, "events": None},
    "requests": [],
}


def overview():
    return {
        "code": 0,
        "message": "Success",
        "data": {
            "pv": 12,
            "uv": 7,
            "eventCount": 9,
            "requestCount": 4,
            "errorCount": 0,
            "averageDurationMs": 12.0,
            "p95DurationMs": 12,
            "trend": [{"date": "2026-09-05", "pv": 12, "uv": 7, "requestCount": 4}],
        },
    }


def events(query, empty=False):
    kind = query.get("eventKind", ["page"])[0]
    path = query.get("path", ["/analytics/overview"])[0]
    rows = [] if empty or kind == "other" or path == "/empty" else [{
        "id": "analytics-ui-state-1",
        "eventName": "page_view",
        "visitorId": "fixture-visitor",
        "userId": None,
        "sessionId": "fixture-session",
        "pagePath": path,
        "referrer": None,
        "apiPath": None,
        "apiMethod": None,
        "statusCode": None,
        "platform": "web",
        "durationMs": 12,
        "isError": False,
        "properties": {},
        "occurredAt": "2026-09-05T00:00:00Z",
        "receivedAt": "2026-09-05T00:00:00Z",
    }]
    return {"code": 0, "message": "Success", "data": {"data": rows, "total": 41 if rows else 0, "success": True}}


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *_):
        return

    def json(self, status, body):
        raw = json.dumps(body, separators=(",", ":")).encode()
        self.send_response(status)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(raw)))
        self.end_headers()
        self.wfile.write(raw)

    def fixture_response(self, route, mode, query):
        state["requests"].append({"route": route, "mode": mode, "query": query})
        if mode == "slow":
            time.sleep(SLOW_SECONDS)
        if mode == "403":
            self.json(403, {"code": 40002, "message": "fixture forbidden", "data": None})
            return True
        if mode == "500":
            self.json(500, {"code": 40002, "message": "fixture failure", "data": None})
            return True
        if route == "/api/insights/overview":
            self.json(200, overview())
        else:
            self.json(200, events(query, mode == "empty"))
        key = "overview" if route.endswith("overview") else "events"
        # Only the explicit background-refresh case changes its second read to
        # 500. Ordinary success cases stay successful so filter/pagination
        # receipts describe their own behavior rather than an injected error.
        if mode == "success" and state["fail_after_first_status"][key] is not None:
            state[key] = state["fail_after_first_status"][key]
            state["fail_after_first_status"][key] = None
        return True

    def proxy(self):
        size = int(self.headers.get("content-length", "0"))
        body = self.rfile.read(size) if size else None
        headers = {key: value for key, value in self.headers.items() if key.lower() not in {"host", "connection", "content-length"}}
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

    def handle_request(self):
        parsed = urlparse(self.path)
        if parsed.path == "/__analytics_fixture/health":
            self.json(200, {"status": "ok"})
            return
        if parsed.path == "/__analytics_fixture/receipt":
            self.json(200, {"requests": state["requests"]})
            return
        if parsed.path in {"/api/insights/overview", "/api/insights/events"}:
            key = "overview" if parsed.path.endswith("overview") else "events"
            self.fixture_response(parsed.path, state[key], parse_qs(parsed.query))
            return
        self.proxy()

    do_GET = handle_request
    do_POST = handle_request
    do_PUT = handle_request
    do_DELETE = handle_request

    def do_PATCH(self):
        if self.path != "/__analytics_fixture/mode":
            self.proxy()
            return
        size = int(self.headers.get("content-length", "0"))
        payload = json.loads(self.rfile.read(size) or b"{}")
        for route in ("overview", "events"):
            mode = payload.get(route)
            if mode is not None:
                if mode not in {"slow", "success", "empty", "403", "500"}:
                    self.json(400, {"error": "invalid fixture mode"})
                    return
                state[route] = mode
            fail_after_first_status = payload.get(f"{route}FailAfterFirstStatus")
            if fail_after_first_status is not None:
                if fail_after_first_status not in {"403", "500"}:
                    self.json(400, {"error": "invalid background refresh status"})
                    return
                state["fail_after_first_status"][route] = fail_after_first_status
        self.json(200, {"status": "ok", "modes": {"overview": state["overview"], "events": state["events"]}})


ThreadingHTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
