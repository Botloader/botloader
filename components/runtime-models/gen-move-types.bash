#!/bin/bash
set -e

# cd to the script location
cd "${0%/*}"

# generate the types
cargo t

# generate indexes
./gen-index.bash

# move bindings
if [ -d "../runtime/src/ts/generated" ]; then 
    rm -r ../runtime/src/ts/generated
fi
mv bindings/ ../runtime/src/ts/generated   