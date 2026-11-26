# VIctory on a 4 GB laptop card

Two lines of [VIctory](https://github.com/Marvisak/VIctory), a compatibility layer that runs an Xbox
Series title natively on Windows from files the user supplies, brought to one machine with little
video memory and measured there:

- Windows 11, NVIDIA RTX 3050 Laptop GPU **4 GB** (32-lane waves), AMD Ryzen 7 7435HS, 24 GB RAM.
- The title's Series S profile (`--console series-s`), 1920x1080.

**This repository contains no game files, code or assets.** You need your own copy of the game. It is
not affiliated with or endorsed by the game's developer or publisher.

It is not finished. The point of publishing it is that the fast line and the good-looking line are
side by side with numbers from the same card, so that others can close the gap. The gap is light.

## The two branches

| Branch | What it is | In the first gameplay scene, on this card | Picture |
|---|---|---|---|
| `main` | [fxceless/victory-fork](https://github.com/fxceless/victory-fork) `main` + pull requests and ports listed below | **5.4 to 6.7 fps** (145 to 185 ms a frame at about 4,400 draws) | clothes lit by the added ambient light, window glass right; skin and rooms still too dark |
| `coolfox-4gb` | codec-d's `cool_fox` at `f4b48ad` (the leading public build on PortWatch until it was taken down) + the render-scale port | **2.4 to 2.5 fps** (400 to 418 ms) | the best here: face, neck, hair, clothes and walls lit |

The two are not merged. A trial merge stops on 39 files and about 300 hunks, most of them in
`crates/victory-gpu/src` (`host.rs` 91, `draw.rs` 39, `indirect.rs` 30, `descriptor/host.rs` 26,
`shadow.rs` 23, `watch.rs` 22). They share a base (`a26859f`, 25 November 2026).

### Why `coolfox-4gb` is slow here, and why that is the interesting part

On this card that tree prepares a frame faster than `main` does (41 to 48 ms against 70 or more),
and then waits for the card: 344 to 360 ms of every frame, the card about 97 % busy by its own
count. Halving the render scale does not change it; skipping the ray-tracing resolve passes does not
change it. With `VICTORY_GPU_TIMING=1`, one group of ten lists with 22 compute programs takes about
113 ms a frame; their hashes are in [`docs/coolfox-113ms-group.txt`](docs/coolfox-113ms-group.txt).
What those programs are, and why they cost that on 32-lane waves, is not known yet. If that group
can be made cheap, `coolfox-4gb` gets the picture at something like `main`'s frame rate.

## What is missing

- **Light in shadow.** The title's ray-tracing structure builder does not run on 32-lane cards, so
  every ray misses. `main` skips the passes that turn those misses into white haze and adds a
  stand-in ambient light (`VICTORY_AMBIENT`, from cool_fox). Rooms stay too dark.
- **Skin.** The title composites characters' skin once while a scene loads. With
  `VICTORY_BAKE_SOURCES=1` this card logs, in one run: 159 bake draws, 82 of them sampling a texture
  that was not there yet, 443 sources still waiting, 123 skipped because the card had no room. On
  4 GB that is structural.
- **Run-to-run variance.** Whether the player character is lit differs between identical runs (also
  reported on an RTX 5070 Ti in fork pull request #799). One run proves little; the numbers above
  are from one run each unless said.
- Hair is dithered (it wants the temporal resolve); two 64-lane compute programs are still refused.

## Build and run

Windows 11 x64, a card with Direct3D 12, shader model 6.6 and mesh shaders, the pinned Rust
toolchain (`rust-toolchain.toml`) and the MSVC build tools with a Windows SDK.

```powershell
cargo build --profile diag -p victory --features diagnostics
```

```powershell
scripts\play-4gb.ps1 -Exe 'D:\Games\MyDump\game_scarlett_master.exe'
```

The script sets the switches the figures above were measured with. The `diag` build is the default
because the ray-tracing skip list is a diagnostics switch. The first run after a build translates
every shader again and is slow for some minutes.

Switches added or ported here (all off unless the script sets them):

| Switch | What it does | From |
|---|---|---|
| `VICTORY_RENDER_SCALE=<f>`, `VICTORY_OUTPUT=<w>x<h>` | the title's internal render scale and its real output size (patches to the 1.0.2.1 executable, compared byte for byte first) | avatarmanz/VIctory-7900XTX; the title patches from ZigZagBoi1/ScarlettPort and fork pull request #53 |
| `VICTORY_AMBIENT`, `VICTORY_AMBIENT_AIR` | a stand-in for the title's traced indirect light | codec-d/cool_fox (`82ca134`, `7da523a`) |
| `VICTORY_SKIP_PIPELINES` with `scripts/rt_off_hashes.txt` | skips the ray-tracing resolve and traced-shadow passes | the list is trustno1-coder/VIctory-NVIDIA's |
| `VICTORY_BLEND_INT_OFF=1` | blending off for integer render targets, which Direct3D 12 refuses | here (fork pull request #1162; #51 fixes the same) |
| `VICTORY_FULLSCREEN=1\|borderless` | a framed window as large as the work area, or a frameless one over the display | here |
| `VICTORY_TEXTURE_BUDGET_MB` floor | the texture floor follows the budget on small cards | here (fork pull request #1122) |

On `main`, half render scale is worth about 13 % (165-190 ms to 145-157 ms). On `coolfox-4gb` it is
worth nothing and costs background detail: leave it at the title's own there.

## Measured and ruled out on this card

- `VICTORY_TEXTURE_BUDGET_MB`: 1024 → 154 ms, 1536 → 186 ms, 2048 → 272 ms a frame; sharper textures
  with more.
- More host copies in the card's memory (`VICTORY_CARD_BUFFERS_MIN_KB=64`): no gain. Fewer
  (`VICTORY_CARD_BUFFERS_MB=128`): 30 % slower.
- `VICTORY_ABSENT_ONCE` with `VICTORY_ABSENT_MS=told`: questions to Windows fall from about 890,000
  to about 50 per 20 s; no frame gain seen.
- The loose-LDS fallback of fork pull request #799 builds 4 of the 6 compute programs this card
  refuses; the picture washed out in two runs with all four built, and not in either half.
- Another project's 25 fps claim on a 16 GB card is 2.4 fps here (`coolfox-4gb`).

## Credits

Everything here stands on other people's work, and the commits keep their authors.

- [Marvisak/VIctory](https://github.com/Marvisak/VIctory): the compatibility layer itself.
- [fxceless/victory-fork](https://github.com/fxceless/victory-fork) (hyphennn): the base of `main`.
  Its README is kept as [README.fxceless.md](README.fxceless.md).
- codec-d (`cool_fox`) and [avatarmanz/VIctory-7900XTX](https://github.com/avatarmanz/VIctory-7900XTX):
  the base of `coolfox-4gb`; the ambient light and the render-scale switches on `main`.
- [trustno1-coder/VIctory-NVIDIA](https://github.com/trustno1-coder/VIctory-NVIDIA): the ray-tracing
  skip list and the explanation of the white haze.
- Fork pull requests merged on `main` with their authors' commits: #1081 and #798 (Kyle
  Soderlund), #1002, #592, #156 (sedzz, with a commit by ryotdev), #799 (Massive-Adrian), #1261
  (Garu667); and, through `main` of the fork, #1260
  (LeonardoS11) and #1264 (UwUCammie).
- HLVM04, atmacafuat and tippythetaidum for measurements and corrections in the fork's threads.

Measured, merged and written with AI assistance (Claude Code and Codex) on the machine above.

## License

GPL-3.0-or-later, as VIctory. See [LICENSE](LICENSE).
