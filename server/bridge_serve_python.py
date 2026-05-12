# bridge_serve_python.py
import http.server
import socketserver
import socket
import os
import glob
import sys
import threading
import json
import shutil
from urllib.parse import urlparse, parse_qs
from datetime import datetime

# Server Version
VERSION = "1.0.5"

# ANSI Colors
GREEN = "\033[32m"
BLUE = "\033[34m"
CYAN = "\033[36m"
RED = "\033[31m"
YELLOW = "\033[33m"
RESET = "\033[0m"
BOLD = "\033[1m"
GRAY = "\033[90m"

# Configuration from CLI args
PORT = 8080
BIND = "0.0.0.0"
PROJECTS_FILE = ""
PROJECT_NAME = ""

def safe_flush():
    if sys.stdout is not None:
        try:
            sys.stdout.flush()
        except Exception:
            pass

def safe_print(msg):
    if sys.stdout is not None:
        try:
            print(msg)
            safe_flush()
        except Exception:
            pass

def print_banner(server_url):
    safe_print(f"{CYAN}{BOLD}BuildBridge Server {VERSION}{RESET}")
    safe_print(f"{GRAY}----------------------------------------{RESET}")
    safe_print(f"{GREEN}Status: {RESET}Active")
    safe_print(f"{GREEN}Project:{RESET} {PROJECT_NAME}")
    safe_print(f"{GREEN}LAN IP: {RESET}{BOLD}{server_url}{RESET}")
    safe_print(f"{GRAY}----------------------------------------{RESET}")
    safe_print(f"{YELLOW}Waiting for connections...{RESET}\n")

def resource_path(relative_path):
    try:
        base_path = sys._MEIPASS
    except Exception:
        base_path = os.path.abspath(os.path.dirname(__file__))
    return os.path.join(base_path, relative_path)

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
        safe_print(f"{RED}Error loading projects: {e}{RESET}")
    return None

def get_time_ago(timestamp):
    now = datetime.now()
    diff = now - datetime.fromtimestamp(timestamp)
    seconds = diff.total_seconds()
    if seconds < 60:
        return "Just now"
    if seconds < 3600:
        return f"{int(seconds // 60)} min ago"
    if seconds < 86400:
        return f"{int(seconds // 3600)} hours ago"
    return f"{int(seconds // 86400)} days ago"

class BridgeHandler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, format, *args):
        # Custom rich logging for PTY
        timestamp = datetime.now().strftime("%H:%M:%S")
        method = args[0]
        path = args[1]
        status = args[2]
        
        status_color = GREEN if status.startswith('2') else (RED if status.startswith('4') else YELLOW)
        
        log_line = f"{GRAY}[{timestamp}]{RESET} {BLUE}{method}{RESET} {path} {status_color}{status}{RESET}\n"
        if sys.stdout is not None:
            try:
                sys.stdout.write(log_line)
                safe_flush()
            except Exception:
                pass

    def end_headers(self):
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'X-Requested-With, Content-Type')
        super().end_headers()

    def do_OPTIONS(self):
        self.send_response(204)
        self.end_headers()

    def do_GET(self):
        if self.path == "/restart":
            self.send_response(200)
            self.send_header("Content-type", "text/plain")
            self.end_headers()
            self.wfile.write(b"Restarting...")
            safe_print(f"{YELLOW}RESTART_SIGNAL RECEIVED{RESET}")
            
            def kill_later():
                import time
                time.sleep(1)
                os._exit(0)
            
            threading.Thread(target=kill_later).start()
            return

        if self.path == "/":
            self.send_response(200)
            self.send_header("Content-type", "text/html; charset=utf-8")
            self.end_headers()
            
            project = load_project_config(PROJECTS_FILE, PROJECT_NAME)
            if not project:
                self.wfile.write(f"Project {PROJECT_NAME} not found".encode())
                return

            main_path = project.get('main_path', '')
            search_path = os.path.join(main_path, "app", "build", "outputs", "apk", "**", "*.apk")
            apk_files = glob.glob(search_path, recursive=True)
            apk_files.sort(key=os.path.getmtime, reverse=True)

            template_path = resource_path("Serve-Page.html")
            if os.path.exists(template_path):
                with open(template_path, 'r', encoding='utf-8') as f:
                    html_content = f.read()
            else:
                html_content = "<html><body><h1>Serve-Page.html not found</h1></body></html>"

            build_items_html = []
            if not apk_files:
                build_items_html.append("<p class='text-center text-gray-500 py-10'>No APK files found.</p>")
            else:
                for apk in apk_files:
                    name = os.path.basename(apk)
                    stats = os.stat(apk)
                    mtime_raw = stats.st_mtime
                    mtime = datetime.fromtimestamp(mtime_raw).strftime('%d %b %Y • %I:%M %p')
                    size_mb = f"{stats.st_size / (1024*1024):.1f} mb"
                    ago = get_time_ago(mtime_raw)

                    item_html = f"""
                    <section class="flex items-center gap-4 p-2 group" data-purpose="build-item">
                    <div class="w-16 h-16 icon-box rounded-2xl flex items-center justify-center flex-shrink-0 transition-all duration-500">
                    <svg class="w-10 h-10 text-white dark:text-black" fill="currentColor" viewbox="0 0 24 24"><path d="M17.523 15.3414C17.0673 15.3414 16.6983 14.9723 16.6983 14.5167C16.6983 14.061 17.0673 13.692 17.523 13.692C17.9786 13.692 18.3477 14.061 18.3477 14.5167C18.3477 14.9723 17.9786 15.3414 17.523 15.3414ZM6.47701 15.3414C6.02136 15.3414 5.65234 14.9723 5.65234 14.5167C5.65234 14.061 6.02136 13.692 6.47701 13.692C6.93266 13.692 7.30168 14.061 7.30168 14.5167C7.30168 14.9723 6.93266 15.3414 6.47701 15.3414ZM17.8932 10.3708L19.7891 7.08708C19.9191 6.8617 19.8419 6.57242 19.6165 6.44237C19.3912 6.31233 19.1019 6.3895 18.9718 6.61488L17.0505 9.94279C15.6171 9.28827 13.9189 8.92308 12.0002 8.92308C10.0814 8.92308 8.38327 9.28827 6.94982 9.94279L5.02852 6.61488C4.89848 6.3895 4.6092 6.31233 4.38382 6.44237C4.15844 6.57242 4.08126 6.8617 4.21131 7.08708L6.1072 10.3708C2.97395 12.0628 0.81665 15.2285 0.54044 18.9231H23.4599C23.1837 15.2285 21.0264 12.0628 17.8932 10.3708Z"></path></svg>
                    </div>
                    <div class="flex-grow">
                    <h3 class="text-lg font-bold transition-colors">{name}</h3>
                    <p class="text-brand-accent dark:text-brand-darkSecondary text-sm font-medium dark:font-normal">{mtime}</p>
                    <p class="text-brand-muted dark:text-brand-darkMuted text-xs mt-0.5">{size_mb} • {ago}</p>
                    </div>
                    <a href="/files/{name}" class="bg-brand-accent dark:bg-brand-darkAccent text-white dark:text-brand-darkBg px-6 py-2 rounded-full font-bold text-sm btn-bubble text-center" data-purpose="download-btn">Get</a>
                    </section>
                    """
                    build_items_html.append(item_html)
            
            final_html = html_content.replace("<!-- BUILD_LIST_PLACEHOLDER -->", "\n".join(build_items_html))
            final_html = final_html.replace("Build Bridge", f"{PROJECT_NAME} Builds")
            final_html = final_html.replace("Server Offline", "Server Online")
            final_html = final_html.replace("text-red-500 animate-status-glow", "text-green-500")

            self.wfile.write(final_html.encode())
        
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

class SafeTCPServer(socketserver.TCPServer):
    def handle_error(self, request, client_address):
        try:
            super().handle_error(request, client_address)
        except OSError:
            pass

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
        safe_print(f"{RED}Missing required arguments{RESET}")
        sys.exit(1)

    ip = get_local_ip()
    server_url = f"http://{ip}:{PORT}/"
    
    try:
        appdata_path = os.getenv('APPDATA')
        if appdata_path:
            status_dir = os.path.join(appdata_path, "BuildBridge", "BuildBridge", "data")
            os.makedirs(status_dir, exist_ok=True)
            status_file = os.path.join(status_dir, "bridge.json")
        else:
            status_file = os.path.join(os.path.dirname(os.path.abspath(__file__)), "bridge_status.json")

        with open(status_file, "w") as f:
            json.dump({
                "url": server_url,
                "pid": os.getpid(),
                "time": datetime.now().isoformat()
            }, f)
    except Exception as e:
        pass

    # Marker for Rust
    safe_print(f"[[SERVER_URL]]={server_url}")
    print_banner(server_url)

    socketserver.TCPServer.allow_reuse_address = True
    with SafeTCPServer((BIND, PORT), BridgeHandler) as httpd:
        httpd.serve_forever()

if __name__ == "__main__":
    main()
