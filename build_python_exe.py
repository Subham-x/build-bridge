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
    cmd = [
        "pyinstaller",
        "--onefile",
        "--name", "Build Stream by build Bridge",
        "--version-file", "version_info.txt",
        "bridge_serve_python.py"
    ]
    
    print(f"Running: {' '.join(cmd)}")
    subprocess.check_call(cmd)
    
    # 3. Move the EXE to the target directory
    # We want it in the same folder as our Rust executable
    exe_name = "Build Stream by build Bridge.exe"
    dist_path = os.path.join("dist", exe_name)
    target_path = os.path.join("target", "debug", exe_name)
    
    os.makedirs(os.path.dirname(target_path), exist_ok=True)
    
    if os.path.exists(dist_path):
        shutil.copy2(dist_path, target_path)
        print(f"SUCCESS: EXE copied to {target_path}")
        
        # Also copy to root for easy development testing
        shutil.copy2(dist_path, "bridge_serve_python.exe")
    else:
        print("ERROR: Build failed, EXE not found in dist/")

if __name__ == "__main__":
    build_exe()
