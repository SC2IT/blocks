# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- The game is now Bitrot. The name shows on the menu, the board and the
  window, in piece colors, and now and then a letter or two glitches with a
  magenta and cyan split. Low Sensory holds the T and the O mid glitch
  instead. Settings and high scores are saved in their own Bitrot folder,
  copied from Blocks' the first time Bitrot runs, so the two can be played
  side by side. The program file is still `blocks`.
- Low Sensory option on the Options page. It holds the background still in
  full color, with no line clear surge or level up rush; the level still sets
  the storm, which keeps one lightning bolt lit. Glitch pieces hold their color
  split without bursting, tough blocks' static holds, and the board no longer
  shakes. Music and sound play at no more than half volume, and their
  sliders still go lower; the top half of each slider is crossed out. Turning
  it off brings back the saved volumes. While it's on, the home screen says
  "Low Sensory On" and how to turn it off. Saved as `low_sensory` in
  `settings.toml`.
- Four modes: Normal, Medium, Hard and Extreme. Extreme plays like the old
  Hard mode, speeding up to instant drops by level 19 with no hold or ghost
  piece. Normal, Medium and Hard keep hold and the ghost piece and stop
  speeding up at level 15.
- Glitch pieces: now and then an odd four-block piece turns up, its blocks
  joined only at the corners or broken by gaps. Glitch pieces flicker in
  magenta and cyan and burst with jumps, color flashes and tears. Drip, an
  I with its end dropped a row, turns up in every mode; Medium adds Split,
  Kink and Hook, and Hard and Extreme add Antlers, Diamond and Corners.
  About 1 in 40 pieces is a glitch at level 1, rising to 1 in 12 by level
  15, never two in a row, and the regular pieces still come in fair sets
  of seven.
- Tough pieces: now and then a regular piece arrives tough, in every mode.
  When a line clears, its tough blocks survive, cracked, and the blocks
  above them drop onto them; a cracked block goes with the next clear
  through it. A line left full after a clear, such as one that was all
  tough, clears straight away and scores again, so cracked blocks never
  fill a line. About 1 in 20 pieces is tough at level 1, rising to 1 in 8
  by level 15. Glitch pieces are never tough, and a held piece stays tough.
  Tough pieces show the background through them, with no outline: the star
  flight under a steady glow of the piece's color, the plasma in a grid,
  the storm's rain, sea and lightning with icicles below, or calm static,
  speeding up and building with the level like the background. A cracked
  block is the piece's color split open by a jagged crack and a chipped
  corner, with the background's scene glowing through: starlight, pulsing
  plasma, ice, or magenta and cyan static.
- Glitch and tough pieces drop their own way. A glitch piece's ghost is
  outlined in magenta and cyan, jumping apart in bursts, and its soft and
  hard drop trails split and flicker, landing with a split flash and a
  spray of magenta and cyan pixels. A tough piece's ghost has a heavy
  outline, and it drops with stronger trails, landing with a brighter
  flash, more and heavier dust and a bigger bump.
- `glitch` screenshot scene, with `GLITCH_TIME` to pick the moment.
- `tough` screenshot scene, tough pieces beside cracked blocks, with
  `BACKGROUND` to pick the background.
- Animation viewer (F12 from the menu): G picks the test piece for the
  falling animations, any of the seven pieces or any glitch shape; a Glitch Pieces entry shows
  all seven shapes at once; and N starts a game where every piece is a
  glitch piece. T makes the test piece tough, a Tough Pieces entry shows all seven pieces
  tough with a row of cracked blocks, a Tough Clear entry shows
  tough blocks cracking and cracked ones going, and Shift N starts a game
  where every regular piece is tough.

- Storm level up: a white flash, a barrage of six lightning bolts sweeping
  across the sea, and the sea heaving up into the new level's state.
- Storm background: rain, wind and a three-layer sea under rolling clouds.
  The storm builds with the level, reaching full strength at level 12, and
  a whiteout of rain and mist creeps in from level 10.
- Five kinds of lightning: forked, double, sheet, crawling and distant, with
  a glow along each bolt. Line clears light up the clouds.
- Storms, a dark 3/4 waltz music theme, with a loop option and a length
  check in `tools/make_chiptune.py`.
- Audio option to keep the music playing when the window loses focus.
- Shanty, a stormy 6/8 sea shanty music theme in D minor with a folk-dance
  bridge, and a jig drum pattern in `tools/make_chiptune.py`.
- `storm`, `storm-max`, `storm-whiteout` and `options-audio` screenshot scenes.

### Changed

- Song, Music and Sound moved to a new Audio page in the options.
- Standard mode is now Normal and Hard mode is now Extreme. The game starts
  on Normal until a mode is picked; the old saved mode isn't carried over.

## [0.5.1] - 2026-11-15

### Changed

- Smaller release binary: fat LTO, one codegen unit and size optimization.
- Sound effects are Ogg Vorbis instead of WAV.

### Removed

- A music track.
- Gamepad support from ggez, which the game never used.

## [0.5.0] - 2026-11-15

### Added

- Standard and Hard modes. Standard stops speeding up at level 15; Hard
  keeps going to instant drops at level 19.
- Points for soft drops (1 per row) and hard drops (2 per row).
- Level progress bar under the playfield that fills as you clear lines and
  bursts on level up.
- Top 10 high score table with arcade-style initials entry.
- Options menu: mode, song, background, block style, board style, music
  volume and sound volume. Options and window state are saved between runs.
- Five music tracks: Dance, Bounce, Baroque, Nocturne and Original. Music
  speeds up with the level without changing pitch.
- Sound effects for moving, rotating, locking, hard drops, holding, line
  clears, quads, level ups, new high scores and menu navigation, plus a game
  over jingle.
- Backgrounds (stars, plasma, off) that react to line clears and level ups
  and freeze on game over.
- Block styles (bevel, flat, neon, retro, jelly) and board styles (grid,
  dots, checker, clean).
- Effects: drop trails, landing impacts and dust, screen flash and shake,
  line clear callouts and a quad glint.
- Game over animation: rows gray out from the bottom up, then the overlay
  drops in.
- Selectable main and pause menus, and a high scores screen.
- Resizable window with pixel-snapped scaling.
- Release packages built in CI when a `v*` tag is pushed: Linux AppImage
  and tarball, Windows zip, and universal macOS `.dmg`.
- Packaging scripts in `tools/` for building each package locally.
- `THIRD_PARTY_LICENSES.md`, generated with cargo-about and included in
  every release package.
- cargo-deny checks for licenses, sources and advisories in CI.
- Debug animation viewer and screenshot scenes.

### Changed

- Renamed the project to Blocks.
- Switched the game engine from macroquad to ggez.
- Simplified controls and smoother held-key handling.
- New piece color palette and app icon to match.
- Redrawn playfield and in-game UI.
- On macOS, resources are also loaded from the app bundle.
- CI uses `Swatinem/rust-cache` and `actions/checkout@v4`.
- Updated dependencies and pinned exact versions.

### Fixed

- Music volume not applying correctly.

## [0.3.2] - 2023-01-31

### Changed

- Builds on the stable toolchain.
- The Windows console window is only hidden in release builds.

### Fixed

- Panic from an out-of-bounds index when a piece reached the top of the
  playfield.

## [0.3.1] - 2023-01-27

### Fixed

- A piece could lock in mid-air during lockdown.
- Clearing a line left stale state on locked blocks at the very top of the
  playfield.

## [0.3.0] - 2023-01-26

### Added

- Wall kicks.

## [0.2.2] - 2023-01-23

### Added

- Lockdown delay.
- Game over when a piece locks entirely above the playfield.

### Changed

- Smoother difficulty ramp-up.
- Games start at level 0.
- Resized the icon to 256x256.
- The console window is hidden on Windows.

### Fixed

- Lines above row 16 did not clear correctly.
- Restarting a game began at level 1.
- Several line clearing and piece state bugs.

## [0.2.1] - 2023-01-08

### Fixed

- Unpausing the game.

## [0.2.0] - 2023-01-08

### Added

- First release: playfield, falling pieces from a shuffled bag, line
  clears and scoring.
- Keyboard controls for moving, rotating, soft and hard dropping, and
  holding a piece, with repeat on held keys.
- Ghost piece, next piece preview, and score and level display.
- Menu and pause screen.
- Looping background music with volume controls, starting at 50%.
- On-screen help listing the controls.
- Window icon.
- CI workflow.

[Unreleased]: https://github.com/ScottCUSA/blocks/compare/v0.5.1...HEAD
[0.5.1]: https://github.com/ScottCUSA/blocks/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/ScottCUSA/blocks/compare/79e4126...v0.5.0
[0.3.2]: https://github.com/ScottCUSA/blocks/compare/v0.3.1...79e4126
[0.3.1]: https://github.com/ScottCUSA/blocks/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/ScottCUSA/blocks/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/ScottCUSA/blocks/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/ScottCUSA/blocks/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/ScottCUSA/blocks/releases/tag/v0.2.0
