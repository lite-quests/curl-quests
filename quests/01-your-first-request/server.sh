#!/bin/sh
python3 - <<'PYEOF'
import json, os, sqlite3
from http.server import HTTPServer, BaseHTTPRequestHandler

DB_PATH = os.environ.get("QUEST_DB", "data/quest.db")

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass  # silence access logs

    def do_GET(self):
        if self.path == "/heroes":
            conn = sqlite3.connect(DB_PATH)
            rows = conn.execute("SELECT id, name, power FROM heroes").fetchall()
            conn.close()
            heroes = [{"id": r[0], "name": r[1], "power": r[2]} for r in rows]
            body = json.dumps(heroes, indent=2).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
        else:
            self.send_response(404)
            self.end_headers()

class ReusableHTTPServer(HTTPServer):
    allow_reuse_address = True

if __name__ == "__main__":
    server = ReusableHTTPServer(("127.0.0.1", 8080), Handler)
    server.serve_forever()
PYEOF
