# VIctory on a 4 GB laptop card

Two lines of [VIctory](https://github.com/Marvisak/VIctory), a compatibility layer that runs an Xbox
Series title natively on Windows from files the user supplies, brought to one machine with little
video memory and measured there:

- Windows 11, NVIDIA RTX 3050 Laptop GPU **4 GB** (32-lane waves), AMD Ryzen 7 7435HS, 24 GB RAM.
- The title's Series S profile (`--console series-s`), 1920x1080.

**This repository contains no game files, code or assets.** You need your own copy of the game. It is
not affiliated with or endorsed by the game's developer or publisher.

## Where it stands (28 November 2026)

On the machine above, branch `coolfox-4gb`:

- **It plays through.** One 40-minute session with an Xbox controller, from the first scene
  through the first mission and its chase to the boat mission in the open world, with no crash and
  no device loss (median 4.0 fps then, before the gains below).
- **About 7 fps driving a boat in the open world, 8.7 to 9 standing still** at the start of free
  play (112 to 115 ms a frame at about 3,500 draws), against 4.1 to 4.9 two days earlier. Most of
  that came from work that only serves ray tracing, which does not run on this card: a per-pass GPU
  profile found it and a skip list removes it (issue #4, pull request #37).
- **The sea has its light.** The three water transmission programs that 32-lane cards refused now
  translate (`VICTORY_WAVE_WIDE=last`, ported from fxceless/victory-fork pull request #1141): the
  sea is a clear turquoise with the light through it instead of a dark grey-blue, for about 3 % of
  the frame. The launcher sets it; `-NoWaterLight` leaves it out (pull request #39).
- **Characters, clothes and props are there**: on a 4 GB card the tree admitted about 5,500
  textures and then no more; at a quarter of their size about 10,000 fit in the same memory
  (issue #3).
- **The fog veil over interiors is gone** in three runs of three (issue #2, pull request #15).

To start it with all of that: `scripts\play-4gb.ps1 -Exe <your game executable>` on `coolfox-4gb`
(`-Quality low|medium|high`, `-TextureScale 1|0.5|0.25`, `-NoWaterLight`, `-Show` to see what it
would set).

How the numbers are taken now: two runs driving the same route end in different places (3,700 to
5,100 draws a frame) and differ by more than most changes do, so a change is compared with the boat
standing still at the start of free play, which repeats within about 1 ms a frame. A batch of CPU-side
ports that looked like 8.0 against 7.2 fps driving measured 112.3 against 112.0 ms that way: no gain
(pull request #38).

**Not good yet**

- Speed: the card is the limit. It works about 77 ms of a 112 ms frame, spread over more than a
  thousand kinds of small commands (no pass over 3 ms); cutting 10 ms of CPU time did not shorten
  the frame. The card's memory is about a tenth over its budget and the game holds 15.6 of the
  PC's 24 GB.
- The first scene while it loads: full-white captures and untextured stand-in characters in some
  runs. Fast cuts in cutscenes outrun the texture upload.
- Water differed between identical runs before the change above: waves, a flat mirror, or white
  all over (issue #21). Not looked at again since; underwater is not checked.
- Interiors and night are clean but dark: indirect light is still missing.
- Two compute programs still do not translate for 32-lane waves (`02384a60fda6f2d0`,
  `6b82bc7d22016cb5`).

It is not finished, and pull requests are what would move it fastest: see
[Contributing](#contributing). If you have a 4 to 8 GB card, a run of yours with the numbers the
log prints is a contribution: the form is under Issues.

## The two branches

| Branch | What it is | In the first gameplay scene, on this card | Picture |
|---|---|---|---|
| `main` | [fxceless/victory-fork](https://github.com/fxceless/victory-fork) `main` + pull requests and ports listed below | **5.4 to 6.7 fps** (145 to 185 ms a frame at about 4,400 draws) | clothes lit by the added ambient light, window glass right; skin and rooms still too dark |
| `coolfox-4gb` | codec-d's `cool_fox` at `f4b48ad` (the leading public build on PortWatch until it was taken down) + the render-scale port | **about 7 fps** driving in open-world free play, 8.7 to 9 standing still (112 to 115 ms a frame), with the launcher's set, 5.2 to 5.5 in the first room before the last gains; 2.4 to 2.5 fps (400 to 418 ms) with its own NVIDIA launcher's set | the best here: face, neck, hair, clothes and walls lit; with the ray-tracing passes skipped the room is lit warm and the haze is gone (one run); daylight outdoors right |

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
VICTORY_INDIRECT_BATCH=2 VICTORY_NAN_SCRUB=1 VICTORY_TEXTURE_DROP_MIPS=2 VICTORY_BAKE_SOURCES=full
VICTORY_UPSCALER=6 VICTORY_RENDER_SCALE=0.5 VICTORY_AMBIENT_SETTLE=250,3 VICTORY_AMBIENT_MARCH=3
```

(with `VICTORY_SKIP_PIPELINES` set to the lists in `scripts/rt_off_hashes.txt` and
`scripts/rt_work_off_hashes.txt` of `coolfox-4gb`; `scripts/play-4gb.ps1` there sets all of it.)

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
- **Skin and late textures.** Some runs show a character's skin pure black, flat untextured
  figures, a face of uninitialised pixels or flat water; others, same settings, show none of it. A
  target dump of one black-skin frame has skin albedo 0 in the G-buffer (lips, eyes, hair and
  clothes are there): the texture had no host copy on a full 4 GB card. `VICTORY_BAKE_SOURCES=full`
  (UwUCammie's fork pull request #1264, ported here) takes bake draws sampling a missing texture
  from 104 of 107 to 0 of 111, which is necessary and not enough.
  The rule behind it: on a 4 GB card the tree lets title textures fill a quarter of the budget
  (about 840 MB) and then never admits more, with 6 to 7.5 GB of them waiting and no eviction.
  `VICTORY_TEXTURE_DROP_MIPS=2` puts about twice the textures in the same bytes: in two runs of
  two the boat cutscene has every face and a patterned shirt that had been black in about twenty
  runs before. The loading of the first scene is still wrong (white captures, untextured
  stand-ins). Issue #3 has the numbers and the code changes that would fix it properly.
- **Interiors and night.** The green-brown fog veil over interiors is located and, in three runs
  of three, gone with `VICTORY_AMBIENT_SETTLE=250,3 VICTORY_AMBIENT_MARCH=3` (pull request #15,
  the measurements in issue #2). It travelled the title's ray-marched in-scatter, which nothing
  bounded, and the stand-in's mean of the room lagged seconds behind every cut at 4 to 6 fps, so
  identical runs came out veiled (about 7 in 10) or clean. Ray-tracing switches had nothing to do
  with it: six runs of them are in issue #2. Still open: full-white captures while the first scene
  loads (fewer, not gone), and interiors are clean but dark, since indirect light is still missing.
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
