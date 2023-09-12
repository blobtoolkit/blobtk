#!/bin/bash -e

if [ "$(uname)" == "Darwin" ]; then
    export HOME="/Users/distiller"
    export HOME=`pwd`
    # copy rust binary
    mkdir -p $PREFIX/bin
    cp $RECIPE_DIR/dist/blobtk $PREFIX/bin/blobtk
    chmod 755 $PREFIX/bin/blobtk
else
  # build binary with Rust
  C_INCLUDE_PATH=$PREFIX/include OPENSSL_DIR=$PREFIX LIBRARY_PATH=$PREFIX/lib cargo install --path ./rust --root $PREFIX
fi



# build binary with Rust
C_INCLUDE_PATH=$PREFIX/include OPENSSL_DIR=$PREFIX LIBRARY_PATH=$PREFIX/lib cargo install --path ./rust --root $PREFIX

# copy rust binary
# mkdir -p $PREFIX/bin
# cp $RECIPE_DIR/dist/blobtk $PREFIX/bin/blobtk
# chmod 755 $PREFIX/bin/blobtk


# install python library from wheel
PY_TAG=cp${PY_VER/./}
if [[ "$OSTYPE" == "linux"* ]]; then
    WHEEL=$RECIPE_DIR/dist/blobtk-*-$PY_TAG-*.manylinux2014_x86_64.whl
elif [[ "$OSTYPE" == "darwin"* ]]; then
    WHEEL=$RECIPE_DIR/dist/blobtk-*-$PY_TAG-*_universal2.whl
fi
$PYTHON -m pip install $WHEEL

