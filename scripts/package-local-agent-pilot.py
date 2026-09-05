#!/usr/bin/env python3
"""Package an already-built isolated macOS pilot; no installation or deployment."""
import argparse
import hashlib
import json
import plistlib
import shutil
import subprocess
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--binary", type=Path, required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
destination = root / "dist-local-agent"
app = destination / "DAARION Edge Local Pilot.app"
contents = app / "Contents"
(contents / "MacOS").mkdir(parents=True, exist_ok=True)
(contents / "Resources").mkdir(exist_ok=True)
binary = contents / "MacOS" / "edge-local-agent"
shutil.copy2(args.binary, binary)
binary.chmod(0o755)
shutil.copy2(root / "src-tauri/icons/icon.icns", contents / "Resources/icon.icns")
with (contents / "Info.plist").open("wb") as output:
    plistlib.dump({"CFBundleName": "DAARION Edge Local Pilot", "CFBundleDisplayName": "DAARION Edge Local Pilot",
                  "CFBundleIdentifier": "city.daarion.edge.local-agent-pilot", "CFBundleExecutable": "edge-local-agent",
                  "CFBundlePackageType": "APPL", "CFBundleShortVersionString": "0.2.2", "CFBundleVersion": "4",
                  "CFBundleIconFile": "icon.icns", "LSMinimumSystemVersion": "11.0", "NSHighResolutionCapable": True}, output)
subprocess.run(["codesign", "--force", "--deep", "--sign", "-", "--timestamp=none", str(app)], check=True, capture_output=True)
subprocess.run(["codesign", "--verify", "--deep", "--strict", str(app)], check=True, capture_output=True)
receipt = {"platform": "macOS", "architecture": "aarch64", "signature": "ad-hoc; not notarized",
           "signed_binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(), "app": app.name}
(destination / "package-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
print(json.dumps(receipt))
