#!/bin/sh

./build-image-base.sh

docker build -t botloader/bl-webapi -f ../cmd/webapi/Dockerfile ../