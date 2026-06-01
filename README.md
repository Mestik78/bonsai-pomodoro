# Bonsai Pomodoro

A beautiful Terminal User Interface (TUI) Pomodoro application written in Rust, inspired by the mobile app "Forest". It helps you stay focused by growing a digital bonsai tree while you work.

> **Note:** This project was built using **vibe coding** (AI-assisted development).

## Features
- Minimalist TUI built with Ratatui.
- Real-time ASCII bonsai tree growth animation.
- Persistence across sessions (saves state automatically).
- Review your completed sessions in the "Forest" tab.
- Quick minute and second adjustments via keyboard shortcuts.

## Controls
### Timer Tab
- **Space:** Pause / Resume timer
- **r:** Reset timer
- **f:** Finish timer early
- **Up / Down:** Adjust minutes
- **Ctrl + Up / Down:** Adjust seconds
- **Left / Right:** Switch between tabs
- **q:** Quit application

### Forest Tab
- **Up / Down / Left / Right:** Navigate between completed sessions
- **Enter:** Enter a specific day to view individual sessions (bonsais)
- **Esc:** Go back to the day view
- **q:** Quit application

## Installation
Ensure you have `cargo` installed.
```bash
git clone https://github.com/Mestik78/bonsai-pomodoro.git
cd bonsai-pomodoro
cargo build --release
```
The binary will be available at `target/release/bonsai_pomodoro`.

## License
[MIT](LICENSE)
