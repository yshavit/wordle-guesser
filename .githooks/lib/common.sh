#!/bin/bash

set -u
set -o pipefail

function quoted() {
  printf "$*... "
  local output
  output="$(2>&1 "$@")"
  if [[ $? -eq 0 ]]; then
    echo -e $'\e[32mOK\e[0m'
  else
    echo $'\e[31mFAILED:\e[0m'
    echo "$output" | quote_output
  fi
}

function quote_output() {
  local last_color=$'\e[0m'
  cat - | while read line ; do
    local colors="$(echo "$line" | grep -Eo $'\e\\[\\d+m' || true)"
    local quote_color=$'\e[0m'
    local quote_char='│'
    if [[ -n "$colors" ]] ; then
      last_color="$(echo "$colors" | tail -n1)"
      if [[ "$(echo "$colors" | wc -l)" -eq 1 ]]; then
        quote_color="$last_color"
      else
        quote_color=$'\e[33m'
        quote_char='┊'
      fi
    fi
    printf "%s%s %s%s\n" "$quote_color" "$quote_char" "$last_color" "$line" 
  done
}
