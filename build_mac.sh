#!/usr/bin/env bash
# Script di build per macOS
set -e

echo "Compilazione release in corso..."
cargo build --release

echo "Confezionamento LutMatch.app..."
mkdir -p LutMatch.app/Contents/MacOS
cp target/release/lut-match LutMatch.app/Contents/MacOS/LutMatch
chmod +x LutMatch.app/Contents/MacOS/LutMatch

cat << 'EOF' > LutMatch.app/Contents/Info.plist
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>LutMatch</string>
    <key>CFBundleIdentifier</key>
    <string>com.lutmatch.studio</string>
    <key>CFBundleName</key>
    <string>LutMatch Studio</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

echo "Completato! LutMatch.app pronta all'uso."
