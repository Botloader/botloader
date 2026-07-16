#!/bin/bash
# Regenerates frontend-common/src/generated/api.ts from frontend-common/openapi.json.
# Pass --update-spec to first refresh openapi.json from a running webapi
# (BL_API_BASE, default http://localhost:7447).
set -e
cd "${0%/*}/.."

if [ "$1" == "--update-spec" ]; then
    curl -sf "${BL_API_BASE:-http://localhost:7447}/api/openapi.json" | python3 -m json.tool --indent 2 > frontend-common/openapi.json
fi

node generate-webapi/index.ts
