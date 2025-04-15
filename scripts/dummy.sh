#!/bin/bash

echo "Arg#1: $1"

shift
echo "The rest:"
for arg in "$@"; do
  echo "$arg"
done
