#/bin/sh

# Keep in mind you still have to build ../frontend-common on changes to it

cd "${0%/*}"

export REACT_APP_BOTLOADER_API_BASE="http://api.botloader.io"
export REACT_APP_BOTLOADER_CLIENT_ID="852626272794181692"

npm start