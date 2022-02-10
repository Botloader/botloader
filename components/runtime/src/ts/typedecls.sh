#!/bin/bash
set -e

# cd to the script location
cd "${0%/*}"

if [ -d "./typings" ]; then 
    rm -r typings
fi

tsc --build tsconfig.json

# mkdir typings
# mv *.d.ts typings

cp -r globals typings