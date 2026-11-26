# VIctory on a 4 GB laptop card

In the Series S boat sequence, the 4 GB RTX 3050 Laptop rendered the characters, shoreline and sunlight on the water. Two captured world views read 222.3 and 227.9 ms/frame (4 FPS on the window counter). Earlier shots in the same run had white transitions and underlit faces. [Capture timeline and settings](https://github.com/Dathinel/victory-4gb/issues/1#issuecomment-7204395814).

Two lines of [VIctory](https://github.com/Marvisak/VIctory), a compatibility layer that runs an Xbox
Series title natively on Windows from files the user supplies, brought to one machine with little
video memory and measured there:

- Windows 11, NVIDIA RTX 3050 Laptop GPU **4 GB** (32-lane waves), AMD Ryzen 7 7435HS, 24 GB RAM.
- The title's Series S profile (`--console series-s`), 1920x1080.

**This repository contains no game files, code or assets.** You need your own copy of the game. It is
not affiliated with or endorsed by the game's developer or publisher.

## Where it stands (26 November 2026)

On the machine above, branch `coolfox-4gb`, one 40-minute session played with an Xbox controller:
from the first scene through the first mission and its chase to the boat mission in the open world,
with no crash and no device loss. Median 4.0 fps over the session (423 one-second
readings); in separate runs since, 5.2 to 7.0 fps in the open-world boat scene with the title's own
upscaler at half scale (4.4 to 4.5 without), and 5.2 to 5.5 fps in the first room.

- **Good:** daylight outdoors. Sky, water, vegetation, vehicles, characters and faces in the boat
  mission and its cutscene, the minimap and the interface.
- **Not good yet:** speed (4 to 5 fps); interiors and night (dark rooms, characters as silhouettes
  with burned edges); characters' skin baked black or flat in some runs (a fix is in, see below).

So: it can be played through, slowly, on a 4 GB laptop card. It is not finished, and pull requests
are what would move it fastest: see [Contributing](#contributing).

## The two branches

| Branch | What it is | In the first gameplay scene, on this card | Picture |
|---|---|---|---|
| `main` | [fxceless/victory-fork](https://github.com/fxceless/victory-fork) `main` + pull requests and ports listed below | **5.4 to 6.7 fps** (145 to 185 ms a frame at about 4,400 draws) | clothes lit by the added ambient light, window glass right; skin and rooms still too dark |
| `coolfox-4gb` | codec-d's `cool_fox` at `f4b48ad` (the leading public build on PortWatch until it was taken down) + the render-scale port | **5.2 to 5.5 fps** (184 to 192 ms) with the switch set below; 2.4 to 2.5 fps (400 to 418 ms) with its own NVIDIA launcher's set | the best here: face, neck, hair, clothes and walls lit; with the ray-tracing passes skipped the room is lit warm and the haze is gone (one run); daylight outdoors right |

The two are not merged. A trial merge stops on 39 files and about 300 hunks, most of them in
`crates/victory-gpu/src` (`host.rs` 91, `draw.rs` 39, `indirect.rs` 30, `descriptor/host.rs` 26,
`shadow.rs` 23, `watch.rs` 22). They share a base (`a26859f`, 25 November 2026).

### `coolfox-4gb` on a small card: which of its switches cost the card

With the switch set of that tree's own NVIDIA launcher (`Play-VIctory.cmd pareto-residency`) this
card is about 97 % busy and the frame waits for it: 344 to 370 ms "on the GPU" of a 400 to 425 ms
frame, 2.4 fps. That does not move with the render scale, with the ray-tracing resolve passes
skipped, with depth of field and autofocus skipped, or with `VICTORY_CARD_STEPS=0 VICTORY_RELAY=1
VICTORY_CARD_BUFFERS_MIN_KB=1024`.

With only `VICTORY_LEVELS_ON_DEMAND=1 VICTORY_TEXTURE_NO_WAIT=1` the same tree, same scene, runs at
224 to 228 ms a frame (142 to 150 ms on the GPU, 84 to 90 ms copying back): 4.4 to 4.6 fps. Adding
`VICTORY_AMBIENT=0.8 VICTORY_AMBIENT_AIR=1.5` costs nothing measurable and lights the clothes and
the neck. Adding `VICTORY_THIN=1 VICTORY_JOIN_PARTS=8 VICTORY_UPLOAD_AHEAD=1 VICTORY_SHOW_AFTER=1
VICTORY_MAP_THREADS=1 VICTORY_CARD_AWAKE=1` on top: 238 to 240 ms, so not those either.

Measured since: the seven remaining switches are not it either (with `VICTORY_RELAY_COPY=1
VICTORY_CARD_REHASH_RUNS=1 VICTORY_JOIN_COPIES=1 VICTORY_INDIRECT_WINDOW=1 VICTORY_INDIRECT_BATCH=2`
added the frame is 204 to 208 ms, 4.8 to 4.9 fps). What loads the card is `VICTORY_CARD_STEPS=2`
and/or `VICTORY_CARD_BUFFERS_MIN_KB`; the continuation's 4 GB launcher, which keeps them, runs at
2.2 to 2.8 fps here.

The set this card runs best with so far:

```
VICTORY_LEVELS_ON_DEMAND=1 VICTORY_TEXTURE_NO_WAIT=1 VICTORY_AMBIENT=0.8 VICTORY_AMBIENT_AIR=1.5
VICTORY_RELAY_COPY=1 VICTORY_CARD_REHASH_RUNS=1 VICTORY_JOIN_COPIES=1 VICTORY_INDIRECT_WINDOW=1
VICTORY_INDIRECT_BATCH=2 VICTORY_NAN_SCRUB=1 VICTORY_TEXTURE_DROP_MIPS=1 VICTORY_BAKE_SOURCES=full
VICTORY_UPSCALER=6 VICTORY_RENDER_SCALE=0.5
```

(with `VICTORY_SKIP_PIPELINES` set to the list in `scripts/rt_off_hashes.txt`.)

184 to 192 ms a frame (98 to 111 ms on the GPU, 65 to 67 ms preparing), 5.2 to 5.5 fps, about
3,350 of 3,300 MB of card memory budgeted, no banding from the dropped texture level
(`VICTORY_NAN_SCRUB` and `VICTORY_TEXTURE_DROP_MIPS` are miaohaixiang1-netizen's commits, picked
from cool-fox-12gb). Adding `VICTORY_SKIP_PIPELINES` with the list in
[`scripts/rt_off_hashes.txt`](scripts/rt_off_hashes.txt) (trustno1-coder's, 1,034 programs of the
ray-traced passes) keeps the speed (5.3 to 5.4 fps) and, in the one run so far, gave the best
picture on this card: character and room lit, the white haze gone. One run only; the character's
light varies between identical runs, so this needs repeating.

A dead end worth knowing: `VICTORY_GPU_TIMING=1` named one group of lists at about 113 ms a frame
([`docs/coolfox-113ms-group.txt`](docs/coolfox-113ms-group.txt): depth of field, autofocus, the
scattering diffusion hierarchy, TAA utilities, a wind-field copy). Skipping the depth-of-field and
autofocus programs of that group changed the frame by about 10 ms. The timing run sends lists one
at a time and does not show where the time of a normal run goes.

## What is missing

- **Light in shadow.** The title's ray-tracing structure builder does not run on 32-lane cards, so
  every ray misses. `main` skips the passes that turn those misses into white haze and adds a
  stand-in ambient light (`VICTORY_AMBIENT`, from cool_fox). Rooms stay too dark.
- **Skin.** The title composites characters' skin once while a scene loads, and on this card the
  textures those draws sample are mostly not uploaded yet. `coolfox-4gb` in the boat scene, counting
  only (`VICTORY_BAKE_SOURCES=count`): 107 bake draws, 104 of them sampling a texture that was not on
  the card. With `VICTORY_BAKE_SOURCES=full` (UwUCammie's fork pull request #1264, ported to that
  tree here): 111 bake draws, 0 sampling wrong, 106 textures uploaded ahead of them, 29 MB in all.
  One run each; faces were right in both runs' captures, so the picture gain is not yet shown, only
  the count. A black face seen in an earlier run of the same scene is what this is meant to end.
- **Interiors and night.** A green-brown fog veil lies over interiors, some characters' skin is
  pure black, and night shows burned white edges. Six runs on `coolfox-4gb`, same opening montage,
  captures every 10 s:
  - ray-tracing resolves skipped (the list): veil; 5.2 to 7.0 fps;
  - nothing skipped: veil, plus blocks of broken picture and a face of uninitialised pixels;
  - `VICTORY_BVH_NO_HIT=1` (fxceless `96ff135`, picked onto `coolfox-4gb` here), nothing skipped:
    **no veil in the first run, veil in the repeat** and in a third run with
    `VICTORY_TEXTURE_NO_WAIT=0`; 4.1 to 5.7 fps;
  - `VICTORY_BVH_NO_HIT=1` with the list: veil; 4.8 to 6.4 fps.
  The one veil-free run was the first after a rebuild, with every shader translating again. So
  none of these switches is shown to remove the veil, and what differed in that run is not known
  yet. An earlier version of this README credited the switch; that was one run.
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

On `main`, half render scale is worth about 13 % (165-190 ms to 145-157 ms).

On `coolfox-4gb` with the Series S profile the render scale is the upscaler's, and which upscaler
matters. Same boat scene, one run each, against 4.4 to 4.5 fps (140 to 147 ms on the GPU):

- `VICTORY_RENDER_SCALE=0.5` alone: nothing (the Series S profile picks FidelityFX, which that
  switch does not reach).
- `VICTORY_FFX_SCALE=0.5`, and `VICTORY_OUTPUT=1280x720`: the title divides by zero 49 to 90 s in
  (its FSR4 tables are for one resolution only). Do not use them on Series S.
- **`VICTORY_UPSCALER=6 VICTORY_RENDER_SCALE=0.5`** (the title's own upscaler at half scale):
  **5.2 to 7.0 fps**, 63 to 118 ms on the GPU. The picture is softer, the far background most.
- Skipping the seven compute programs fork issue #1278 names: nothing in this scene (they do not
  run in it). `VICTORY_TEXTURE_BUDGET_MB=1024`: within the noise.

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

## Contributing

Pull requests and issues are welcome on this repository, against `coolfox-4gb` (the line that is
played) or `main`. Small ones are merged fastest. What would help most, in order:

1. **Interiors and night on 32-lane cards**: where the fog veil over interiors comes from (see
   "What is missing"); an indirect-light stand-in for rooms; skin that reads the probe atlas only traced passes fill (Linox-IDK/LNX-victory
   `0f9a071` fills it with the ambient value); the
   ambient stand-in settling by time instead of by frame (avatarmanz `b4ba4bb`, `9576c45`); traced
   reflections clamped to the ambient estimate (avatarmanz pull request #314).
2. **Card time**: the frame waits 100 to 150 ms for a 4 GB card. Which compute passes can be thinned
   or run at lower resolution without losing the picture (fxceless issue #1278 names seven).
3. **Memory**: the tree keeps 3,072 MB of the card for render targets
   (`VICTORY_CARD_ROOM_FOR_TARGETS_MB`), and they hold about 1,350 MB here; a default that follows
   the card (as bolado-dev/mandy-port's `55b3635` does) is untested here.
4. **Measurements from other small cards** (4 to 6 GB, NVIDIA or AMD): frame time, the log's
   one-second summary line, and what the picture does. Text and numbers.

Rules: no game files, captures of the game, dumps or keys in issues, pull requests or commits; keep
the authors of commits you bring from elsewhere (GPL-3.0-or-later) and say where they are from; say
what you measured and on what, and what you did not test. Results that say "this did not help" are
as welcome as fixes: several are listed above.

### Test a shader on your GPU

A Windows PC with a 4 GB GPU can contribute a shader result without any game files. This test
runs a synthetic 64-lane floating-point sum on a Direct3D 12 GPU that supports Shader Model 6.6 and 32-lane waves, then compares
the value read back from the card with a CPU reference. Use the Rust toolchain, MSVC build tools
and Windows SDK listed under [Build and run](#build-and-run).

From the repository root on the `coolfox-4gb` branch, run in PowerShell:

```powershell
git rev-parse --short HEAD
$env:VICTORY_WARP = "0"  # Use the default hardware adapter in this PowerShell session
cargo test --locked -p victory-shader --lib a_sum_of_halves_over_the_wave_is_the_consoles_tree_bit_for_bit -- --nocapture
```

On an RTX 3050 Laptop (4 GB) the hardware readback printed `ran on the hardware device: the sum
0x5914 of the console's tree`. If the output says `not run`, the GPU check was skipped even if
Cargo reports a pass. If it says `0 tests`, check the branch and test name.

Include the commit and the readback, skip message or first failed assertion in your
[GPU report](CONTRIBUTING.md). On a PC with multiple GPUs the test uses the default adapter.
If its identity is uncertain, mention the installed GPUs and mark the tested adapter as unknown. This checks the arithmetic result. Frame time and memory use
require separate measurements.

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
- UwUCammie: the skin-bake source upload (fork pull request #1264), ported to `coolfox-4gb`.
- miaohaixiang1-netizen (cool-fox-12gb): `VICTORY_NAN_SCRUB` and `VICTORY_TEXTURE_DROP_MIPS` on
  `coolfox-4gb`; the DoubleGhosty/cool_fox continuation, merged there.
- HLVM04, atmacafuat and tippythetaidum for measurements and corrections in the fork's threads.

Measured, merged and written with AI assistance (Claude Code and Codex) on the machine above.

## License

GPL-3.0-or-later, as VIctory. See [LICENSE](LICENSE).
