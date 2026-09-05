#!/usr/bin/env python3
"""Collect the two Unix readiness datagrams used by the Linux Agent runtime gate."""

from __future__ import annotations

import argparse
import json
import os
import selectors
import socket
import sys
import time
from pathlib import Path


def listen(paths: list[str], evidence: Path) -> None:
    selector = selectors.DefaultSelector()
    sockets: list[socket.socket] = []
    try:
        for raw_path in paths:
            path = Path(raw_path)
            path.unlink(missing_ok=True)
            value = socket.socket(socket.AF_UNIX, socket.SOCK_DGRAM)
            value.bind(str(path))
            os.chmod(path, 0o666)
            selector.register(value, selectors.EVENT_READ, data=path.name)
            sockets.append(value)
        while True:
            for key, _ in selector.select(timeout=1):
                payload = key.fileobj.recv(256).decode("utf-8", errors="replace")
                with evidence.open("a", encoding="utf-8") as output:
                    output.write(json.dumps({"socket": key.data, "payload": payload, "at": time.time()}) + "\n")
    finally:
        for value in sockets:
            value.close()
        for raw_path in paths:
            Path(raw_path).unlink(missing_ok=True)


def wait_for(evidence: Path, expected: int, timeout: int) -> int:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if evidence.exists():
            events = [json.loads(line) for line in evidence.read_text(encoding="utf-8").splitlines() if line]
            if len(events) >= expected and all(event["payload"] == "READY=1" for event in events):
                return 0
        time.sleep(0.1)
    return 1


parser = argparse.ArgumentParser()
parser.add_argument("--socket", action="append", default=[])
parser.add_argument("--evidence", type=Path, required=True)
parser.add_argument("--wait", action="store_true")
parser.add_argument("--expected", type=int, default=2)
parser.add_argument("--timeout", type=int, default=90)
args = parser.parse_args()

if args.wait:
    sys.exit(wait_for(args.evidence, args.expected, args.timeout))
if len(args.socket) != 2:
    parser.error("exactly two --socket values are required")
listen(args.socket, args.evidence)
