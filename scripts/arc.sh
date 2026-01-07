#!/bin/zsh

arc() {
  local response
  response=$(CLICOLOR_FORCE=1 command arc "$@")
  eval "$response"
  echo "Exporting the following to the shell:"
  echo $response
}
