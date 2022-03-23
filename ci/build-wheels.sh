#!/bin/bash
set -ex

curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y
source $HOME/.cargo/env

cd /cao-lang

function build_for_py {
    local pybin=$1
    # minimum 3.7 supported
    local version_test
    version_test=$(${pybin}/python -c 'import sys; print(sys.version_info.minor)' 2>/dev/null)

    if [[ $version_test -ge "7" ]]; then 
        "${pybin}/pip" install -U auditwheel setuptools_rust wheel toml
        "${pybin}/python" setup.py build bdist_wheel --py-limited-api=cp35
    fi
}

# build cpython wheels
for pybin in /opt/python/cp*/bin; do
    build_for_py $pybin
done

for whl in dist/*.whl; do
    auditwheel repair "$whl" -w dist/
    rm "$whl"
done
