#!/bin/sh
# Kill any process still holding port 8080 from a previous session.
lsof -ti:8080 | xargs kill -9 2>/dev/null || true
sleep 0.5
python3 - <<'PYEOF'
import json, os, sqlite3, cgi, io
from http.server import HTTPServer, BaseHTTPRequestHandler

DB_PATH = os.environ.get("QUEST_DB", "data/quest.db")

PAYSLIP = [
    {"month": "January",   "salary": 5000},
    {"month": "February",  "salary": 5000},
    {"month": "March",     "salary": 0},
    {"month": "April",     "salary": 5000},
    {"month": "May",       "salary": 5000},
    {"month": "June",      "salary": 5000},
    {"month": "July",      "salary": 5000},
    {"month": "August",    "salary": 5000},
    {"month": "September", "salary": 5000},
    {"month": "October",   "salary": 5000},
    {"month": "November",  "salary": 5000},
    {"month": "December",  "salary": 5000},
]

CORRECT_PAYSLIP = [
    {"month": "January",   "salary": 5000},
    {"month": "February",  "salary": 5000},
    {"month": "March",     "salary": 5200},
    {"month": "April",     "salary": 5000},
    {"month": "May",       "salary": 5000},
    {"month": "June",      "salary": 5000},
    {"month": "July",      "salary": 5000},
    {"month": "August",    "salary": 5000},
    {"month": "September", "salary": 5000},
    {"month": "October",   "salary": 5000},
    {"month": "November",  "salary": 5000},
    {"month": "December",  "salary": 5000},
]

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

    def parse_multipart_file(self):
        """Parse a multipart/form-data request and return the uploaded file bytes."""
        env = {
            "REQUEST_METHOD": "POST",
            "CONTENT_TYPE": self.headers.get("Content-Type", ""),
            "CONTENT_LENGTH": self.headers.get("Content-Length", "0"),
        }
        form = cgi.FieldStorage(fp=self.rfile, headers=self.headers, environ=env)
        if "file" not in form:
            return None
        item = form["file"]
        return item.file.read()

    def do_GET(self):
        if self.path == "/files/payslip":
            body = json.dumps(PAYSLIP, indent=2).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Disposition", 'attachment; filename="payslip.json"')
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        elif self.path == "/files/correct-payslip":
            body = json.dumps(CORRECT_PAYSLIP, indent=2).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Disposition", 'attachment; filename="correct_payslip.json"')
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        else:
            self.send_response(404)
            self.end_headers()

    def do_POST(self):
        if self.path != "/payslips":
            self.send_response(404)
            self.end_headers()
            return

        raw = self.parse_multipart_file()
        if raw is None:
            self.send_json(400, {"error": "Bad Request: expected a multipart field named 'file'"})
            return

        try:
            entries = json.loads(raw)
        except Exception:
            self.send_json(400, {"error": "Bad Request: uploaded file is not valid JSON"})
            return

        if len(entries) != 12:
            self.send_json(400, {"error": f"Expected 12 monthly entries, got {len(entries)}"})
            return

        conn = sqlite3.connect(DB_PATH)
        conn.execute("DELETE FROM payslips")
        for entry in entries:
            conn.execute(
                "INSERT INTO payslips (month, salary) VALUES (?, ?)",
                (entry["month"], entry["salary"])
            )
        conn.commit()
        conn.close()

        # Detect the mistake and tell the user
        bad = [e for e in entries if e["salary"] == 0]
        if bad:
            months = ", ".join(e["month"] for e in bad)
            self.send_json(201, {
                "status": "uploaded",
                "entries": len(entries),
                "warning": f"Salary is 0 for: {months}. This looks like a mistake  please correct and re-upload with PUT.",
            })
        else:
            self.send_json(201, {"status": "uploaded", "entries": len(entries)})

    def do_PUT(self):
        if self.path != "/payslips":
            self.send_response(404)
            self.end_headers()
            return

        raw = self.parse_multipart_file()
        if raw is None:
            self.send_json(400, {"error": "Bad Request: expected a multipart field named 'file'"})
            return

        try:
            entries = json.loads(raw)
        except Exception:
            self.send_json(400, {"error": "Bad Request: uploaded file is not valid JSON"})
            return

        if len(entries) != 12:
            self.send_json(400, {"error": f"Expected 12 monthly entries, got {len(entries)}"})
            return

        bad = [e for e in entries if e["salary"] == 0]
        if bad:
            months = ", ".join(e["month"] for e in bad)
            self.send_json(400, {"error": f"Salary is still 0 for: {months}. Upload the corrected file."})
            return

        conn = sqlite3.connect(DB_PATH)
        conn.execute("DELETE FROM payslips")
        for entry in entries:
            conn.execute(
                "INSERT INTO payslips (month, salary) VALUES (?, ?)",
                (entry["month"], entry["salary"])
            )
        conn.commit()
        conn.close()

        self.send_json(200, {"status": "corrected", "entries": len(entries)})

class ReusableHTTPServer(HTTPServer):
    allow_reuse_address = True

if __name__ == "__main__":
    server = ReusableHTTPServer(("127.0.0.1", 8080), Handler)
    server.serve_forever()
PYEOF
