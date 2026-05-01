#!/bin/sh
# Kill any process still holding port 8080 from a previous session.
lsof -ti:8080 | xargs kill -9 2>/dev/null || true
sleep 0.2
python3 - <<'PYEOF'
import json, os, sqlite3
from http.server import HTTPServer, BaseHTTPRequestHandler

DB_PATH = os.environ.get("QUEST_DB", "data/quest.db")

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass  # silence access logs

    def do_GET(self):
        if self.path == "/inventory":
            conn = sqlite3.connect(DB_PATH)
            rows = conn.execute("SELECT id, name FROM inventory").fetchall()
            conn.close()
            items = [{"id": r[0], "name": r[1]} for r in rows]
            body = json.dumps(items, indent=2).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
        else:
            self.send_response(404)
            self.end_headers()

    def do_POST(self):
        if self.path == "/inventory":
            length = int(self.headers.get("Content-Length", 0))
            body = json.loads(self.rfile.read(length))
            name = body.get("name", "")
            conn = sqlite3.connect(DB_PATH)
            cur = conn.execute("INSERT INTO inventory (name) VALUES (?)", (name,))
            item_id = cur.lastrowid
            conn.commit()
            conn.close()
            result = json.dumps({"id": item_id, "name": name}, indent=2).encode()
            self.send_response(201)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(result)))
            self.end_headers()
            self.wfile.write(result)
        else:
            self.send_response(404)
            self.end_headers()

class ReusableHTTPServer(HTTPServer):
    allow_reuse_address = True

if __name__ == "__main__":
    server = ReusableHTTPServer(("127.0.0.1", 8080), Handler)
    server.serve_forever()
PYEOF
