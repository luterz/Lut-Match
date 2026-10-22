# LutMatch Studio 🎬✨

[![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Windows-blue.svg)](https://github.com)
[![Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![DaVinci Resolve](https://img.shields.io/badge/Compatibility-DaVinci%20Resolve-red.svg)](https://www.blackmagicdesign.com/products/davinciresolve)

**LutMatch Studio** is a native desktop application built in **Rust** designed to extract the color grade / mood from any reference photo or video and replicate it onto your **Rec.709** footage, generating a standard **3D LUT (`.cube`)** for **DaVinci Resolve**, Premiere Pro, or Final Cut.

Featuring an ultra-clean **Apple Liquid Light** three-pane interface where you can preview your base footage, the target color reference, and the live color-matched output side by side.

<img width="2674" height="1778" alt="image" src="https://github.com/user-attachments/assets/4c31a858-3f16-4c26-b7c6-7e270868522d" />

---

## 🌟 Key Features

- **Three-Pane Synchronized UI**:
  - **Pane 1 (Source)**: Load your flat / Rec.709 video (`MP4`, `MOV`, `MKV`, `ProRes`) or photo (`JPG`, `PNG`, `TIFF`, `WEBP`).
  - **Pane 2 (Reference)**: Load the inspiration image or video with the color correction you want to copy.
  - **Pane 3 (Live Output)**: Live preview of your original Rec.709 footage with the generated 3D LUT applied in real time.
- **Perceptual CIE Lab Color Transfer**: Accurate statistical mapping of luminance ($L^*$), color deviations ($a^*$, $b^*$), with intelligent contrast clamping to prevent banding and posterization.
- **Real-Time Look Intensity Slider**: Adjust the color transfer intensity seamlessly from **0% to 100%**.
- **Industry Standard 3D LUT**: Exports 33x33x33 Adobe/Blackmagic `.cube` files compatible with all major NLEs.
- **One-Click DaVinci Resolve Export**: Automatically saves the LUT directly into DaVinci Resolve's LUT library folder on both macOS and Windows.
- **Blazing Fast**: Pure native Rust binary (~1.8 MB), no bloated runtimes.

---

## 🖥️ Layout Preview

```text
┌─────────────────────────┬─────────────────────────┬─────────────────────────┐
│     1. Base Rec.709     │  2. Riferimento Color   │   3. LUT Applicata      │
│                         │                         │                         │
│  [Source Preview]       │  [Mood Reference]       │  [Live Output Preview]  │
│  File: my_clip.mp4      │  File: cinema_mood.jpg  │  LUT 33x33 / Rec.709    │
│                         │                         │                         │
│   [ Scegli File... ]    │   [ Scegli File... ]    │      (Live Update)      │
└─────────────────────────┴─────────────────────────┴─────────────────────────┘
┌─────────────────────────────────────────────────────────────────────────────┐
│  Intensità Look: [==== 100% ====]     [Salva con nome...]  [Salva in DaVinci]│
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 🚀 Download & Installation

Pre-built binaries are available under [Releases](../../releases):
- **macOS (Apple Silicon & Intel)**: Download `LutMatch-macOS.zip`, extract and double-click `LutMatch.app`.
- **Windows (10 / 11 64-bit)**: Download and run `LutMatch.exe`.

---

## 🛠️ Building from Source

### Prerequisites
- Install [Rust & Cargo](https://rustup.rs/):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- Install [FFmpeg](https://ffmpeg.org) (used for instant video frame sampling):
  - **macOS**: `brew install ffmpeg`
  - **Windows**: `winget install Gyan.FFmpeg` or download from ffmpeg.org

### macOS Build
```bash
git clone https://github.com/your-username/lut-match.git
cd lut-match
./build_mac.sh
```
This builds an optimized release binary and bundles `LutMatch.app`.

### Windows Build
```cmd
git clone https://github.com/your-username/lut-match.git
cd lut-match
build_windows.bat
```
Or simply:
```cmd
cargo build --release
```
The executable will be located at `target\release\lut-match.exe`.

---

## 🎨 Where are LUTs saved for DaVinci Resolve?

When clicking **"Salva in DaVinci Resolve"**, the app deposits the `.cube` file directly into DaVinci's default folder:
- **macOS**: `~/Library/Application Support/Blackmagic Design/DaVinci Resolve/LUT/LutMatch/<name>.cube`
- **Windows**: `C:\ProgramData\Blackmagic Design\DaVinci Resolve\Support\LUT\LutMatch\<name>.cube`

> [!TIP]
> In DaVinci Resolve, go to the **Color** page, open the **LUTs** browser, right-click anywhere and select **Refresh**. Your LUT will immediately appear in the `LutMatch` folder!

---

## 📄 License
Released under the [MIT License](LICENSE).
