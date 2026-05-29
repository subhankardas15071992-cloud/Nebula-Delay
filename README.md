# Nebula Delay

*A lightweight monophonic delay processor by Nebula Audio.*

Nebula Delay is the mono sibling of Nebula Stereo Delay. It keeps the same double-precision delay core, lock-free meters, MIDI learn workflow, hard bypass, A/B comparison, undo/redo, preset management, tempo sync, filters, and oversampling, but exposes a true 1-in/1-out plugin for mono tracks.
---
**Screenshot of macOS and Linux variant that uses EGUI:**
<img width="709" height="642" alt="image" src="https://github.com/user-attachments/assets/9d76d47e-2e96-449a-8bc2-4afba439dbda" />
---
**Screenshot of Windows variant that uses Direct2D**
<img width="700" height="638" alt="image" src="https://github.com/user-attachments/assets/77b0fe62-1cd5-4077-8b50-7c5ce653165e" />
---
**What's new in v1.2.0**
- **Freely resizable plugin windows** - The plugin window is now freely resizable on all platforms.

**What's new in v1.1.0**
- **Tweaked the filter knobs for better aesthetics** - Now even the arc on the LPF knob moves counterclockwise to follow the knob.
- **Microsoft Windows specific UI tweaks** - The Direct2D UI used on Microsoft Windows has been tweaked to look more coherent and the Note drop down menu going out of bound has been fixed.
- **Microsoft Windows rendering fixes** - Fixed an issue on the Direct2D variant wherein launching the Stereo Delay and our mono Delay together used to replicate the same screen across the two delays. This issue was caused due to shared class name in the tow plugins. It has been fixed now.
---
## Formats

| Platform | Formats |
| --- | --- |
| macOS | CLAP, VST3 |
| Linux | CLAP, VST3 |
| Windows | VST3 |

Windows CLAP builds remain disabled because of the current NIH-plug CLAP compatibility issues on Windows.

## Controls

| Control | Range / Options | Default |
| --- | --- | --- |
| Input Level | -50 dB to +50 dB | 0 dB |
| Delay Time | 5 ms to 2000 ms | 500 ms |
| Note | 1/1 through 1/64, triplet, dotted | 1/4 |
| Tempo Sync | Free / Sync | Free |
| :2 / x2 | Halve / double delay time | Off |
| HPF | 20 Hz to 20 kHz | 20 Hz |
| HPFS | 1 dB/oct to 100 dB/oct | 12 dB/oct |
| LPF | 20 kHz down to 20 Hz | 20 kHz |
| LPFS | 1 dB/oct to 100 dB/oct | 12 dB/oct |
| Feedback | 0% to 100% | 40% |
| Phase Invert | Normal / inverted delayed signal | Normal |
| Oversampling | Off, 2x, 4x, 6x, 8x | Off |
| Dry Level | 0% to 100% | 100% |
| Wet Level | 0% to 100% | 100% |
| Output Level | -50 dB to +50 dB | 0 dB |

The phase invert control flips the delayed signal before it is mixed with the dry signal. Used with the Dry and Wet level knobs, this can create comb-filter and cancellation textures while keeping the dry path intact.

## Build

```bash
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

Bundle builds:

```bash
./scripts/build_macos.sh
./scripts/build_linux.sh
powershell -ExecutionPolicy Bypass -File scripts/build_windows.ps1
```

The build scripts create properly structured CLAP/VST3 bundles under `build/`.

## License

Nebula Delay is open-source software licensed under the GNU Affero General Public License v3. See [LICENSE](LICENSE) for details.

---

**Reporting Issues:**
For reporting any issues create an issue on the Github repository, and while creating the issue do mention your email ID in the issue. The issues of paid customers will be solved on priority basis (Minimum payment of $10). Free customers are expected to workout any issues on their own, no support will be provided to them.
