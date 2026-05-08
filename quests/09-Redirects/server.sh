#!/bin/sh
lsof -ti:8080 | xargs kill -9 2>/dev/null || true
sleep 0.2
python3 - <<'PYEOF'
from http.server import HTTPServer, BaseHTTPRequestHandler

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args): pass

    def do_GET(self):
        if self.path == "/old-archive":
            # Send a 301 Redirect with a body so it's visible to the user
            message = "301 Moved Permanently: The archive has been migrated to /redirect-1\n"
            self.send_response(301)
            self.send_header("Location", "http://localhost:8080/redirect-1")
            self.send_header("Content-Type", "text/plain")
            self.send_header("Content-Length", str(len(message)))
            self.end_headers()
            self.wfile.write(message.encode())
        elif self.path.startswith("/redirect-"):
            try:
                step = int(self.path.split("-")[1])
                if step < 10:
                    next_path = f"/redirect-{step + 1}"
                    message = f"301 Moved Permanently: Moving to {next_path}\n"
                    self.send_response(301)
                    self.send_header("Location", f"http://localhost:8080{next_path}")
                else:
                    message = "301 Moved Permanently: Final jump to /new-secure-vault\n"
                    self.send_response(301)
                    self.send_header("Location", "http://localhost:8080/new-secure-vault")
                
                self.send_header("Content-Type", "text/plain")
                self.send_header("Content-Length", str(len(message)))
                self.end_headers()
                self.wfile.write(message.encode())
            except ValueError:
                self.send_response(404)
                self.end_headers()
                self.wfile.write(b"404 Not Found")
        elif self.path == "/new-secure-vault":
            # The final destination
            response = "Welcome to the new vault. The Archive Access Code is: FOLLOW_THE_LIGHT_99\n"
            self.send_response(200)
            self.send_header("Content-Type", "text/plain")
            self.send_header("Content-Length", str(len(response)))
            self.end_headers()
            self.wfile.write(response.encode())
        else:
            self.send_response(404)
            self.end_headers()
            self.wfile.write(b"404 Not Found")

if __name__ == "__main__":
    server = HTTPServer(("127.0.0.1", 8080), Handler)
    server.serve_forever()
PYEOF

