#!/bin/bash
set -e

# cd to the script location
cd "${0%/*}"

# build docs
cd components/runtime/docgen
npm install
npm run build
cd -

# copy docs to frontend
cp -r components/runtime/docgen/docs frontend/public/docs

# build frontend
cd frontend-common 
npm install
cd ../frontend
npm install
npm run build