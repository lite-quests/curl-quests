#!/bin/sh
lsof -ti:8080 | xargs kill -9 2>/dev/null || true
sleep 0.2
python3 - <<'PYEOF'
import json
import random
import string
from http.server import HTTPServer, BaseHTTPRequestHandler

def random_string(length=10):
    return ''.join(random.choices(string.ascii_letters + string.digits, k=length))

# Pre-generate the massive config
config_data = {
    "system": {
        f"module_{i}": {"status": "ok", "latency": random.randint(10, 100)} for i in range(50)
    },
    "features": {
        f"flag_{i}": random.choice([True, False]) for i in range(100)
    },
    "next_path": "hidden_logs",
    "legacy": {
        f"old_module_{i}": "deprecated" for i in range(50)
    }
}

# Pre-generate 100 logs
logs_data = []
for i in range(100):
    if i == 87:
        logs_data.append({"id": i, "event": "cron", "timestamp": "2023-10-01T12:00:00Z", "token": "ALPHA_77", "details": random_string(50)})
    else:
        logs_data.append({"id": i, "event": random.choice(["boot", "auth", "sync", "fail"]), "timestamp": "2023-10-01T12:00:00Z", "token": "NONE", "details": random_string(50)})

# Pre-generate 200 users
users_data = []
for i in range(200):
    users_data.append({
        "id": i,
        "name": f"user_{random_string(5)}",
        "role": random.choice(["viewer", "editor", "guest"]),
        "secret_password": "null",
        "last_login": "2023-01-01"
    })

# Insert admin at index 142
users_data.insert(142, {
    "id": 9999,
    "name": "admin",
    "role": "owner",
    "secret_password": "JQ_MASTER_4242",
    "last_login": "2023-10-01"
})

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args): pass

    def do_GET(self):
        if self.path == "/config":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write((json.dumps(config_data, indent=2) + "\n").encode())
            
        elif self.path == "/hidden_logs":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write((json.dumps(logs_data, indent=2) + "\n").encode())

        elif self.path == "/vault/ALPHA_77":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write((json.dumps(users_data, indent=2) + "\n").encode())

        else:
            self.send_response(404)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write((json.dumps({"error": "Not Found"}) + "\n").encode())

if __name__ == "__main__":
    server = HTTPServer(("127.0.0.1", 8080), Handler)
    server.serve_forever()
PYEOF
