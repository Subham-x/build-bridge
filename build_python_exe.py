import subprocess
import sys
import os
import shutil

def build_exe():
    print("--- Building Bridge Serve Python EXE ---")
    
    # 1. Ensure PyInstaller is installed
    try:
        import PyInstaller
    except ImportError:
        print("PyInstaller not found. Installing...")
        subprocess.check_call([sys.executable, "-m", "pip", "install", "pyinstaller"])

    # 2. Build the EXE
    # --onefile: Create a single executable
    # --name: Name of the output file
    # --version-file: Add metadata (publisher info)
    # --add-data: Bundle the HTML template
    # --icon: Set the executable icon
    # --specpath: Put the .spec file in the server directory
    
    server_dir = os.path.abspath("server")
    html_path = os.path.join(server_dir, "Serve-Page.html")
    version_file = os.path.join(server_dir, "version_info.txt")
    icon_file = os.path.join(server_dir, "icon.ico")
    script_file = os.path.join(server_dir, "bridge_serve_python.py")

    cmd = [
        "pyinstaller",
        "--onefile",
        "--noconsole",
        "--name", "Build Stream",
        "--version-file", version_file,
        "--icon", icon_file,
        "--add-data", f"{html_path}{os.pathsep}.",
        "--specpath", "server",
        script_file
    ]
    
    print(f"Running: {' '.join(cmd)}")
    subprocess.check_call(cmd)
    
    # 3. Move the EXE to the target directory
    exe_name = "Build Stream.exe"
    dist_path = os.path.join("dist", exe_name)
    target_path = os.path.join("target", "debug", "server", exe_name)
    
    os.makedirs(os.path.dirname(target_path), exist_ok=True)
    
    if os.path.exists(dist_path):
        shutil.copy2(dist_path, target_path)
        print(f"SUCCESS: EXE copied to {target_path}")
        
        # Also copy to server/ directory for development testing
        shutil.copy2(dist_path, os.path.join("server", exe_name))
        print(f"SUCCESS: EXE copied to server/{exe_name}")
    else:
        print("ERROR: Build failed, EXE not found in dist/")

if __name__ == "__main__":
    build_exe()
