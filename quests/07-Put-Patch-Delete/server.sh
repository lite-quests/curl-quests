#!/bin/sh
lsof -ti:8080 | xargs kill -9 2>/dev/null || true
sleep 0.2
python3 - <<'PYEOF'
import json, os, sqlite3
from http.server import HTTPServer, BaseHTTPRequestHandler

DB_PATH = os.environ.get("QUEST_DB", "data/quest.db")

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args): pass

    def get_id(self):
        # Handle trailing slashes and extract ID
        parts = [p for p in self.path.split("/") if p]
        return parts[-1] if len(parts) > 1 else None

    def do_GET(self):
        conn = sqlite3.connect(DB_PATH)
        if self.path == "/inventory" or self.path == "/inventory/":
            rows = conn.execute("SELECT id, name, price FROM inventory").fetchall()
            items = [{"id": r[0], "name": r[1], "price": r[2]} for r in rows]
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(items, indent=2).encode())
        else:
            self.send_response(404); self.end_headers()
        conn.close()

    def do_PUT(self):
        item_id = self.get_id()
        if not item_id:
            self.send_response(400); self.end_headers(); return
        
        try:
            content_length = int(self.headers.get('Content-Length', 0))
            data = json.loads(self.rfile.read(content_length))
        except:
            self.send_response(400); self.end_headers(); return
        
        conn = sqlite3.connect(DB_PATH)
        # Verify item exists
        exists = conn.execute("SELECT 1 FROM inventory WHERE id = ?", (item_id,)).fetchone()
        if not exists:
            self.send_response(404); self.end_headers(); conn.close(); return

        conn.execute("UPDATE inventory SET name = ?, price = ? WHERE id = ?", (data.get("name"), data.get("price"), item_id))
        conn.commit(); conn.close()
        
        self.send_response(200); self.end_headers()
        self.wfile.write(b"Item replaced.")

    def do_PATCH(self):
        item_id = self.get_id()
        if not item_id:
            self.send_response(400); self.end_headers(); return
        
        try:
            content_length = int(self.headers.get('Content-Length', 0))
            data = json.loads(self.rfile.read(content_length))
        except:
            self.send_response(400); self.end_headers(); return
        
        conn = sqlite3.connect(DB_PATH)
        # Verify item exists
        exists = conn.execute("SELECT 1 FROM inventory WHERE id = ?", (item_id,)).fetchone()
        if not exists:
            self.send_response(404); self.end_headers(); conn.close(); return

        updates = []
        params = []
        if "name" in data:
            updates.append("name = ?")
            params.append(data["name"])
        if "price" in data:
            updates.append("price = ?")
            params.append(data["price"])
        
        if updates:
            params.append(item_id)
            conn.execute(f"UPDATE inventory SET {', '.join(updates)} WHERE id = ?", params)
            conn.commit()
        
        conn.close()
        self.send_response(200); self.end_headers()
        self.wfile.write(b"Item updated.")

    def do_DELETE(self):
        item_id = self.get_id()
        if not item_id:
            self.send_response(400); self.end_headers(); return

        conn = sqlite3.connect(DB_PATH)
        conn.execute("DELETE FROM inventory WHERE id = ?", (item_id,))
        conn.commit(); conn.close()
        
        self.send_response(204); self.end_headers()

if __name__ == "__main__":
    server = HTTPServer(("127.0.0.1", 8080), Handler)
    server.serve_forever()
PYEOF

