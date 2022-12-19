#!/bin/bash

LEVEL=$1

if [ -z "$LEVEL" ]; then
  echo "Usage: ./bump_version.sh major|minor|patch"
  exit 1;
fi

if [ ! -z "$(git status --porcelain)" ]; then
  echo "Commit changes before running this script"
  exit 1;
fi

cd rust &&

cargo bump $LEVEL &&

cd - &&

git add --all

bump2version $LEVEL --allow-dirty
