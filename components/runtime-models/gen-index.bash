# This script goes over already generated ts types (using cargo t)
# and creates a index.ts file for each folder with all of the ts files
# in that folder exported from it

#!/bin/bash
set -e

# cd to the script location
cd "${0%/*}"

gen_index(){
    echo "handling $1"
    echo "// generated index file using gen-index.bash" > "${1}/index.ts"

    FILES="$1/*"
    for f in $FILES
    do
        # file without the path and just name
        FILE_NAME_ONLY=${f#"$1/"}

        if [ "${FILE_NAME_ONLY}" = "index.ts" ]; then 
            echo "skipping index..."
        elif [ -d "${f}" ]; then
            echo "$f is a dir"
            # echo "export * from './${FILE_NAME_ONLY}/index'" >> "${1}/index.ts"
            gen_index $f
        else
            # strip the .ts suffix
            FILE_STRIPPED=${FILE_NAME_ONLY%".ts"}
            echo "Processing $FILE_STRIPPED file..."
            echo "export * from './${FILE_STRIPPED}'" >> "${1}/index.ts"
        fi
    done
}

gen_index "bindings"