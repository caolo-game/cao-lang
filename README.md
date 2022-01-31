# Cao-Lang

![GitHub Workflow Status](https://img.shields.io/github/workflow/status/caolo-game/cao-lang/Cao-Lang%20tests)
![MIT license](https://img.shields.io/github/license/caolo-game/cao-lang)
![Crates.io version](https://img.shields.io/crates/v/cao-lang)
![PyPI version](https://img.shields.io/pypi/v/cao-lang)
![npm version](https://img.shields.io/npm/v/@caolo-game/cao-lang-wasm)

The node based "language" that governs the actors in the game CaoLo

[Core documentation](https://docs.rs/cao_lang/)

## Project layout

```
 |+ cao-lang/           # core library
 |+ c/                  # C interface
 |+ py/                 # Python interface
 |+ wasm/               # WASM interface
 |+ xtask/              # Additional scripts via Cargo
 |  CHANGELOG.md        # Changelog
 |  CMakeLists.txt      # Root cmake file
 |  MANIFEST.in         # Python build dependency
 |  pyproject.toml      # Python build dependency
 |  README.md
 |  setup.py            # Python build dependency
 |  tox.ini             # Python testing dependency
 |  cliff.toml          # git-cliff config
```
