#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p static/img static/fonts
UA="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36"

dl() { curl -fsSL -A "$UA" "$1" -o "$2" && echo "ok  $2" || echo "MISS $1 (using fallback)"; }

dl "https://assets.juspay.in/hyper/assets/in.juspay.merchants/images/testbalaji/jp_barclaycardlogoheaderbig_20260724155833.png" static/img/barclaycard-logo.png
dl "https://assets.juspay.in/hyper/images/common/jp_secured.png" static/img/secured.png
dl "https://assets.juspay.in/hyper/assets/in.juspay.merchants/fonts/testbalaji/BarclaysEffra/BarclaysEffra-Regular.ttf" static/fonts/BarclaysEffra-Regular.ttf
dl "https://assets.juspay.in/hyper/assets/in.juspay.merchants/fonts/testbalaji/BarclaysEffra/BarclaysEffra-SemiBold.ttf" static/fonts/BarclaysEffra-SemiBold.ttf
dl "https://assets.juspay.in/hyper/assets/in.juspay.merchants/fonts/testbalaji/BarclaysEffra/BarclaysEffra-Bold.ttf" static/fonts/BarclaysEffra-Bold.ttf
echo "done"
