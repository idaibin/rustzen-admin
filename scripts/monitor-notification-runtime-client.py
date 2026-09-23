#!/usr/bin/env python3
# The pinned Linux verifier includes Python but intentionally has no Bun or
# OpenSSL. Python's standard library handles this independent HMAC/HTTP check
# and the companion shell's SQLite checks without a network/package install.
import argparse
import hashlib
import hmac
import json
import time
import urllib.error
import urllib.request
import uuid


DOMAIN = "rz-notification-producer-v1"
PATH = "/internal/v1/notification-events"
CONTENT_TYPE = "application/json"


def canonical(key_id, producer, created, expires, nonce, body):
    fields = [
        DOMAIN,
        "1",
        key_id,
        producer,
        "admin",
        "POST",
        PATH,
        CONTENT_TYPE,
        hashlib.sha256(body).hexdigest(),
        str(created),
        str(expires),
        nonce,
    ]
    return b"".join(f"{len(field)}:{field}".encode() for field in fields)


def send(args):
    body = open(args.body, "rb").read()
    created = int(time.time())
    expires = created + 60
    nonce = str(uuid.uuid4())
    signature = hmac.new(
        args.secret.encode(),
        canonical(args.key_id, "monitor", created, expires, nonce, body),
        hashlib.sha256,
    ).hexdigest()
    if args.bad_signature:
        signature = "0" * 64
    headers = {
        "content-type": CONTENT_TYPE,
        "x-rustzen-notify-version": "1",
        "x-rustzen-notify-key-id": args.key_id,
        "x-rustzen-notify-producer": "monitor",
        "x-rustzen-notify-created": str(created),
        "x-rustzen-notify-expires": str(expires),
        "x-rustzen-notify-nonce": nonce,
        "x-rustzen-notify-signature": signature,
    }
    request = urllib.request.Request(args.url, body, headers=headers, method="POST")
    try:
        response = urllib.request.urlopen(request, timeout=5)
    except urllib.error.HTTPError as error:
        response = error
    payload = response.read().decode()
    print(
        json.dumps(
            {
                "status": response.status,
                "contentType": response.headers.get_content_type(),
                "body": json.loads(payload) if payload else None,
            },
            separators=(",", ":"),
        )
    )


def self_test():
    value = hmac.new(
        b"0123456789abcdef0123456789abcdef",
        canonical("key-1", "monitor", 1_700_000_000, 1_700_000_060, "abc", b'{"x":1}'),
        hashlib.sha256,
    ).hexdigest()
    if value != "4af5d59a23f3d472ed96b7fb4c3839f7e8cf5b06ff83b82c036ea5ebde13f850":
        raise SystemExit("notification signing golden mismatch")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--url")
    parser.add_argument("--body")
    parser.add_argument("--key-id", default="runtime-v1")
    parser.add_argument("--secret", default="runtime-notification-secret-0123456789")
    parser.add_argument("--bad-signature", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return
    if not args.url or not args.body:
        parser.error("--url and --body are required")
    send(args)


if __name__ == "__main__":
    main()
