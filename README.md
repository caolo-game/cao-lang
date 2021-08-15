# Cao-Lang

[![Cao-Lang core tests](https://github.com/caolo-game/cao-lang/actions/workflows/cao-lang-tests.yml/badge.svg)](https://github.com/caolo-game/cao-lang/actions/workflows/cao-lang-tests.yml)
[![C bindings tests](https://github.com/caolo-game/cao-lang/actions/workflows/c.yml/badge.svg)](https://github.com/caolo-game/cao-lang/actions/workflows/c.yml)
[![Python tests](https://github.com/caolo-game/cao-lang/actions/workflows/python-test.yml/badge.svg)](https://github.com/caolo-game/cao-lang/actions/workflows/python-test.yml)
[![WASM tests](https://github.com/caolo-game/cao-lang/actions/workflows/wasm-tests.yml/badge.svg)](https://github.com/caolo-game/cao-lang/actions/workflows/wasm-tests.yml)

The node based "language" that governs the actors in the game CaoLo

[Core documentation](https://docs.rs/cao_lang/)

[WASM package documentation](https://caolo-game.github.io/cao-lang/index.html)

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
