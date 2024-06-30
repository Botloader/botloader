# Botloader

https://botloader.io | https://discord.gg/GhUeYeekdu

Botloader is a fully programmable discord bot where you can use plugins created by the author and community or create custom scripts for custom functionality.

Scripts are created in typescript using the online editor with full autocomplete support or using the vs code extension.

## Project structure

This project is divided into several services and components:

### Backend

The backend consists of several pieces:

- webapi
  - The botloader HTTP API that the frontend uses
- dbmigrations
  - Applies database migrations found in `components/stores`
- DiscordBroker
  - Manages the gateway connection with discord, receives events and either queues them up or forwards them to the scheduler is connected
  - Also responsible for managing the state cache, exposing a API to query cached information
- Scheduler
  - Manages a guild's js vm worker state, scheduling the vm workers between the guilds when needed
  - Takes in the events from the broker and forwards them to a vm worker
  - Handles all the timers and scheduled tasks and dispatches them to a vm worker
- vmworker
  - A single instance of a js v8 VM,
  - Each vm runs in a dedicated process due to v8 being somewhat funky
  - Originally I kept all vms in the same process but multiple threads but there's several issues related to this such as v8 having hardcoded process exit calls in certain siutations
  - Takes in events from the scheduler to then run code in v8 on
- jobs
  - Runs various background jobs at intervals
- runtime-models
  - Contains models used primarily internally in the layer between the js SDK and the native op calls
  - We use rs-ts to auto-generate typescript typings for these models which reduces the possibility of bugs in this layer, leading to compile-time errors if we forget to change a field somewhere when we make changes.

### Frontend

The frontend bits are located in the `frontend` folder together with the `frontend-common` folder which is also used with the vs code plugin.

### Integration tests suite

In the `integration-tests` folder you will see a bunch of scripts.

In short, it runs a set of scripts, and the scripts test various functionality and report when complete.

In more detail:
- It runs `cmd/prepare-integration-tests` to load them all into a test database as guild scripts
- It runs the backend discordbroker and scheduler in integration test mode
- The integration test mode attaches a log subscriber that listens for when scripts self-report that they finish and also exceptions and errors
- Once all scripts have self-reported their finish or a certain amount of time has passed without anything happening, it stops the backend with either a failure or success depending on the former.
