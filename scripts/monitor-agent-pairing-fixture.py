#!/usr/bin/env python3
"""Linux-only installed-Agent readiness fixture.

Python is intentional: the installed arm64 container has no Bun, while the
standard library supplies both a TLS HTTP server and Unix datagram receiver.
"""

import argparse
import hashlib
import http.server
import json
import os
import pwd
import re
import select
import socket
import ssl
import subprocess
import sys
import threading
import time
from pathlib import Path


TOKEN = "fixture-token-is-not-a-placeholder"
PATH = "/api/monitor/agent-reports"


class FixtureFailure(RuntimeError):
    pass


def parse_args():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--agent", type=Path, required=True)
    parser.add_argument("--certificate", type=Path, required=True)
    parser.add_argument("--private-key", type=Path, required=True)
    parser.add_argument("--runtime-root", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--endpoint", required=True)
    parser.add_argument("--port", type=int, required=True)
    return parser.parse_args()


def event(writer, **values):
    writer.write(json.dumps(values, sort_keys=True) + "\n")
    writer.flush()


def wait_for(receiver, timeout):
    readable, _, _ = select.select([receiver], [], [], timeout)
    if not readable:
        return None
    return receiver.recv(64)


def stop_agent(process):
    if process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=3)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=3)


def redacted_tail(path):
    try:
        text = path.read_text(encoding="utf-8", errors="replace")[-4096:]
    except OSError as error:
        return f"<unavailable: {error}>"
    text = text.replace(TOKEN, "[redacted]")
    return re.sub(r"(RUSTZEN_MONITOR_AGENT_TOKEN=)[^\\s]+", r"\\1[redacted]", text)


def run_case(args, writer, scenario):
    received = []

    class Handler(http.server.BaseHTTPRequestHandler):
        def do_POST(self):
            size = int(self.headers.get("content-length", "0"))
            body = self.rfile.read(size)
            try:
                report = json.loads(body)
            except json.JSONDecodeError as error:
                raise FixtureFailure(f"{scenario}: Agent sent invalid JSON: {error}") from error
            valid = (
                self.path == PATH
                and self.headers.get("x-rustzen-monitor-agent-token") == TOKEN
                and report.get("nodeId") == "fixture-agent"
                and isinstance(report.get("sequence"), int)
            )
            received.append(valid)
            response = "drop" if scenario == "drop" else 401 if scenario == "unauthorized" else 200
            event(
                writer,
                case=scenario,
                event="request",
                valid=valid,
                path=self.path,
                sequence=report.get("sequence"),
                bodySha256=hashlib.sha256(body).hexdigest(),
                response=response,
            )
            if not valid:
                self.send_error(400)
                return
            if scenario == "drop":
                self.close_connection = True
                return
            if scenario == "unauthorized":
                self.send_response(401)
                self.send_header("content-length", "0")
                self.end_headers()
                return
            payload = json.dumps(
                {"code": 0, "message": "Success", "data": {"status": scenario}}
            ).encode()
            self.send_response(200)
            self.send_header("content-type", "application/json")
            self.send_header("content-length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

        def log_message(self, *_):
            return

    class Server(http.server.ThreadingHTTPServer):
        allow_reuse_address = True

    socket_path = args.runtime_root / f"d55c-{scenario}.sock"
    socket_path.unlink(missing_ok=True)
    receiver = socket.socket(socket.AF_UNIX, socket.SOCK_DGRAM)
    receiver.bind(str(socket_path))
    os.chmod(socket_path, 0o666)
    server = Server(("127.0.0.1", args.port), Handler)
    context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    context.load_cert_chain(args.certificate, args.private_key)
    server.socket = context.wrap_socket(server.socket, server_side=True)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    output = args.runtime_root / f"d55c-{scenario}.agent.log"
    environment = os.environ | {
        "RUSTZEN_ENV": "production",
        "RUSTZEN_RUNTIME_ROOT": str(args.runtime_root),
        "RUSTZEN_MONITOR_AGENT_TOKEN": TOKEN,
        "RUSTZEN_MONITOR_NODE_ID": "fixture-agent",
        "RUSTZEN_MONITOR_CONTROLLER_URL": args.endpoint,
        "NOTIFY_SOCKET": str(socket_path),
        "NO_PROXY": "monitor.example,127.0.0.1,localhost",
        "no_proxy": "monitor.example,127.0.0.1,localhost",
    }
    process = None
    account = pwd.getpwnam("rz-monitor-agent")
    try:
        with output.open("wb") as stream:
            process = subprocess.Popen(
                [str(args.agent)],
                env=environment,
                stdout=stream,
                stderr=subprocess.STDOUT,
                user=account.pw_uid,
                group=account.pw_gid,
                extra_groups=[],
            )
            deadline = time.monotonic() + 10
            while not received and time.monotonic() < deadline:
                time.sleep(0.05)
            if received != [True]:
                stop_agent(process)
                raise FixtureFailure(
                    f"{scenario}: expected one valid Controller request, got {received}; "
                    f"Agent log tail: {redacted_tail(output)}"
                )
            ready = wait_for(receiver, 4 if scenario in {"accepted", "duplicate"} else 1)
            event(writer, case=scenario, event="readiness", payload=ready.decode() if ready else None)
            expected_ready = scenario in {"accepted", "duplicate"}
            if expected_ready and ready != b"READY=1":
                stop_agent(process)
                raise FixtureFailure(
                    f"{scenario}: expected READY=1, got {ready!r}; Agent log tail: {redacted_tail(output)}"
                )
            if not expected_ready and ready is not None:
                stop_agent(process)
                raise FixtureFailure(
                    f"{scenario}: must not become ready, got {ready!r}; Agent log tail: {redacted_tail(output)}"
                )
    finally:
        if process is not None:
            stop_agent(process)
        server.shutdown()
        server.server_close()
        thread.join(timeout=2)
        receiver.close()
        socket_path.unlink(missing_ok=True)
        event(writer, case=scenario, event="cleanup", pid=process.pid if process else None)


def main():
    if sys.platform != "linux":
        raise FixtureFailure("D55c fixture requires Linux")
    args = parse_args()
    if os.geteuid() != 0:
        raise FixtureFailure("D55c fixture must start as root to run the Agent service identity")
    args.runtime_root.mkdir(parents=True, exist_ok=True)
    os.chown(args.runtime_root, 2345, 2345)
    os.chmod(args.runtime_root, 0o750)
    with args.evidence.open("w", encoding="utf-8") as writer:
        for scenario in ("accepted", "duplicate", "unauthorized", "drop"):
            run_case(args, writer, scenario)
    print(f"D55c installed Agent TLS/readiness fixture passed: {args.evidence}")


if __name__ == "__main__":
    try:
        main()
    except (FixtureFailure, OSError, subprocess.SubprocessError) as error:
        print(f"D55c fixture failed: {error}", file=sys.stderr)
        raise SystemExit(1)
