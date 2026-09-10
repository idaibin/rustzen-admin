import base64, hashlib, json, os, shutil, socket, struct, subprocess, tempfile, time, urllib.parse, urllib.request
from pathlib import Path

MAX_CDP_FRAME_BYTES = 16 * 1024 * 1024
MAX_CDP_MESSAGE_BYTES = 16 * 1024 * 1024
MAX_HANDSHAKE_BYTES = 16 * 1024

def devtools_active_port(profile):
    lines = (profile / "DevToolsActivePort").read_text("ascii").splitlines()
    if len(lines) != 2 or not lines[0].isdigit() or not 0 < int(lines[0]) < 65536:
        raise RuntimeError("Chromium DevToolsActivePort is invalid")
    path = lines[1]
    if not path.startswith("/devtools/browser/") or any(character.isspace() for character in path) or "?" in path or "#" in path:
        raise RuntimeError("Chromium DevToolsActivePort is invalid")
    return int(lines[0]), path

def wait_devtools_active_port(profile, process, timeout=15, clock=time.monotonic, sleep=time.sleep):
    deadline = clock() + timeout; invalid = False
    while True:
        if process.poll() is not None: raise RuntimeError("Chromium exited before DevTools became available")
        try: return devtools_active_port(profile)
        except (FileNotFoundError, UnicodeDecodeError, RuntimeError) as error:
            invalid = invalid or not isinstance(error, FileNotFoundError)
            if clock() >= deadline:
                raise RuntimeError("Chromium DevToolsActivePort remained invalid" if invalid else "Chromium did not expose DevToolsActivePort")
            sleep(min(.1, deadline - clock()))

def websocket_url(value, port):
    parsed = urllib.parse.urlsplit(value)
    try: actual_port = parsed.port
    except ValueError: actual_port = None
    if parsed.scheme != "ws" or parsed.hostname != "127.0.0.1" or actual_port != port or parsed.username or parsed.password or parsed.query or parsed.fragment or not parsed.path.startswith("/devtools/") or any(character.isspace() for character in parsed.path):
        raise RuntimeError("Chromium CDP WebSocket URL is not the pinned loopback endpoint")
    return parsed

def endpoint(port, path):
    with urllib.request.urlopen(f"http://127.0.0.1:{port}{path}", timeout=1) as response:
        if response.status != 200: raise RuntimeError("Chromium CDP endpoint status differs")
        return json.load(response)

def recv_exact(sock, length):
    if not isinstance(length, int) or length < 0 or length > MAX_CDP_FRAME_BYTES:
        raise RuntimeError("Chromium CDP frame exceeds limit")
    data = b""
    while len(data) < length:
        chunk = sock.recv(length - len(data))
        if not chunk: raise RuntimeError("Chromium CDP socket closed")
        data += chunk
    return data

def websocket(url, port):
    parsed = websocket_url(url, port)
    sock = socket.create_connection((parsed.hostname, parsed.port), timeout=10)
    key = base64.b64encode(os.urandom(16)).decode()
    target = parsed.path
    try:
        sock.sendall((f"GET {target} HTTP/1.1\r\nHost: {parsed.netloc}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n").encode())
        response = b""
        while b"\r\n\r\n" not in response:
            response += recv_exact(sock, 1)
            if len(response) > MAX_HANDSHAKE_BYTES: raise RuntimeError("Chromium CDP WebSocket upgrade exceeds limit")
        lines = response[:-4].decode("ascii", "strict").split("\r\n")
        headers = {}
        for line in lines[1:]:
            if ":" not in line: raise RuntimeError("Chromium CDP WebSocket upgrade is malformed")
            name, value = line.split(":", 1); headers[name.lower()] = value.strip()
        expected = base64.b64encode(hashlib.sha1((key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").encode()).digest()).decode()
        status = lines[0].split(" ", 2)
        if (len(status) < 2 or status[0] != "HTTP/1.1" or status[1] != "101"
                or headers.get("upgrade", "").lower() != "websocket"
                or "upgrade" not in {token.strip().lower() for token in headers.get("connection", "").split(",")}
                or headers.get("sec-websocket-accept") != expected):
            raise RuntimeError("Chromium CDP WebSocket upgrade differs")
        return sock
    except Exception:
        sock.close(); raise

def send(sock, value):
    payload = json.dumps(value, separators=(",", ":")).encode(); mask = os.urandom(4)
    header = bytes([129, 128 | len(payload)]) if len(payload) < 126 else bytes([129, 254]) + struct.pack("!H", len(payload))
    sock.sendall(header + mask + bytes(byte ^ mask[index % 4] for index, byte in enumerate(payload)))

def control(sock, opcode, payload):
    mask = os.urandom(4)
    sock.sendall(bytes([128 | opcode, 128 | len(payload)]) + mask + bytes(byte ^ mask[index % 4] for index, byte in enumerate(payload)))

def frame(sock):
    first, second = recv_exact(sock, 2)
    if first & 112 or second & 128: raise RuntimeError("Chromium CDP frame has invalid flags")
    length = second & 127
    if length == 126: length = struct.unpack("!H", recv_exact(sock, 2))[0]
    elif length == 127:
        length = struct.unpack("!Q", recv_exact(sock, 8))[0]
        if length & (1 << 63): raise RuntimeError("Chromium CDP frame has invalid length")
    if length > MAX_CDP_FRAME_BYTES: raise RuntimeError("Chromium CDP frame exceeds limit")
    opcode, final = first & 15, bool(first & 128)
    if opcode >= 8 and (not final or length > 125): raise RuntimeError("Chromium CDP control frame is invalid")
    return final, opcode, recv_exact(sock, length)

def receive(sock):
    fragments = []; fragmenting = False; size = 0
    while True:
        final, opcode, payload = frame(sock)
        if opcode == 8: raise RuntimeError("Chromium CDP socket closed")
        if opcode == 9: control(sock, 10, payload); continue
        if opcode == 10: continue
        if opcode == 0:
            if not fragmenting: raise RuntimeError("Chromium CDP continuation has no message")
        elif opcode == 1:
            if fragmenting: raise RuntimeError("Chromium CDP text frame interrupts a fragmented message")
        else: raise RuntimeError("Chromium CDP frame has unsupported opcode")
        size += len(payload)
        if size > MAX_CDP_MESSAGE_BYTES: raise RuntimeError("Chromium CDP message exceeds limit")
        fragments.append(payload); fragmenting = not final
        if final: return json.loads(b"".join(fragments).decode())

class CDP:
    def __init__(self, chromium):
        self.profile = Path(tempfile.mkdtemp(prefix="rz-selected-web-", dir="/tmp")); os.chmod(self.profile, 0o700); self.sock = self.process = None
        try:
            self.process = subprocess.Popen([chromium, "--headless=new", "--no-sandbox", "--remote-allow-origins=*", "--remote-debugging-port=0", f"--user-data-dir={self.profile}", "about:blank"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            port, browser_path = wait_devtools_active_port(self.profile, self.process)
            info = endpoint(port, "/json/version")
            if websocket_url(info["webSocketDebuggerUrl"], port).path != browser_path: raise RuntimeError("Chromium browser WebSocket differs from DevToolsActivePort")
            self.sock = websocket(info["webSocketDebuggerUrl"], port); self.id = 0
            self.call("Target.createTarget", {"url":"about:blank"})
            tabs = endpoint(port, "/json/list")
            page = next(tab["webSocketDebuggerUrl"] for tab in tabs if tab.get("type") == "page")
            self.sock.close(); self.sock = websocket(page, port); self.id = 0
            self.call("Page.enable"); self.call("Runtime.enable"); self.call("Network.enable")
        except Exception:
            try: self.close()
            except Exception: pass
            raise
    def call(self, method, params=None):
        self.id += 1; call_id = self.id; send(self.sock, {"id":call_id, "method":method, "params":params or {}})
        while True:
            value = receive(self.sock)
            if value.get("id") == call_id:
                if "error" in value: raise RuntimeError(value["error"])
                return value.get("result", {})
    def evaluate(self, expression):
        result = self.call("Runtime.evaluate", {"expression":expression, "awaitPromise":True, "returnByValue":True})
        return result["result"].get("value")
    def close(self):
        try:
            if self.sock:
                try: self.sock.close()
                except OSError: pass
            if self.process and self.process.poll() is None:
                try:
                    self.process.terminate(); self.process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    try: self.process.kill(); self.process.wait(timeout=10)
                    except (OSError, subprocess.TimeoutExpired): pass
                except OSError: pass
        finally: shutil.rmtree(self.profile, ignore_errors=True)
