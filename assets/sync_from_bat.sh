#!/usr/bin/env bash

if [[ -d ../../bat ]]; then
  git -C ../../bat pull
else
  git clone --recurse-submodules https://github.com/sharkdp/bat ../../bat
fi

rm -rf ./syntaxes/* ./themes/*
cp -r ../../bat/assets/syntaxes/* ./syntaxes/
cp -r ../../bat/assets/themes/* ./themes/

silicon --build-cache .

echo Finished.