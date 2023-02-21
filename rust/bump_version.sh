#!/bin/bash

if [ -z "$1" ]; then
  echo "USAGE: ./bump_version.sh major|minor|patch"
  exit 1
fi

if output=$(git status --porcelain) && [ ! -z "$output" ]; then
  # Uncommitted changes
  echo "Unable to bump version. Git working directory is not clean."
  exit 1
fi

cargo bump $1 &&

cd .. &&

bump2version --allow-dirty $1 &&

cd -

