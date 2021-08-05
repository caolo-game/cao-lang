#!/bin/bash
set -ex

for PYBIN in /opt/python/cp{36,37,38,39}*/bin; do
    "${PYBIN}/pip" install -U auditwheel build
    "${PYBIN}/python" build --wheel
done

for whl in dist/*.whl; do
    auditwheel repair "$whl" -w dist/
done
