# Self hosting botloader:

Note: this is **UNSAFE** to expose to the public due to no http proxy being set
up. Users can make scripts that interacts with the internal API's. Do not expose
this, this is only for development purposes.

1. Run the `build-all-images.sh` script, this will build all the docker images.
2. Copy the `.env-sample` file as `.env` and fill in the required fields
3. Update the bot application and add a new oauth2 redirect
   `http://localhost:3000/confirm_login`
4. Run `docker compose up`
5. It should now be running, navigate to `localhost:3000` in your browser to
   view it.
