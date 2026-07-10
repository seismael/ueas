import sys
import subprocess
import os
import json
from http.server import BaseHTTPRequestHandler, HTTPServer

# Start ueas-mcp as a persistent subprocess
mcp_proc = subprocess.Popen(
    ['ueas-mcp'], 
    stdin=subprocess.PIPE, 
    stdout=subprocess.PIPE, 
    text=True, 
    bufsize=1
)

class MCPBridge(BaseHTTPRequestHandler):
    def do_POST(self):
        content_length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(content_length).decode('utf-8')
        
        # Send to MCP, ensuring it ends with newline
        if not body.endswith('\n'):
            body += '\n'
            
        mcp_proc.stdin.write(body)
        mcp_proc.stdin.flush()
        
        # Read exactly one line of response (since ueas-mcp writes one JSON response per line)
        response_line = mcp_proc.stdout.readline()
        
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        self.wfile.write(response_line.encode('utf-8'))
        
    def do_GET(self):
        # Respond to health checks
        self.send_response(200)
        self.send_header('Content-Type', 'text/plain')
        self.end_headers()
        self.wfile.write(b"UEAS MCP Server is running")

if __name__ == '__main__':
    port = int(os.environ.get("PORT", 7860))
    print(f"Starting MCP bridge on port {port}...")
    HTTPServer(('0.0.0.0', port), MCPBridge).serve_forever()
