# Cao-Lang

<!-- [![Coverage Status](https://coveralls.io/repos/github/caolo-game/cao-lang/badge.svg?branch=master)](https://coveralls.io/github/caolo-game/cao-lang?branch=master) -->

[![Cao-Lang core tests](https://github.com/caolo-game/cao-lang/actions/workflows/cao-lang-tests.yml/badge.svg)](https://github.com/caolo-game/cao-lang/actions/workflows/cao-lang-tests.yml)
[![C bindings tests](https://github.com/caolo-game/cao-lang/actions/workflows/c.yml/badge.svg)](https://github.com/caolo-game/cao-lang/actions/workflows/c.yml)
[![Python tests](https://github.com/caolo-game/cao-lang/actions/workflows/python-test.yml/badge.svg)](https://github.com/caolo-game/cao-lang/actions/workflows/python-test.yml)
[![WASM tests](https://github.com/caolo-game/cao-lang/actions/workflows/wasm-tests.yml/badge.svg)](https://github.com/caolo-game/cao-lang/actions/workflows/wasm-tests.yml)

The node based "language" that governs the actors in the game CaoLo

[WASM package documentation](https://caolo-game.github.io/cao-lang/index.html)


## Project layout

```
 |+ cao-lang/           # core library
 |+ c/                  # C interface
 |+ py/                 # Python interface
 |+ wasm/               # WASM interface
 |  CMakeLists.txt      # Root cmake file
 |  MANIFEST.in         # Python build dependency
 |  pyproject.toml      # Python build dependency
 |  README.md
 |  setup.py            # Python build dependency
 |  tox.ini             # Python testing dependency

```
