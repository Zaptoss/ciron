#!/usr/bin/env python3
"""Simple HTTP server for demonstration."""
import http.server
import socketserver
import os

PORT = int(os.getenv('PORT', 8080))

class MyHandler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header('Content-type', 'text/html')
        self.end_headers()
        html = f"""
        <!DOCTYPE html>
        <html>
        <head><title>Ciron Demo</title></head>
        <body>
            <h1>Hello from Ciron managed app!</h1>
            <p>This Python app is managed by Ciron process manager.</p>
            <p>Running on port {PORT}</p>
        </body>
        </html>
        """
        self.wfile.write(html.encode())

with socketserver.TCPServer(("", PORT), MyHandler) as httpd:
    print(f"Serving on port {PORT}")
    httpd.serve_forever()
