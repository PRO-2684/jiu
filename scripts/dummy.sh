#!/bin/bash

echo "TERM = $1"

shift
echo "Arguments:"
for arg in "$@"; do
  echo "$arg"
done
