#!/usr/bin/env python3
"""Verify the notification stream without exposing its bearer credential."""
import argparse
import http.client
import json
import os
import urllib.parse


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--label", required=True)
    parser.add_argument("--url", required=True)
    args = parser.parse_args()
    token = os.environ["RUSTZEN_SSE_PREFLIGHT_TOKEN"]
    url = urllib.parse.urlsplit(args.url)
    connection = http.client.HTTPConnection(url.hostname, url.port, timeout=5)
    path = urllib.parse.urlunsplit(("", "", url.path, url.query, ""))
    try:
        connection.request("GET", path, headers={
            "accept": "text/event-stream",
            "authorization": f"Bearer {token}",
            "cache-control": "no-cache",
        })
        response = connection.getresponse()
        content_type = response.headers.get_content_type()
        frames = []
        for _ in range(2):
            lines = []
            while True:
                line = response.readline()
                if not line:
                    raise RuntimeError("notification stream ended before its initial frames")
                if line in (b"\n", b"\r\n"):
                    break
                lines.append(line.decode("utf-8", "strict").rstrip("\r\n"))
            frames.append(next((line[7:] for line in lines if line.startswith("event: ")), "comment"))
        result = {
            "label": args.label,
            "status": response.status,
            "contentType": content_type,
            "path": url.path,
            "query": bool(url.query),
            "bearer": True,
            "frames": frames,
        }
        print(json.dumps(result, separators=(",", ":")))
        if response.status != 200 or content_type != "text/event-stream":
            raise SystemExit(1)
        if frames != ["reconcile.required", "inbox.changed"]:
            raise SystemExit(1)
    finally:
        connection.close()


if __name__ == "__main__":
    main()
