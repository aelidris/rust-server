#!/usr/bin/env python3
import os

# Print the mandatory CGI header followed by a blank line
print("Content-Type: text/html\r\n")

print("<html>")
print("<head>")
print("<title>CGI Test Page</title>")
print("</head>")
print("<body>")
print("<h1>Hello from Python CGI! 🚀</h1>")
print(f"<p>Request Method: <b>{os.environ.get('REQUEST_METHOD', 'UNKNOWN')}</b></p>")
print(f"<p>Query String: <b>{os.environ.get('QUERY_STRING', 'None')}</b></p>")
print("</body>")
print("</html>")