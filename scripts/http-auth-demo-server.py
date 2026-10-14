"""Local authenticated API. Expected token is read via hidden input, never an argument."""
import argparse
import getpass
import hmac
import json
import threading
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlsplit

parser = argparse.ArgumentParser()
parser.add_argument("--port", type=int, default=8766)
args = parser.parse_args()
token = getpass.getpass("输入测试 API 接受的 Token（隐藏输入）：") if sys.stdin.isatty() else sys.stdin.readline().rstrip("\r\n")
if not token or "\r" in token or "\n" in token:
    raise SystemExit("Token 必须非空且为单行")
expected = ("Bearer " + token).encode("utf-8")
lock = threading.Lock()
count = 0

class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        global count
        if urlsplit(self.path).path != "/protected":
            self.send_error(404)
            return
        supplied = self.headers.get("Authorization", "").encode("utf-8")
        authenticated = hmac.compare_digest(supplied, expected)
        with lock:
            if authenticated:
                count += 1
            current = count
        payload = {"data": {"message": "认证成功", "count": current}} if authenticated else {"error": "unauthorized"}
        body = json.dumps(payload, ensure_ascii=False).encode("utf-8")
        self.send_response(200 if authenticated else 401)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

server = ThreadingHTTPServer(("127.0.0.1", args.port), Handler)
print(f"Authenticated demo API: http://127.0.0.1:{server.server_port}/protected", flush=True)
server.serve_forever()
