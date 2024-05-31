#!/usr/bin/env bash

set -euo pipefail

WGET=wget
if command -v wget2 &> /dev/null; then WGET=wget2; fi

$WGET 'https://github.com/github-linguist/linguist/raw/HEAD/lib/linguist/languages.yml' -O - | yq -o json > languages.json
