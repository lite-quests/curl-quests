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
        conn = sqlite3.connect(DB_PATH)
        
        if self.path == "/users":
            rows = conn.execute("SELECT uuid, name FROM users").fetchall()
            users = [{"uuid": r[0], "name": r[1]} for r in rows]
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(users, indent=2).encode())
            
        elif self.path.startswith("/profiles/"):
            uuid = self.path.split("/")[-1]
            user = conn.execute("SELECT name FROM users WHERE uuid = ?", (uuid,)).fetchone()
            
            if user and user[0] == "Alice":
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                # Instead of the token, give an access key for the vault
                self.wfile.write(json.dumps({
                    "user": "Alice",
                    "status": "Verified",
                    "access_key": "VAULT_KEY_9921"
                }).encode())
            elif user:
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({"message": f"This is {user[0]}'s public profile. Access denied."}).encode())
            else:
                self.send_response(404); self.end_headers()

        elif self.path == "/vault":
            auth_header = self.headers.get("X-Access-Key")
            if auth_header == "VAULT_KEY_9921":
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({
                    "vault_status": "Unlocked",
                    "secret_token": "PIPELINE_MASTER_88"
                }).encode())
            else:
                self.send_response(403)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({"error": "Invalid or missing X-Access-Key header"}).encode())
        else:
            self.send_response(404); self.end_headers()
        
        conn.close()

if __name__ == "__main__":
    server = HTTPServer(("127.0.0.1", 8080), Handler)
    server.serve_forever()
PYEOF

