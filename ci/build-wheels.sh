#!/bin/bash
set -ex

curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y
source $HOME/.cargo/env

cd /cao-lang

ls -al /opt/python

for PYBIN in /opt/python/cp{35,36,37,38,39}*/bin; do
    "${PYBIN}/pip" install -U auditwheel build
    "${PYBIN}/python" -m build --wheel
done

for whl in dist/*.whl; do
    auditwheel repair "$whl" -w dist/
    rm "$whl"
done
