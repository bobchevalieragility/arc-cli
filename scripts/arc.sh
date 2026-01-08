#!/bin/zsh

arc() {
  local response
  response=$(CLICOLOR_FORCE=1 command arc "$@")
  eval "$response"
}
