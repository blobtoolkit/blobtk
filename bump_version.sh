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

while [ $? == 0 ]; do
  sleep 2;
  git diff --exit-code --name-only Cargo.lock
done;

cd - &&

git add --all

bump2version $LEVEL --allow-dirty

NEW_VERSION=$(grep current_version .bumpversion.cfg | head -n 1 | cut -d' ' -f 3)

git commit -a -m "Bump version: ${CURRENT_VERSION} → ${NEW_VERSION}"
git tag -a $NEW_VERSION -m "Bump version: ${CURRENT_VERSION} → ${NEW_VERSION}"
