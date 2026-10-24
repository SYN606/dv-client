import zipfile
import os

zip_path = "dist/DigitalVigital-v1.0.0-windows-x64-portable.zip"
with zipfile.ZipFile(zip_path, "w", zipfile.ZIP_DEFLATED) as z:
    z.write("dv-launcher-app.exe", "DigitalVigitalLauncher.exe")
    z.write("WebView2Loader.dll", "WebView2Loader.dll")
    z.write("assets/logo.png", "assets/logo.png")
    z.write("assets/logo.ico", "assets/logo.ico")
    z.write("README.md", "README.md")
    z.write("LICENSE", "LICENSE")

print(f"Created {zip_path}: {os.path.getsize(zip_path)} bytes")
