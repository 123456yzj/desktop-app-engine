"""Local demo endpoint: python scripts/http-demo-server.py --port 8765."""
import argparse
import json
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlsplit

parser = argparse.ArgumentParser()
parser.add_argument("--port", type=int, default=8765)
args = parser.parse_args()
lock = threading.Lock()
sequence = 0

class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        global sequence
        if urlsplit(self.path).path != "/metrics":
            self.send_error(404)
            return
        with lock:
            sequence += 1
            tick = sequence
        values = [25 + tick * 7 % 60, 30 + tick * 11 % 65, 20 + tick * 5 % 55]
        body = json.dumps({"data": {
            "stats": {"cpu": sum(values) / 300, "requests": 12000 + tick * 340},
            "option": {
                "title": {"text": "接口采集 CPU 使用率"},
                "xAxis": {"type": "category", "data": ["节点 A", "节点 B", "节点 C"]},
                "yAxis": {"type": "value", "min": 0, "max": 100},
                "series": [{"name": "CPU", "type": "bar", "data": values}]
            }
        }}, ensure_ascii=False).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

print(f"Demo API: http://127.0.0.1:{args.port}/metrics", flush=True)
ThreadingHTTPServer(("127.0.0.1", args.port), Handler).serve_forever()
