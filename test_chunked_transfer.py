import socket

# Connect to your server port
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.connect(("127.0.0.1", 8080))

# Send request headers specifying chunked transfer encoding
# (Ensure path matches a route that triggers a POST or CGI script that reads the body)
request_header = (
    "POST /cgi-bin/test.py HTTP/1.1\r\n"
    "Host: localhost\r\n"
    "Transfer-Encoding: chunked\r\n"
    "\r\n"
)
s.sendall(request_header.encode())

# Send chunk 1: "Hello" (Length 5 in hex is '5')
s.sendall(b"5\r\nHello\r\n")

# Send chunk 2: " World!" (Length 7 in hex is '7')
s.sendall(b"7\r\n World!\r\n")

# Send the final zero-length chunk terminator
s.sendall(b"0\r\n\r\n")

# Read and print the server's response
response = s.recv(4096)
print("Server Response:\n", response.decode(errors="ignore"))
s.close()