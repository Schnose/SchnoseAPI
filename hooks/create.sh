#!/usr/bin/env bash

HOOKS_DIR="$(git rev-parse --git-path hooks)"

# pre-push
cat "./pre-push" > "$HOOKS_DIR/pre-push"
chmod +x "$HOOKS_DIR/pre-push"
