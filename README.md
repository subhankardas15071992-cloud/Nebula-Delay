# Nebula Delay

*A lightweight monophonic delay processor by Nebula Audio.*

Nebula Delay is the mono sibling of Nebula Stereo Delay. It keeps the same double-precision delay core, lock-free meters, MIDI learn workflow, hard bypass, A/B comparison, undo/redo, preset management, tempo sync, filters, and oversampling, but exposes a true 1-in/1-out plugin for mono tracks.
---
**Screenshot of macOS and Linux variant that uses EGUI:**
<img width="703" height="642" alt="image" src="https://github.com/user-attachments/assets/76659636-6822-4d52-b1cf-22ad9a4a60c5" />
---
**Screenshot of Windows variant that uses Direct2D**
<img width="710" height="611" alt="image" src="https://github.com/user-attachments/assets/d9a98932-b69d-41ea-938c-5b3cae1c8007" />
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
For reporting any issues create an issue on the Github repository, and while creating the issue do mention your email ID in the issue. The issues of paid customers will be solved on priority basis. Free customers are expected to workout any issues on their own, no support will be provided to them.
