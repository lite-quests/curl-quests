#!/bin/sh
# Kill any process still holding port 8080 from a previous session.
lsof -ti:8080 | xargs kill -9 2>/dev/null || true
sleep 0.2
python3 - <<'PYEOF'
import json, os, sqlite3, hmac, hashlib, base64, time
from http.server import HTTPServer, BaseHTTPRequestHandler

DB_PATH = os.environ.get("QUEST_DB", "data/quest.db")
SECRET = "curl-quest-jwt-secret-2024"

# --- Pure stdlib JWT (HS256) ---

def b64url_encode(data):
    if isinstance(data, str):
        data = data.encode()
    return base64.urlsafe_b64encode(data).rstrip(b'=').decode()

def b64url_decode(s):
    padding = 4 - len(s) % 4
    if padding != 4:
        s += '=' * padding
    return base64.urlsafe_b64decode(s)

def jwt_create(username):
    header  = b64url_encode(json.dumps({"alg": "HS256", "typ": "JWT"}))
    payload = b64url_encode(json.dumps({
        "sub": username,
        "iat": int(time.time()),
        "exp": int(time.time()) + 3600,
    }))
    signing_input = f"{header}.{payload}"
    sig = hmac.new(SECRET.encode(), signing_input.encode(), hashlib.sha256).digest()
    return f"{signing_input}.{b64url_encode(sig)}"

def jwt_verify(token):
    """Returns the payload dict on success, or None on failure."""
    try:
        parts = token.split('.')
        if len(parts) != 3:
            return None
        signing_input = f"{parts[0]}.{parts[1]}"
        expected_sig = b64url_encode(
            hmac.new(SECRET.encode(), signing_input.encode(), hashlib.sha256).digest()
        )
        if not hmac.compare_digest(expected_sig, parts[2]):
            return None
        payload = json.loads(b64url_decode(parts[1]))
        if payload.get("exp", 0) < time.time():
            return None
        return payload
    except Exception:
        return None

def hash_password(pw):
    return hashlib.sha256(pw.encode()).hexdigest()

# --- HTTP Handler ---

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def send_json(self, status, data):
        body = json.dumps(data, indent=2).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def read_json_body(self):
        length = int(self.headers.get("Content-Length", 0))
        if length == 0:
            return None
        raw = self.rfile.read(length)
        try:
            return json.loads(raw)
        except Exception:
            return None

    def get_auth_user(self):
        """Extract and verify the Bearer token; return username or None."""
        auth = self.headers.get("Authorization", "")
        if not auth.startswith("Bearer "):
            return None
        token = auth[len("Bearer "):]
        payload = jwt_verify(token)
        if payload is None:
            return None
        return payload.get("sub")

    # POST /signup
    def handle_signup(self):
        data = self.read_json_body()
        if not data:
            self.send_json(400, {"error": "Expected JSON body"})
            return

        username = data.get("username", "").strip()
        password = data.get("password", "")

        if username != "curl-quest":
            self.send_json(400, {"error": "username must be 'curl-quest'"})
            return

        if not (6 <= len(password) <= 12):
            self.send_json(400, {"error": "password must be between 6 and 12 characters"})
            return

        conn = sqlite3.connect(DB_PATH)
        existing = conn.execute(
            "SELECT id FROM users WHERE username = ?", (username,)
        ).fetchone()

        if existing:
            conn.close()
            self.send_json(409, {"error": "User already exists"})
            return

        conn.execute(
            "INSERT INTO users (username, password_hash) VALUES (?, ?)",
            (username, hash_password(password))
        )
        conn.commit()
        conn.close()
        self.send_json(201, {"message": "Account created", "username": username})

    # POST /login
    def handle_login(self):
        data = self.read_json_body()
        if not data:
            self.send_json(400, {"error": "Expected JSON body"})
            return

        username = data.get("username", "").strip()
        password = data.get("password", "")

        conn = sqlite3.connect(DB_PATH)
        row = conn.execute(
            "SELECT password_hash FROM users WHERE username = ?", (username,)
        ).fetchone()
        conn.close()

        if not row or row[0] != hash_password(password):
            self.send_json(401, {"error": "Invalid credentials"})
            return

        token = jwt_create(username)
        self.send_json(200, {
            "token": token,
            "note": (
                "This is a JWT (JSON Web Token). It is signed with HS256  the server can "
                "verify it is genuine without touching the database. It encodes who you are "
                "(sub), when it was issued (iat), and when it expires (exp). "
                "Pass it on every protected request with: "
                "-H \"Authorization: Bearer <token>\""
            ),
        })

    # GET /profile  (JWT protected)
    def handle_profile(self):
        username = self.get_auth_user()
        if not username:
            self.send_json(401, {"error": "Unauthorized pass your token with: -H \"Authorization: Bearer <token>\""})
            return

        conn = sqlite3.connect(DB_PATH)
        user_row = conn.execute(
            "SELECT id FROM users WHERE username = ?", (username,)
        ).fetchone()
        if not user_row:
            conn.close()
            self.send_json(404, {"error": "User not found"})
            return

        friends = [
            r[0] for r in conn.execute(
                "SELECT friend_name FROM friends WHERE user_id = ? ORDER BY id", (user_row[0],)
            ).fetchall()
        ]
        conn.close()
        self.send_json(200, {"username": username, "friends": friends})

    # PUT /friends  (JWT protected)
    def handle_put_friends(self):
        username = self.get_auth_user()
        if not username:
            self.send_json(401, {"error": "Unauthorized pass your token with: -H \"Authorization: Bearer <token>\""})
            return

        data = self.read_json_body()
        if not data or "friends" not in data:
            self.send_json(400, {"error": "Expected JSON body with a 'friends' array, e.g. {\"friends\":[\"Alice\",\"Bob\"]}"})
            return

        friends_list = data["friends"]
        if not isinstance(friends_list, list) or len(friends_list) < 2:
            self.send_json(400, {"error": "Provide at least 2 friends in the 'friends' array"})
            return

        conn = sqlite3.connect(DB_PATH)
        user_row = conn.execute(
            "SELECT id FROM users WHERE username = ?", (username,)
        ).fetchone()
        if not user_row:
            conn.close()
            self.send_json(404, {"error": "User not found"})
            return

        user_id = user_row[0]
        conn.execute("DELETE FROM friends WHERE user_id = ?", (user_id,))
        for name in friends_list:
            conn.execute(
                "INSERT INTO friends (user_id, friend_name) VALUES (?, ?)",
                (user_id, str(name).strip())
            )
        conn.commit()
        conn.close()
        self.send_json(200, {"message": "Friends updated", "friends": friends_list})

    def do_POST(self):
        if self.path == "/signup":
            self.handle_signup()
        elif self.path == "/login":
            self.handle_login()
        else:
            self.send_response(404)
            self.end_headers()

    def do_GET(self):
        if self.path == "/profile":
            self.handle_profile()
        else:
            self.send_response(404)
            self.end_headers()

    def do_PUT(self):
        if self.path == "/friends":
            self.handle_put_friends()
        else:
            self.send_response(404)
            self.end_headers()

class ReusableHTTPServer(HTTPServer):
    allow_reuse_address = True

if __name__ == "__main__":
    server = ReusableHTTPServer(("127.0.0.1", 8080), Handler)
    server.serve_forever()
PYEOF
