#!/usr/bin/env python3
"""Verify that repeated fault matches are counted and never reach upstream."""
import concurrent.futures
import http.client
import json
import os
import signal
import socket
import subprocess
import tempfile
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path


class Upstream(BaseHTTPRequestHandler):
    mutation_count = 0
    lock = threading.Lock()

    def log_message(self, *_args):
        pass

    def do_PUT(self):
        with self.lock:
            type(self).mutation_count += 1
        self.send_response(204)
        self.end_headers()

    def do_POST(self):
        if self.path == "/api/manage/tasks/cleanup-operation-logs-retention/run":
            body = b'{"data":{"id":"manual-run"}}'
            self.send_response(200)
            self.send_header("content-type", "application/json")
            self.send_header("content-length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        self.send_response(404)
        self.end_headers()

    def do_GET(self):
        if self.path == "/api/manage/tasks":
            body = b'{"data":[{"taskKey":"cleanup-operation-logs-retention","running":false,"lastStatus":"success"}]}'
            self.send_response(200)
            self.send_header("content-type", "application/json")
            self.send_header("content-length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        self.send_response(404)
        self.end_headers()


def free_port():
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


def request(proxy_port):
    connection = http.client.HTTPConnection("127.0.0.1", proxy_port, timeout=5)
    try:
        connection.request("PUT", "/mutation", body=b"{}")
        response = connection.getresponse()
        response.read()
        return response.status
    except (http.client.HTTPException, OSError):
        return None
    finally:
        connection.close()


def healthy(proxy_port):
    connection = http.client.HTTPConnection("127.0.0.1", proxy_port, timeout=1)
    try:
        connection.request("GET", "/__verify_proxy_health")
        return connection.getresponse().status == 204
    except (http.client.HTTPException, OSError):
        return False
    finally:
        connection.close()


def task_request(proxy_port, method, path):
    connection = http.client.HTTPConnection("127.0.0.1", proxy_port, timeout=5)
    try:
        connection.request(method, path)
        response = connection.getresponse()
        return response.status, json.loads(response.read())
    finally:
        connection.close()


def main():
    root = Path(__file__).resolve().parent
    upstream_port = free_port()
    proxy_port = free_port()
    upstream = ThreadingHTTPServer(("127.0.0.1", upstream_port), Upstream)
    upstream_thread = threading.Thread(target=upstream.serve_forever, daemon=True)
    upstream_thread.start()
    with tempfile.TemporaryDirectory(prefix="rz-fault-proxy-test-") as temporary:
        receipt = Path(temporary) / "receipt.json"
        environment = os.environ.copy()
        environment.update(
            {
                "RUSTZEN_VERIFY_PROXY_UPSTREAM_PORT": str(upstream_port),
                "RUSTZEN_VERIFY_PROXY_LISTEN_PORT": str(proxy_port),
                "RUSTZEN_VERIFY_FAULT_METHOD": "PUT",
                "RUSTZEN_VERIFY_FAULT_MODE": "network",
                "RUSTZEN_VERIFY_FAULT_ROUTE": "/mutation",
                "RUSTZEN_VERIFY_FAULT_RECEIPT": str(receipt),
            }
        )
        proxy = subprocess.Popen(
            ["python3", str(root / "admin-browser-fault-proxy.py")], env=environment
        )
        try:
            for _ in range(100):
                if healthy(proxy_port):
                    break
                time.sleep(0.02)
            else:
                raise AssertionError("fault proxy did not become ready")
            with concurrent.futures.ThreadPoolExecutor(max_workers=2) as executor:
                results = list(executor.map(lambda _index: request(proxy_port), range(2)))
            if results != [None, None]:
                raise AssertionError(f"network faults unexpectedly returned statuses: {results}")
        finally:
            proxy.send_signal(signal.SIGTERM)
            proxy.wait(timeout=5)
        data = json.loads(receipt.read_text(encoding="utf-8"))
        if data["hitCount"] != 2:
            raise AssertionError(f"expected two exact fault hits, received {data}")
        if Upstream.mutation_count != 0:
            raise AssertionError("a matching mutation reached the upstream server")
        receipt_count = Path(temporary) / "count-receipt.json"
        environment["RUSTZEN_VERIFY_FAULT_MODE"] = "count"
        environment["RUSTZEN_VERIFY_FAULT_RECEIPT"] = str(receipt_count)
        proxy = subprocess.Popen(
            ["python3", str(root / "admin-browser-fault-proxy.py")], env=environment
        )
        try:
            for _ in range(100):
                if healthy(proxy_port):
                    break
                time.sleep(0.02)
            else:
                raise AssertionError("count proxy did not become ready")
            if request(proxy_port) != 204:
                raise AssertionError("count mode did not forward the matching mutation")
        finally:
            proxy.send_signal(signal.SIGTERM)
            proxy.wait(timeout=5)
        count_data = json.loads(receipt_count.read_text(encoding="utf-8"))
        if count_data["hitCount"] != 1 or Upstream.mutation_count != 1:
            raise AssertionError(f"count mode did not record one forwarded mutation: {count_data}")
        for index in range(3):
            transition_receipt = Path(temporary) / f"transition-{index}.json"
            environment.update(
                {
                    "RUSTZEN_VERIFY_FAULT_METHOD": "GET",
                    "RUSTZEN_VERIFY_FAULT_MODE": "task-transition",
                    "RUSTZEN_VERIFY_FAULT_ROUTE": "/api/manage/tasks",
                    "RUSTZEN_VERIFY_FAULT_RECEIPT": str(transition_receipt),
                }
            )
            proxy = subprocess.Popen(["python3", str(root / "admin-browser-fault-proxy.py")], env=environment)
            try:
                for _ in range(100):
                    if healthy(proxy_port):
                        break
                    time.sleep(0.02)
                else:
                    raise AssertionError("task transition proxy did not become ready")
                status, created = task_request(proxy_port, "POST", "/api/manage/tasks/cleanup-operation-logs-retention/run")
                if status != 200 or created.get("data", {}).get("id") != "manual-run":
                    raise AssertionError(f"transition did not forward the real run: {created}")
                status, running = task_request(proxy_port, "GET", "/api/manage/tasks")
                if status != 200 or running["data"][0]["running"] is not True:
                    raise AssertionError(f"transition did not expose running state: {running}")
                status, succeeded = task_request(proxy_port, "GET", "/api/manage/tasks")
                if status != 200 or succeeded["data"][0]["running"] is not False:
                    raise AssertionError(f"transition did not restore upstream state: {succeeded}")
            finally:
                proxy.send_signal(signal.SIGTERM)
                proxy.wait(timeout=5)
            transition = json.loads(transition_receipt.read_text(encoding="utf-8"))
            if transition["transitionRunId"] != "manual-run" or transition["transitionReads"] != 2 or transition["hitCount"] != 3:
                raise AssertionError(f"transition receipt was not replay-safe: {transition}")
        upstream.shutdown()
        upstream.server_close()
    print("Admin browser fault proxy replay guard passed")


if __name__ == "__main__":
    main()
