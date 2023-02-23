#!/bin/bash -e

if [ "$(uname)" == "Darwin" ]; then
    export HOME="/Users/distiller"
    export HOME=`pwd`
fi



# build binary with Rust
C_INCLUDE_PATH=$PREFIX/include OPENSSL_DIR=$PREFIX LIBRARY_PATH=$PREFIX/lib cargo install --path ./rust --root $PREFIX


# install python library from wheel
PY_TAG=cp${PY_VER/./}
if [[ "$OSTYPE" == "linux"* ]]; then
    $PYTHON -m pip install --no-deps $RECIPE_DIR/dist/blobtk-*-$PY_TAG-*.manylinux2014_x86_64.whl
elif [[ "$OSTYPE" == "darwin"* ]]; then
    $PYTHON -m pip install --no-deps $RECIPE_DIR/dist/blobtk-*-$PY_TAG-*_universal2.whl
fi
