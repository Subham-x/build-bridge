# serve_apk.py

import http.server
import socketserver
import socket
import os
import glob
import subprocess
import sys
import time
import msvcrt
import threading
from urllib.parse import urlparse, parse_qs
from datetime import datetime
from PIL import Image

# 👇 CHANGE THIS TO YOUR APK BUILD PATH
FOLDER_PATH = ""

# 👇 Change port if you want shorter URL:
# 8080 works everywhere; 80 lets you skip ":8080" in URL (may require admin on some systems)
PORT = 8080

# Try to import qrcode (optional)
try:
    import qrcode
except ImportError:
    qrcode = None


def get_local_ip():
    """Get local IP address of this machine."""
    try:
        # This trick usually gives the correct local IP even without real internet
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.connect(("8.8.8.8", 80))
        ip = s.getsockname()[0]
        s.close()
        return ip
    except Exception:
        # Fallback
        return socket.gethostbyname(socket.gethostname())


class APKHandler(http.server.SimpleHTTPRequestHandler):
    def do_POST(self):
        parsed_path = urlparse(self.path)
        if parsed_path.path == "/restart":
            self.send_response(200)
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            self.wfile.write(b"Restarting")

            # Restart after sending a response so client gets a clean acknowledgement.
            threading.Thread(target=restart_server_process, daemon=True).start()
        elif parsed_path.path == "/upload":
            qs = parse_qs(parsed_path.query)
            filename = qs.get("filename", ["uploaded_file"])[0]
            
            # Ensure folder exists in the script directory
            script_dir = os.path.dirname(os.path.abspath(__file__))
            out_dir = os.path.join(script_dir, "$Feedback-files")
            os.makedirs(out_dir, exist_ok=True)
            
            content_length = int(self.headers.get('Content-Length', 0))
            if content_length > 0:
                file_data = self.rfile.read(content_length)
                
                safe_filename = os.path.basename(filename)
                if not safe_filename:
                    safe_filename = "upload.bin"
                
                out_path = os.path.join(out_dir, safe_filename)
                with open(out_path, "wb") as f:
                    f.write(file_data)
            
            self.send_response(200)
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            self.wfile.write(b"OK")
        else:
            self.send_response(404)
            self.end_headers()

    def do_GET(self):
        if self.path in ("/", "/index.html"):
            self.send_response(200)
            self.send_header("Content-type", "text/html; charset=utf-8")
            self.end_headers()

            apk_files = glob.glob("*.apk")
            apk_files.sort()

            html_parts = [
                "<!DOCTYPE html>",
                "<html>",
                "<head>",
                "<meta charset='utf-8'>",
                "<meta name='viewport' content='width=device-width, initial-scale=1.0'>",
                "<title>APK Server</title>",
                "<style>",
                "body { font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; "
                "margin: 0; padding: 16px; background: #f5f5f5; }",
                "h1 { font-size: 20px; margin-bottom: 12px; }",
                ".card { background: #ffffff; border-radius: 10px; padding: 16px; "
                "box-shadow: 0 2px 6px rgba(0,0,0,0.1); }",
                ".apk-list { list-style: none; padding: 0; margin: 0; }",
                ".apk-item { margin-bottom: 10px; }",
                ".apk-link { display: block; padding: 12px 14px; border-radius: 8px; text-decoration: none; "
                "background: #007bff; color: white; font-size: 16px; text-align: center; }",
                ".apk-link:active { transform: scale(0.98); }",
                ".filename { font-weight: 500; word-break: break-all; }",
                ".note { margin-top: 14px; font-size: 13px; color: #555; }",
                ".actions-wrap { max-width: 800px; margin: 12px auto 0; }",
                ".actions-row { display: flex; gap: 10px; }",
                ".action-btn { border: none; cursor: pointer; flex: 1; font-family: inherit; }",
                ".restart-btn { background: #dc3545; }",
                ".reload-btn { background: #198754; }",
                "</style>",
                "</head>",
                "<body>",
                "<div class='card'>",
                "<h1>Available APKs</h1>",
            ]

            if not apk_files:
                html_parts.append("<p>No .apk files found in this folder.</p>")
            else:
                html_parts.append("<ul class='apk-list'>")
                for apk in apk_files:
                    apk_time = datetime.fromtimestamp(os.path.getmtime(apk))
                    time_str = apk_time.strftime('%d-%b-%Y')
                    time_hm = apk_time.strftime('%I:%M %p')
                    html_parts.append(
                        f"<li class='apk-item'>"
                        f"<a class='apk-link' href='/{apk}'>"
                        f"<span class='filename'>{apk}</span><br>"
                        f"<span style='font-size: 11px; opacity: 0.8;'>{time_str} <b>{time_hm}</b></span>"
                        f"</a></li>"
                    )
                html_parts.append("</ul>")

            html_parts.append(
                "<div class='card' style='margin-top: 20px;'>"
                "<h2>Upload Feedback Files</h2>"
                "<input type='file' id='fileInput' multiple style='margin-bottom: 10px; width: 100%; box-sizing: border-box;'>"
                "<button id='uploadBtn' class='apk-link' style='border:none; cursor:pointer; width: 100%; font-family: inherit;' onclick='uploadFiles()'>Upload</button>"
                "<p id='uploadStatus' style='font-size: 13px; color: green; margin-top: 10px; display: none; text-align: center;'></p>"
                "</div>"
                "<script>"
                "async function uploadFiles() {"
                "  const files = document.getElementById('fileInput').files;"
                "  const btn = document.getElementById('uploadBtn');"
                "  const status = document.getElementById('uploadStatus');"
                "  if (files.length === 0) return;"
                "  btn.disabled = true; btn.innerText = 'Uploading...';"
                "  status.style.display = 'none';"
                "  for(let i=0; i<files.length; i++) {"
                "    const file = files[i];"
                "    await fetch('/upload?filename=' + encodeURIComponent(file.name), {"
                "      method: 'POST',"
                "      body: file"
                "    });"
                "  }"
                "  status.innerText = 'Upload complete!';"
                "  status.style.display = 'block';"
                "  btn.disabled = false; btn.innerText = 'Upload';"
                "  document.getElementById('fileInput').value = '';"
                "}"
                "</script>"
            )

            html_parts.append(
                "<p class='note'>If install is blocked, enable "
                "<b>Install unknown apps</b> for your browser in Android settings.</p>"
            )
            html_parts.append("</div>")

            html_parts.append(
                "<div class='actions-wrap'>"
                "<div class='actions-row'>"
                "<button class='apk-link action-btn restart-btn' onclick='restartServer()'>Restart Server</button>"
                "<button class='apk-link action-btn reload-btn' onclick='hardReloadPage()'>Hard Reload</button>"
                "</div>"
                "<p id='actionStatus' style='font-size: 13px; margin-top: 10px; text-align: center; display: none;'></p>"
                "</div>"
            )

            html_parts.append(
                "<script>"
                "async function hardReloadPage() {"
                "  if ('caches' in window) {"
                "    const keys = await caches.keys();"
                "    await Promise.all(keys.map((k) => caches.delete(k)));"
                "  }"
                "  window.location.reload();"
                "}"
                "async function restartServer() {"
                "  const status = document.getElementById('actionStatus');"
                "  status.style.display = 'block';"
                "  status.style.color = '#333';"
                "  status.innerText = 'Restarting server...';"
                "  try {"
                "    await fetch('/restart', { method: 'POST' });"
                "    status.style.color = '#198754';"
                "    status.innerText = 'Server restart requested. Reconnecting...';"
                "    setTimeout(() => { hardReloadPage(); }, 1800);"
                "  } catch (e) {"
                "    status.style.color = '#dc3545';"
                "    status.innerText = 'Could not restart server.';"
                "  }"
                "}"
                "</script>"
                "</body></html>"
            )

            html = "\n".join(html_parts)
            self.wfile.write(html.encode("utf-8"))
        else:
            super().do_GET()


def restart_server_process(delay_seconds: float = 0.7):
    """Restart this script in-place after a short delay."""
    time.sleep(delay_seconds)
    script_path = os.path.abspath(__file__)
    os.execv(sys.executable, [sys.executable, script_path, *sys.argv[1:]])


class ReusableTCPServer(socketserver.TCPServer):
    allow_reuse_address = True


def generate_qr(url: str, size: int = 400):
    """Generate and display a QR image without saving to disk.

    - `size` is the output image size in pixels (square).
    - Uses high error correction and reasonable box/border so QR remains scannable.
    - Opens the QR in the default image viewer without creating a file.
    """
    if qrcode is None:
        print("⚠️ qrcode package not installed. Run: pip install qrcode[pil]")
        return

    qr = qrcode.QRCode(
        version=None,
        error_correction=qrcode.constants.ERROR_CORRECT_H,
        box_size=10,
        border=4,
    )
    qr.add_data(url)
    qr.make(fit=True)

    img = qr.make_image(fill_color="black", back_color="white").convert("RGB")
    # Resize to exact requested size (nearest to keep blocks crisp)
    img = img.resize((size, size), Image.NEAREST)
    
    # Display without saving
    img.show()
    print(f"📸 QR displayed ({size}x{size}px)")

def main():
    if not os.path.isdir(FOLDER_PATH):
        print("❌ Folder not found. Check FOLDER_PATH in serve_apk.py:")
        print(FOLDER_PATH)
        return

    os.chdir(FOLDER_PATH)

    ip = get_local_ip()
    url = f"http://{ip}"
    if PORT != 80:
        url += f":{PORT}"

    print("=====================================")
    print("📁 Serving Folder:")
    print(FOLDER_PATH)
    print()
    print("🌐 Open on your phone:")
    print(url)
    print("=====================================")
    print()

    # Generate and display QR without saving
    try:
        generate_qr(url, size=400)
    except Exception as e:
        print(f"⚠️ Could not generate QR image: {e}")

    print("Press CTRL + C to stop the server.")
    print("Press CTRL + E to change the folder path.\n")

    handler = APKHandler
    try:
        with ReusableTCPServer(("", PORT), handler) as httpd:
            
            def check_keys():
                while True:
                    if msvcrt.kbhit():
                        key = msvcrt.getch()
                        if key == b'\x05':  # Ctrl + E
                            print("\n[Ctrl+E] detected! Requesting new path...")
                            os._exit(5)
                    time.sleep(0.1)
            
            threading.Thread(target=check_keys, daemon=True).start()
            
            httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nStopping server...")
        sys.exit(0)


if __name__ == "__main__":
    main()
