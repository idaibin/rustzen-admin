#!/usr/bin/env python3
# The pinned Linux verifier already contains Python but no Bun or OpenSSL.
# Standard-library HMAC/HTTP and the one-shot TCP proxy add no runtime package.
import argparse
import hashlib
import hmac
import json
import socket
import time
import urllib.error
import urllib.request
import uuid

DOMAIN = "rz-notification-producer-v1"
PATH = "/internal/v1/notification-events"
TYPE = "application/json"


def canonical(key_id, created, expires, nonce, body):
    fields = [DOMAIN, "1", key_id, "reports", "admin", "POST", PATH, TYPE,
              hashlib.sha256(body).hexdigest(), str(created), str(expires), nonce]
    return b"".join(f"{len(value)}:{value}".encode() for value in fields)


def signed_headers(body, key_id, secret, bad=False):
    created = int(time.time())
    expires = created + 60
    nonce = str(uuid.uuid4())
    signature = hmac.new(secret.encode(), canonical(key_id, created, expires, nonce, body),
                         hashlib.sha256).hexdigest()
    if bad:
        signature = "0" * 64
    return {
        "content-type": TYPE,
        "x-rustzen-notify-version": "1",
        "x-rustzen-notify-key-id": key_id,
        "x-rustzen-notify-producer": "reports",
        "x-rustzen-notify-created": str(created),
        "x-rustzen-notify-expires": str(expires),
        "x-rustzen-notify-nonce": nonce,
        "x-rustzen-notify-signature": signature,
    }


def send(args):
    body = open(args.body, "rb").read()
    request = urllib.request.Request(
        args.url, body, headers=signed_headers(body, args.key_id, args.secret, args.bad_signature),
        method="POST")
    try:
        response = urllib.request.urlopen(request, timeout=5)
    except urllib.error.HTTPError as error:
        response = error
    payload = response.read().decode()
    print(json.dumps({"status": response.status,
                      "contentType": response.headers.get_content_type(),
                      "body": json.loads(payload) if payload else None}, separators=(",", ":")))


def read_request(stream):
    data = b""
    while b"\r\n\r\n" not in data:
        chunk = stream.recv(4096)
        if not chunk:
            return data
        data += chunk
    head, body = data.split(b"\r\n\r\n", 1)
    length = 0
    for line in head.split(b"\r\n")[1:]:
        if line.lower().startswith(b"content-length:"):
            length = int(line.split(b":", 1)[1].strip())
    while len(body) < length:
        body += stream.recv(min(4096, length - len(body)))
    return head + b"\r\n\r\n" + body


def proxy(args):
    host, port = args.listen.rsplit(":", 1)
    upstream_host, upstream_port = args.upstream.rsplit(":", 1)
    server = socket.socket()
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind((host, int(port)))
    server.listen(8)
    accepted = 0
    while True:
        client, _ = server.accept()
        request = read_request(client)
        if not request:
            client.close()
            continue
        head, request_body = request.split(b"\r\n\r\n", 1)
        head = b"\r\n".join(
            line for line in head.split(b"\r\n") if not line.lower().startswith(b"connection:")
        )
        request = head + b"\r\nConnection: close\r\n\r\n" + request_body
        with socket.create_connection((upstream_host, int(upstream_port)), timeout=5) as peer:
            peer.sendall(request)
            response = b""
            while True:
                chunk = peer.recv(4096)
                if not chunk:
                    break
                response += chunk
        accepted += 1
        _, body = request.split(b"\r\n\r\n", 1)
        if accepted == 1:
            open(args.body_out, "wb").write(body)
        else:
            client.sendall(response)
        client.close()
        open(args.state_out, "w").write(json.dumps(
            {"accepted": accepted, "firstResponseDropped": True}, separators=(",", ":")))


def self_test():
    value = hmac.new(b"0123456789abcdef0123456789abcdef",
                     canonical("key-1", 1_700_000_000, 1_700_000_060, "abc", b'{"x":1}'),
                     hashlib.sha256).hexdigest()
    if value != "9668d0b7d9dbc9f6f341554fdb1da55b51f35f145dfbd4b98ecfb88e06cf80c0":
        raise SystemExit("Reports notification signing golden mismatch")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--proxy", action="store_true")
    parser.add_argument("--url")
    parser.add_argument("--body")
    parser.add_argument("--key-id", default="reports-runtime-v1")
    parser.add_argument("--secret", default="reports-runtime-notification-secret")
    parser.add_argument("--bad-signature", action="store_true")
    parser.add_argument("--listen")
    parser.add_argument("--upstream")
    parser.add_argument("--body-out")
    parser.add_argument("--state-out")
    args = parser.parse_args()
    if args.self_test:
        self_test()
    elif args.proxy:
        proxy(args)
    elif args.url and args.body:
        send(args)
    else:
        parser.error("select --self-test, --proxy, or --url with --body")


if __name__ == "__main__":
    main()
