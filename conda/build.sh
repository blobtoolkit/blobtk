#!/bin/bash -e

if [ "$(uname)" == "Darwin" ]; then
    export HOME="/Users/distiller"
    export HOME=`pwd`
fi



# build binary with Rust
C_INCLUDE_PATH=$PREFIX/include OPENSSL_DIR=$PREFIX LIBRARY_PATH=$PREFIX/lib cargo install --path ./rust --root $PREFIX

# install python library from wheel (not working)
which python
python --version
# VERSION=0.2.2
# PY_VERSION=3.8
# PY_TAG=cp${PY_VERSION/./}
# pip install --no-deps https://files.pythonhosted.org/packages/d6/4c/c0fd388d63b219699e0033b1dcbd8fab092a1ed076a46a6e30ad4b8bb16c/blobtk-0.2.2-cp38-cp38-macosx_10_7_x86_64.whl
# pip install --no-deps https://files.pythonhosted.org/packages/79/4d/3ba109d950e63a18992cc59096c8dc62f96bbea87b16d92b2de6d8c60b57/blobtk-0.2.2-cp38-cp38-manylinux_2_17_x86_64.manylinux2014_x86_64.whl
# ERROR: blobtk-0.2.2-cp38-cp38-macosx_10_9_x86_64.macosx_11_0_arm64.macosx_10_9_universal2.whl is not a supported wheel on this platform.
# ERROR: blobtk-0.2.2-cp38-cp38-macosx_10_7_x86_64.whl is not a supported wheel on this platform.
