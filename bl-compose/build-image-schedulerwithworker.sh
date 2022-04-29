#!/bin/sh

./build-image-base.sh

docker build -t botloader/bl-schedulerwithworker -f ../cmd/schedulerwithworker/Dockerfile ../