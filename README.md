# Cao-Lang

[![Coverage Status](https://coveralls.io/repos/github/caolo-game/cao-lang/badge.svg?branch=master)](https://coveralls.io/github/caolo-game/cao-lang?branch=master)
![Run Cao-Lang tests](https://github.com/caolo-game/cao-lang/workflows/Run%20Cao-Lang%20tests/badge.svg)
![Run WASM tests](https://github.com/caolo-game/cao-lang/workflows/Run%20WASM%20tests/badge.svg)

The abstract, node based "language" that governs the actors in the game CaoLo

[WASM package documentation](https://caolo-game.github.io/cao-lang/index.html)


## Project layout

```
 |+ cao-lang/           # core library
 |+ cli/                # command line interface
 |+ py/                 # Python interface
 |+ wasm/               # WASM interface
 |  MANIFEST.in         # Python build dependency
 |  pyproject.toml      # Python build dependency
 |  README.md
 |  setup.py            # Python build dependency
 |  tox.ini             # Python testing dependency

```
