#!/bin/bash
set -ex

curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y
source $HOME/.cargo/env

pip install -U auditwheel build
python -m build --wheel

for whl in dist/*.whl; do
    auditwheel repair "$whl" -w dist/
done
