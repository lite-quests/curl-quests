#!/bin/sh
lsof -ti:8080 | xargs kill -9 2>/dev/null || true
sleep 0.2
python3 - <<'PYEOF'
import json, os, sqlite3
from http.server import HTTPServer, BaseHTTPRequestHandler

DB_PATH = os.environ.get("QUEST_DB", "data/quest.db")

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args): pass

    def do_GET(self):
        # 1. 301 Moved Permanently
        if self.path == "/satellite":
            self.send_response(301)
            self.send_header("Location", "/v2/satellite")
            self.end_headers()
            return

        if self.path == "/v2/satellite":
            # 2. 401 Unauthorized
            key = self.headers.get("X-Repair-Key")
            if key != "RED_LEADER":
                self.send_response(401)
                self.send_header("Content-Type", "text/plain")
                self.end_headers()
                self.wfile.write(b"Error 401: Unauthorized. Repair Key 'X-Repair-Key' required.")
                return

            # 3. 503 Service Unavailable
            bypass = self.headers.get("X-Bypass")
            if bypass != "true":
                self.send_response(503)
                self.send_header("Content-Type", "text/plain")
                self.end_headers()
                self.wfile.write(b"Error 503: System Overloaded. Hint: Use -H 'X-Bypass: true' to stabilize.")
                return

            # 4. 200 OK (Success)
            conn = sqlite3.connect(DB_PATH)
            conn.execute("INSERT INTO systems (name, status) VALUES ('Galactic Relay', 'online')")
            conn.commit(); conn.close()

            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({
                "message": "Relay Stabilized. System Online.",
                "repair_code": "GALAXY_FIXED_2024"
            }).encode())
            return

        self.send_response(404)
        self.end_headers()

if __name__ == "__main__":
    server = HTTPServer(("127.0.0.1", 8080), Handler)
    print("Server started on port 8080")
    server.serve_forever()
PYEOF
