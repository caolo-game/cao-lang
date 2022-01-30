#!/bin/bash
set -ex

curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y
source $HOME/.cargo/env

cd /cao-lang

# build cpython wheels
for PYBIN in /opt/python/cp*/bin; do
    "${PYBIN}/pip" install -U auditwheel setuptools_rust wheel toml
    "${PYBIN}/python" setup.py build bdist_wheel --py-limited-api=cp35
done

for whl in dist/*.whl; do
    auditwheel repair "$whl" -w dist/
    rm "$whl"
done

# build pypy wheels
for PYBIN in /opt/python/pypy*/bin; do
    "${PYBIN}/pip" install -U auditwheel setuptools_rust wheel toml
    "${PYBIN}/python" setup.py build bdist_wheel --py-limited-api=cp35
done

