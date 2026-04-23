#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
#
# SPDX-License-Identifier: Apache-2.0

set -euo pipefail

if [ $# -ne 1 ]; then
  echo "Usage: $0 <branch-name>" >&2
  exit 1
fi

name="$1"
path="${name//\//__}"
root="$(dirname "$(git rev-parse --git-common-dir)")"
worktree="$root/.worktree/$path"

git worktree add -b "$name" "$worktree"

# Activate direnv for the new worktree if direnv is installed and current directory is allowed
# `foundRC.allowed`: 0 -> allowed, 2 -> denied
if command -v direnv >/dev/null 2>&1; then
  if direnv status --json |
    jq -e --arg p "$(pwd -P)/.envrc" '.state.foundRC.path == $p and .state.foundRC.allowed == 0' >/dev/null; then
    echo "==> current direnv is allowed, propagating to worktree"
    direnv allow "$worktree"
  else
    echo "==> current direnv not allowed (or no .envrc), skip"
  fi
fi

# Initialize development calendar in the new worktree
cd "$worktree" && just init-dev
