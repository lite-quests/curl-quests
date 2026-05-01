#!/bin/sh
lsof -ti:8080 | xargs kill -9 2>/dev/null || true
sleep 0.2
python3 - <<'PYEOF'
import json, os
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args): pass

    def do_GET(self):
        self._handle_request()

    def do_HEAD(self):
        self._handle_request()

    def _handle_request(self):
        parsed_url = urlparse(self.path)
        path = parsed_url.path
        query = parse_qs(parsed_url.query)
        user_agent = self.headers.get("User-Agent", "")

        if path == "/employee-portal":
            self.send_response(200)
            self.send_header("X-Manager-Token", "Manager-Access-99")
            self.end_headers()
            if self.command != "HEAD":
                body = """
========================================
       EMPLOYEE PORTAL DIRECTORY
========================================
ID    Name               Role
---   ----------------   -------------
001   Alice Smith        Engineer
002   Bob Jones          Data Analyst
003   Charlie Davis      IT Support
004   Eve Adams          HR Specialist

*** SYSTEM MESSAGE: Manager tokens are hidden in the response headers. ***
"""
                self.wfile.write(body.encode('utf-8'))

        elif path == "/staff-inventory":
            token = query.get("token", [None])[0]
            if token != "Manager-Access-99":
                self.send_response(401)
                self.end_headers()
                return

            # This endpoint has a massive body to force using -I
            self.send_response(200)
            self.send_header("X-Required-Device", "The-Bosses-iPad")
            self.end_headers()
            if self.command != "HEAD":
                # 10,000 lines of inventory data
                self.wfile.write(b"STAFF_INVENTORY_REPORT_#99-B\n" * 10000)

        elif path == "/manager-vault":
            token = query.get("token", [None])[0]
            
            # Check for the token
            if token != "Manager-Access-99":
                self.send_response(401)
                self.end_headers()
                return

            # Check for the "User" signature (curl's default)
            if "curl" in user_agent.lower():
                self.send_response(403)
                self.end_headers()
                if self.command != "HEAD":
                    self.wfile.write(b"Unauthorized device detected! Access restricted to The-Bosses-iPad.")
            elif user_agent == "The-Bosses-iPad":
                self.send_response(200)
                self.send_header("X-Ultimate-Coupon", "ULTIMATE_99_OFF")
                self.end_headers()
                if self.command != "HEAD":
                    self.wfile.write(b"Access Granted. Your ultimate coupon code is in the headers.")
            else:
                self.send_response(403)
                self.end_headers()
        else:
            self.send_response(404)
            self.end_headers()

if __name__ == "__main__":
    server = HTTPServer(("127.0.0.1", 8080), Handler)
    server.serve_forever()
PYEOF
