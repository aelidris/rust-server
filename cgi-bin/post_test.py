#!/usr/bin/env python3
import sys
import os

# Read incoming POST data length
content_length = int(os.environ.get('CONTENT_LENGTH', 0))
post_data = sys.stdin.read(content_length) if content_length > 0 else "No body received"

print("Content-Type: text/html\r\n")
print("<html><body>")
print("<h1>POST CGI Test Response 🚀</h1>")
print(f"<p>Method: <b>{os.environ.get('REQUEST_METHOD')}</b></p>")
print(f"<p>Received Body: <b>{post_data}</b></p>")
print("</body></html>")