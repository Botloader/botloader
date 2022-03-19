# Botloader frontend

This is the frontend for botloader, it's set up using create-react-app.

# Running it

First build frontend-common, that package holds common data structures and a API client.

`cd frontend-common && npm install && npm run build`

Then install the deps of the frontend:

`npm install`

Then you can run it using `npm start`.

# Running it using api.botloader.io

You can run the frontend without the rest of the stack by using api.botloader.io as the API server.

To do this first follow the steps in the "Running it" except the last one, instead use the `run_using_api.sh` script.