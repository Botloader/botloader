#!/bin/bash
set -e

# cd to the script location
cd "${0%/*}"

# generate the types
TS_RS_EXPORT_DIR="" cargo t

# Fix "[key in string]?: T" > "Record<string, T>"
find bindings -name "*.ts" -type f -exec sed -i 's/{ \[key in string\]?: \(.*\) }/Record<string, \1>/g' {} +

# merge the one-file-per-type output into a single index.ts per directory
node merge-types.mjs bindings

# move bindings
if [ -d "../runtime/src/ts/generated" ]; then
    rm -r ../runtime/src/ts/generated
fi
mv bindings/ ../runtime/src/ts/generated/