# vmbench

Benchmarks guild vm creation, in two modes.

## `vm` mode: in-process vm creation

Uses the same code path as the vm worker (`vm::vmthread::spawn_vm_thread` with
the full runtime extensions and module map), measured from just before the vm
thread is spawned until the vm reports `VmFinished`: isolate created from the
snapshot, extensions initialized, and all provided scripts compiled
(tscompiler) and ran to completion.

No database, discord broker, or discord connection is needed — the runtime
context is stubbed and only touched if a script actually calls storage/discord
APIs (which bench scripts shouldn't do).

## `full` mode: the whole scheduler flow

Benchmarks the flow as it stands in production, with real vm worker child
processes:

```
event in -> guild activation (premium tier + scripts loaded from db)
         -> worker claimed from the pool -> create vm rpc
         -> vm created, scripts ran -> event dispatched -> acked by the vm
```

This drives the same code as `GuildHandler`/`VmSession` (via
`scheduler::bench`), dispatching a synthetic `MESSAGE_CREATE` event and
measuring until the vm acks it. Between iterations the vm is torn down so
every run is a cold start.

Requirements (same environment as the integration tests, e.g. `source .env`):

- `DATABASE_URL` — the provided scripts **replace the bench guild's scripts in
  this database** (default bench guild id: `999000000000000001`, override with
  `--guild-id`/`VMBENCH_GUILD_ID`)
- `DISCORD_BOT_TOKEN`, `DISCORD_CLIENT_ID`, `DISCORD_CLIENT_SECRET` — the vm
  worker processes fetch the discord config on startup
- don't run it while a real scheduler is running on the same machine, they
  share the worker socket (`/tmp/botloader_scheduler_workers`)

## Usage

The `run.sh` wrapper runs one of the sample workloads from `scripts/`:

```sh
./run.sh light                 # vm mode (default), 1 small script
./run.sh medium                # vm mode, 5 medium sized scripts
./run.sh heavy                 # vm mode, 15 large scripts
./run.sh all                   # vm mode, all three variants

./run.sh full light            # full scheduler flow
./run.sh full all              # full scheduler flow, all three variants

# extra args are forwarded to vmbench
./run.sh medium --iterations 50 --quiet
./run.sh full medium --iterations 20
```

Or invoke vmbench directly with your own scripts:

```sh
# in-process, empty vm (isolate + snapshot + extensions only)
cargo run --release -p vmbench -- vm

# in-process with scripts
cargo run --release -p vmbench -- vm path/to/script.ts --iterations 50

# full flow (needs the env described above, at least one script required)
cargo run --release -p vmbench -- full path/to/script.ts --workers 1
```

Script errors and `console.log` output are printed to stderr (disable with
`--quiet`) so silently-failing scripts don't skew results.

## Sample workloads

`scripts/` contains three sample workloads with realistic guild scripts
(commands, event handlers, interval timers, storage buckets):

- `light`: 1 small script
- `medium`: 5 medium sized scripts (welcome, moderation, leveling, tags, polls)
- `heavy`: 15 large scripts (economy, automod, tickets, starboard, giveaways,
  reaction roles + `_2`/`_3` variants of most of them)

When adding scripts, note that all scripts in a variant load into the same vm,
so slash command names and interval timer names must be unique across the
whole variant — duplicates make script loading fail (which the stderr log
will show).
