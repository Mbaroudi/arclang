"""An HTTP API replaying a day of the heat pump's measurements, for the twin's live tests.

    python3 http_replay_server.py <day.csv> <seconds per row> [port] [token]

The replay starts at the first request. GET /house/042/latest answers the rows
replayed so far, the latest five hours of them (a list of JSON objects, each
with its time): the twin keeps only what is new.
With a token, a request without `Authorization: Bearer <token>` is refused (401).
"""
import csv
import json
import sys
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

path, pace = sys.argv[1], float(sys.argv[2])
port = int(sys.argv[3]) if len(sys.argv) > 3 else 8765
token = sys.argv[4] if len(sys.argv) > 4 else None
rows = [r for r in csv.reader(line for line in open(path) if not line.startswith('#'))][1:]
started = None


class Replay(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path != '/house/042/latest':
            return self.answer(404, {'error': 'not found'})
        if token and self.headers.get('Authorization') != f'Bearer {token}':
            return self.answer(401, {'error': 'a bearer token is required'})
        global started
        started = started or time.time()
        replayed = rows[: min(len(rows), int((time.time() - started) / pace) + 1)]
        latest = [{'time': stamp, 'outdoor_c': float(o), 'indoor_c': float(i)} for stamp, o, i in replayed[-60:]]
        self.answer(200, latest)

    def answer(self, code, body):
        data = json.dumps(body).encode()
        self.send_response(code)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def log_message(self, *args):
        pass


ThreadingHTTPServer(('127.0.0.1', port), Replay).serve_forever()
