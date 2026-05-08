#!/bin/sh
# Kill any process still holding port 8080 from a previous session.
lsof -ti:8080 | xargs kill -9 2>/dev/null || true
sleep 0.2
python3 - <<'PYEOF'
import json, os, sqlite3
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs

DB_PATH = os.environ.get("QUEST_DB", "data/quest.db")

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

    def do_GET(self):
        parsed = urlparse(self.path)

        if parsed.path != "/pokemon/search":
            self.send_response(404)
            self.end_headers()
            return

        # parse_qs handles %20 / + decoding automatically
        params = parse_qs(parsed.query, keep_blank_values=False)

        # Require at least one filter  prevent bare /pokemon/search from leaking all data
        if not params:
            self.send_json(400, {"error": "Bad Request: at least one query parameter is required (type, region, role, sort)"})
            return

        # Standard approach: ?type=fire&type=grass (URL must be quoted in shell)
        types = params.get("type", [])

        regions = params.get("region", [])    # single value
        roles   = params.get("role", [])      # single value, may be URL-encoded
        sorts   = params.get("sort", [])      # single value

        sql = "SELECT id, name, type, region, role, base_stat FROM pokemon WHERE 1=1"
        args = []

        if types:
            placeholders = ",".join(["?"] * len(types))
            sql += f" AND type IN ({placeholders})"
            args.extend(types)

        if regions:
            sql += " AND region = ?"
            args.append(regions[0])

        if roles:
            sql += " AND role = ?"
            args.append(roles[0])

        if sorts:
            if sorts[0] == "base_stat_desc":
                sql += " ORDER BY base_stat DESC"
            elif sorts[0] == "base_stat_asc":
                sql += " ORDER BY base_stat ASC"

        conn = sqlite3.connect(DB_PATH)
        rows = conn.execute(sql, args).fetchall()
        conn.close()

        result = [
            {"id": r[0], "name": r[1], "type": r[2], "region": r[3], "role": r[4], "base_stat": r[5]}
            for r in rows
        ]
        self.send_json(200, result)

class ReusableHTTPServer(HTTPServer):
    allow_reuse_address = True

if __name__ == "__main__":
    server = ReusableHTTPServer(("127.0.0.1", 8080), Handler)
    server.serve_forever()
PYEOF
