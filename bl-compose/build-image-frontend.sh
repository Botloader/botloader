#!/bin/sh

if [[ -z "$BOTLOADER_API_BASE" ]]; then
	BOTLOADER_API_BASE="http://localhost:7447"
fi

echo "Using client id: $DISCORD_CLIENT_ID"
docker build --build-arg BOTLOADER_API_BASE="$BOTLOADER_API_BASE" --build-arg BOTLOADER_CLIENT_ID="$DISCORD_CLIENT_ID" -t botloader/bl-frontend -f ../frontend/Dockerfile ../
