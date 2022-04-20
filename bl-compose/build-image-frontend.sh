#!/bin/sh

./build-image-base.sh

echo "Using client id: $DISCORD_CLIENT_ID"
docker build --build-arg BOTLOADER_API_BASE="http://localhost:7978" --build-arg BOTLOADER_CLIENT_ID="$DISCORD_CLIENT_ID" -t botloader/bl-frontend -f ../frontend/Dockerfile ../