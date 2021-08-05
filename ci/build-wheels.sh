#!/bin/bash
set -ex

pip install -U auditwheel build
python build --wheel

for whl in dist/*.whl; do
    auditwheel repair "$whl" -w dist/
done
