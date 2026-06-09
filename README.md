# Clocky
Simple clock app - AI does something better than slop

A polished, cross-platform desktop clock application built with Rust. Features a live clock, stopwatch, countdown timer, and alarm — all wrapped in a themeable UI with wallpaper support.

> **Built with the help of [Qwen](https://qwenlm.github.io/) and [Claude](https://claude.ai) AI assistants.**

---

## ✨ Features

### 🕰 Clock
- Displays the current time and full date
- Switchable between **12-hour** (AM/PM) and **24-hour** format

### ⏱ Stopwatch
- Start, pause, and reset
- Displays hours, minutes, seconds, and centiseconds

### ⏳ Timer
- Set any duration in minutes and seconds
- Animated circular progress ring with a glowing comet head
- System sound alert when time is up
- Pause, resume, and reset support

### ⏰ Alarm
- Set an alarm for any hour and minute
- Animated ring while the alarm is pending
- System sound + on-screen alert when the alarm triggers
- Respects the selected time format (12 / 24 hr)

### ⚙️ Settings
- **Time format** — toggle between 12-hour and 24-hour globally
- **Colour themes** — five eye-soothing built-in palettes:

  | Theme | Accent Colour | Mood |
  |-------|--------------|------|
  | Midnight | Soft indigo-blue | Deep, calm |
  | Aurora | Emerald teal | Fresh, natural |
  | Sakura | Rose pink | Warm, gentle |
  | Dusk | Amber gold | Cosy, warm |
  | Slate | Cool cyan | Clean, modern |

- **Wallpaper** — load any image (PNG, JPG, WEBP, BMP) as the background; the image is always **cover-cropped** (fills the window, centred, never stretched or distorted)
- **Opacity slider** — adjust wallpaper visibility from 5% to 100%

---

## 📸 Screenshots

<img width="747" height="673" alt="image" src="https://github.com/user-attachments/assets/c55ffa5f-ea4f-4b6e-b879-850d61f0c814" />
<img width="748" height="661" alt="image" src="https://github.com/user-attachments/assets/e0e160a9-b245-4194-855e-b32189418be0" />
<img width="748" height="669" alt="image" src="https://github.com/user-attachments/assets/cc760611-d498-472f-8ef1-6f444e1025fa" />
<img width="747" height="664" alt="image" src="https://github.com/user-attachments/assets/9c67b915-e48f-4694-b86a-5c3d6dbb4630" />





## 🚀 Installation

### Download a pre-built binary

Go to the [**Releases**](../../releases) page and download the file for your operating system:


## 🔨 Build from Source

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)

**Linux only** — install these system libraries first:

```bash
sudo apt-get install -y \
  libgtk-3-dev \
  libxcb-render0-dev \
  libxcb-shape0-dev \
  libxcb-xfixes0-dev \
  libxkbcommon-dev \
  libssl-dev \
  libglib2.0-dev \
  libcanberra-gtk3-module
```

### Build & run

```bash
git clone https://github.com/YOUR_USERNAME/clock-app.git
cd clocky
cargo run --release
```

The compiled binary will be at `target/release/clock-app` (or `clock-app.exe` on Windows).

---

## 🗂️ Project Structure

```
clocky/
├── .github/
│   └── workflows/
│       └── release.yml     # CI/CD — builds for all platforms on every tag
├── src/
│   └── main.rs             # entire application source
├── Cargo.toml              # dependencies and build config
└── README.md
```

---

## 📦 Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| [eframe](https://crates.io/crates/eframe) | 0.29 | Native window + app framework |
| [egui](https://crates.io/crates/egui) | 0.29 | Immediate-mode GUI |
| [chrono](https://crates.io/crates/chrono) | 0.4 | Local time and date |
| [image](https://crates.io/crates/image) | 0.25 | Wallpaper image loading |
| [rfd](https://crates.io/crates/rfd) | 0.15 | Native file-picker dialog |

---

## 🔁 Release & CI/CD

This repo uses **GitHub Actions** to automatically build release binaries for all three platforms whenever a version tag is pushed.

```
git tag v1.2.0
git push origin v1.2.0
```


## 🤖 AI Assistance

This application was developed with the assistance of:

- **[Qwen](https://qwenlm.github.io/)** — Alibaba's AI assistant, used for initial code generation and feature implementation
- **[Claude](https://claude.ai)** — Anthropic's AI assistant, used for code refinement, UI polish, theming system, wallpaper support, settings panel, and the improved circular animation

---

## 📄 License

MIT License — feel free to use, modify, and distribute.

---

## 🙌 Contributing

Pull requests are welcome. For major changes, open an issue first to discuss what you'd like to change.
