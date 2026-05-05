# bridge_serve_python.py
import http.server
import socketserver
import socket
import os
import glob
import sys
import threading
import json
from urllib.parse import urlparse, parse_qs
from datetime import datetime

# Configuration from CLI args
PORT = 8080
BIND = "0.0.0.0"
PROJECTS_FILE = ""
PROJECT_NAME = ""

def get_local_ip():
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.connect(("8.8.8.8", 80))
        ip = s.getsockname()[0]
        s.close()
        return ip
    except Exception:
        return socket.gethostbyname(socket.gethostname())

def load_project_config(projects_file, project_name):
    try:
        with open(projects_file, 'r', encoding='utf-8') as f:
            projects = json.load(f)
            for p in projects:
                if p.get('name') == project_name:
                    return p
    except Exception as e:
        print(f"Error loading projects: {e}")
    return None

class BridgeHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'X-Requested-With, Content-Type')
        super().end_headers()

    def do_OPTIONS(self):
        self.send_response(204)
        self.end_headers()

    def do_GET(self):
        # Serve the index page or files
        if self.path == "/":
            self.send_response(200)
            self.send_header("Content-type", "text/html; charset=utf-8")
            self.end_headers()
            
            project = load_project_config(PROJECTS_FILE, PROJECT_NAME)
            if not project:
                self.wfile.write(f"Project {PROJECT_NAME} not found".encode())
                return

            main_path = project.get('main_path', '')
            # Find APKs in app/build/outputs/apk/ (typical Android path)
            search_path = os.path.join(main_path, "app", "build", "outputs", "apk", "**", "*.apk")
            apk_files = glob.glob(search_path, recursive=True)
            apk_files.sort(key=os.path.getmtime, reverse=True)

            html_parts = [
                "<html><head><meta name='viewport' content='width=device-width, initial-scale=1.0'>",
                "<style>body{font-family:sans-serif;padding:20px;background:#f0f0f0;}",
                ".card{background:white;padding:15px;border-radius:8px;box-shadow:0 2px 4px rgba(0,0,0,0.1);margin-bottom:10px;}",
                "a{display:block;padding:15px;background:#007bff;color:white;text-decoration:none;border-radius:5px;text-align:center;}",
                "</style></head><body>",
                f"<h1>{PROJECT_NAME} Builds</h1>"
            ]

            if not apk_files:
                html_parts.append("<p>No APK files found.</p>")
            else:
                for apk in apk_files:
                    name = os.path.basename(apk)
                    mtime = datetime.fromtimestamp(os.path.getmtime(apk)).strftime('%Y-%m-%d %H:%M')
                    # We'll use a virtual path /files/filename
                    html_parts.append(f"<div class='card'><b>{name}</b><br><small>{mtime}</small><br><br>")
                    html_parts.append(f"<a href='/files/{name}'>Download APK</a></div>")
            
            html_parts.append("</body></html>")
            self.wfile.write("\n".join(html_parts).encode())
        
        elif self.path.startswith("/files/"):
            filename = self.path.replace("/files/", "")
            project = load_project_config(PROJECTS_FILE, PROJECT_NAME)
            if project:
                main_path = project.get('main_path', '')
                search_path = os.path.join(main_path, "app", "build", "outputs", "apk", "**", filename)
                found_files = glob.glob(search_path, recursive=True)
                if found_files:
                    file_path = found_files[0]
                    self.send_response(200)
                    self.send_header("Content-type", "application/vnd.android.package-archive")
                    self.send_header("Content-Length", os.path.getsize(file_path))
                    self.end_headers()
                    with open(file_path, 'rb') as f:
                        self.wfile.write(f.read())
                    return
            self.send_error(404, "File not found")
        else:
            super().do_GET()

def main():
    global PROJECTS_FILE, PROJECT_NAME, PORT, BIND
    
    args = sys.argv
    for i in range(len(args)):
        if args[i] == "--projects" and i+1 < len(args):
            PROJECTS_FILE = args[i+1]
        elif args[i] == "--project" and i+1 < len(args):
            PROJECT_NAME = args[i+1]
        elif args[i] == "--port" and i+1 < len(args):
            PORT = int(args[i+1])
        elif args[i] == "--bind" and i+1 < len(args):
            BIND = args[i+1]

    if not PROJECTS_FILE or not PROJECT_NAME:
        print("Missing required arguments")
        sys.exit(1)

    ip = get_local_ip()
    print(f"Bridge status: connected")
    print(f"Listening on http://{BIND}:{PORT}")
    print(f"LAN IP: http://{ip}:{PORT}")

    socketserver.TCPServer.allow_reuse_address = True
    with socketserver.TCPServer((BIND, PORT), BridgeHandler) as httpd:
        httpd.serve_forever()

if __name__ == "__main__":
    main()
