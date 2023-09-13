#!/bin/bash -e

if [ "$(uname)" == "Darwin" ]; then
    export HOME="/Users/distiller"
    export HOME=`pwd`
fi


# copy rust binary
mkdir -p $PREFIX/bin
cp $RECIPE_DIR/dist/blobtk $PREFIX/bin/blobtk
chmod 755 $PREFIX/bin/blobtk


# install python library from wheel
PY_TAG=cp${PY_VER/./}
WHEEL=$RECIPE_DIR/dist/blobtk-*-$PY_TAG-*.whl
#if [[ "$OSTYPE" == "linux"* ]]; then
#    WHEEL=$RECIPE_DIR/dist/blobtk-*-$PY_TAG-*.manylinux2014_x86_64.whl
#elif [[ "$OSTYPE" == "darwin"* ]]; then
#    WHEEL=$RECIPE_DIR/dist/blobtk-*-$PY_TAG-*_universal2.whl
#fi
$PYTHON -m pip install $WHEEL

