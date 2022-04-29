#!/bin/sh

./build-image-base.sh

docker build -t botloader/bl-broker -f ../cmd/discordbroker/Dockerfile ../