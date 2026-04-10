# Twitch ADC Advisor

A desktop app that live-monitors your League of Legends game and tells you what items to build on Twitch ADC — adapting to the enemy team, your lane matchup, your support, and how much gold you had on first back.

Uses Riot's official [Live Client Data API](https://developer.riotgames.com/docs/lol#live-client-data-api) (runs locally while a game is active). Completely legal and Vanguard-safe.

## What it does

- Recommends a full item build (Crit/Lethality, Pure Crit, or On-Hit) based on enemy composition
- Adjusts the first item recommendation based on how much gold you had on first back (Yun Tal gated at 1300g)
- Flags situational items when the enemy has healers, tanks, AP threats, assassins, or hard CC
- Shows tips for your specific support synergy and lane matchup
- Updates every 3 seconds during a game
- Separate window — alt-tab to it or pin it always-on-top

---

## Running from a pre-built installer

### Mac (Apple Silicon)

1. Download `ItemBuildL_0.1.0_aarch64.dmg` from [Releases](https://github.com/olindkri/ItemBuildL/releases)
2. Open the `.dmg`, drag **ItemBuildL** to Applications
3. If macOS says the app is damaged, run this in Terminal:
   ```bash
   xattr -cr /Applications/ItemBuildL.app
   ```
4. Launch **ItemBuildL** normally after running that command
5. Start a League of Legends game — the advisor will update automatically

### Windows

1. Download `ItemBuildL_0.1.0_x64-setup.exe` from [Releases](https://github.com/olindkri/ItemBuildL/releases)
2. Run the installer (Windows Defender SmartScreen may warn — click "More info" → "Run anyway" since the app is unsigned)
3. Launch the app from the Start Menu or Desktop shortcut
4. Start a League of Legends game — the advisor will update automatically

---

## Building from source

### Prerequisites

**All platforms:**
- [Node.js](https://nodejs.org/) 18 or later
- [Rust](https://rustup.rs/) (stable)

**Mac only:**
- Xcode Command Line Tools: `xcode-select --install`

**Windows only:**
- [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (select "Desktop development with C++")
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) — already included on Windows 11, install manually on Windows 10

### Install dependencies

```bash
git clone https://github.com/olindkri/ItemBuildL.git
cd ItemBuildL
npm install
```

### Run in development mode

```bash
npm run tauri dev
```

Opens the app window with hot-reload. Start a League of Legends game to see live data.

### Build a production installer

```bash
npm run tauri build
```

Output:
- **Mac:** `src-tauri/target/release/bundle/dmg/ItemBuildL_x.x.x_aarch64.dmg`
- **Windows:** `src-tauri/target/release/bundle/nsis/ItemBuildL_x.x.x_x64-setup.exe`

---

## How it works

The app polls `https://127.0.0.1:2999/liveclientdata/allgamedata` — a local HTTP server that League of Legends runs while a game is active. This is the same API used by U.GG, Overwolf, and other sanctioned tools. When no game is running, the endpoint is unavailable and the app shows "Waiting for game...".

The recommendation engine runs entirely offline — no internet connection needed during a game. All item, synergy, and matchup data is bundled into the binary.

---

## Updating item data

Item builds, synergies, and matchup tips are stored in:

```
src-tauri/src/knowledge/
├── synergies.json   # Support champion tips
└── matchups.json    # Enemy champion archetypes and tips
```

Edit these files and rebuild to update recommendations after a patch.
