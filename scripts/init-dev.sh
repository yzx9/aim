#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
#
# SPDX-License-Identifier: Apache-2.0

set -euo pipefail

if [ -d .dev ]; then
  read -r -p ".dev already exists and will be deleted before re-initializing. Continue? [y/N] " confirm
  case "$confirm" in
  [yY] | [yY][eE][sS]) ;;
  *)
    echo "Initialization cancelled"
    exit 0
    ;;
  esac
  rm -rf .dev
fi

mkdir -p .dev/calendar
cp examples/*.ics .dev/calendar/
touch .dev/calendar/.dev-marker
echo "Copied $(ls examples/*.ics 2>/dev/null | wc -l) example files to .dev/calendar/"
echo "Dev database will be initialized on first 'aim' run"
