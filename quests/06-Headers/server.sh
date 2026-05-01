#!/bin/sh
# Kill any process still holding port 8080 from a previous session.
lsof -ti:8080 | xargs kill -9 2>/dev/null || true
sleep 0.2
python3 - <<'PYEOF'
import json, os, sqlite3
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import parse_qs

DB_PATH = os.environ.get("QUEST_DB", "data/quest.db")
API_KEY = "secret123"

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass  # silence access logs

    def send_json(self, status, data):
        body = json.dumps(data, indent=2).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def check_api_key(self):
        """Returns True if valid, else sends error response and returns False."""
        api_key = self.headers.get("x-api-key", "")
        if not api_key:
            self.send_json(401, {"error": "Unauthorized: missing x-api-key header"})
            return False
        if api_key != API_KEY:
            self.send_json(403, {"error": "Forbidden: invalid x-api-key"})
            return False
        return True

    def do_GET(self):
        if not self.check_api_key():
            return
        if self.path == "/groceries":
            conn = sqlite3.connect(DB_PATH)
            rows = conn.execute("SELECT id, name FROM groceries").fetchall()
            conn.close()
            items = [{"id": r[0], "name": r[1]} for r in rows]
            self.send_json(200, items)
        else:
            self.send_response(404)
            self.end_headers()

    def do_POST(self):
        if self.path != "/groceries":
            self.send_response(404)
            self.end_headers()
            return

        if not self.check_api_key():
            return

        # Check Content-Type
        content_type = self.headers.get("Content-Type", "").split(";")[0].strip()
        if content_type not in ("application/json", "application/x-www-form-urlencoded"):
            self.send_json(415, {"error": "Unsupported Media Type: use application/json or application/x-www-form-urlencoded"})
            return

        # Parse body
        length = int(self.headers.get("Content-Length", 0))
        raw = self.rfile.read(length)
        name = None
        try:
            if content_type == "application/json":
                body = json.loads(raw)
                name = body.get("name", "").strip()
            else:
                params = parse_qs(raw.decode())
                values = params.get("name", [""])
                name = values[0].strip()
        except Exception:
            pass

        if not name:
            self.send_json(400, {"error": "Bad Request: 'name' field is required"})
            return

        # Bananas requires the Done: Success header
        done_header = self.headers.get("Done", "")
        if name == "Bananas":
            if done_header != "Success":
                self.send_json(400, {"error": "Bad Request: adding Bananas requires the header -H \"Done: Success\""})
                return

        conn = sqlite3.connect(DB_PATH)
        cur = conn.execute(
            "INSERT INTO groceries (name, content_type, done_header) VALUES (?, ?, ?)",
            (name, content_type, done_header or None)
        )
        item_id = cur.lastrowid
        conn.commit()
        conn.close()
        self.send_json(201, {"id": item_id, "name": name})

class ReusableHTTPServer(HTTPServer):
    allow_reuse_address = True

if __name__ == "__main__":
    server = ReusableHTTPServer(("127.0.0.1", 8080), Handler)
    server.serve_forever()
PYEOF
