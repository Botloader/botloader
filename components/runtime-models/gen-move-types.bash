#!/bin/bash
set -e

# cd to the script location
cd "${0%/*}"

# generate the types
cargo t

# generate indexes
./gen-index.bash

# move bindings
if [ -d "../runtime/src/ts/models" ]; then 
    rm -r ../runtime/src/ts/models
fi
mv bindings/ ../runtime/src/ts/generated   