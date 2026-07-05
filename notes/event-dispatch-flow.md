# Event dispatch flow: broker → scheduler → worker → v8

Written 2026-07-04 while investigating why the vmbench full-flow numbers grow
roughly linearly with the number and size of a guild's scripts. File references
are against the current working tree.

The flow has two very different regimes:

- **Warm path** — the guild's VM is alive on a worker and the same worker is
  re-claimed: an event dispatch is a few channel sends, one JSON hop over a
  unix socket and a v8 function call. Nothing here scales with script size.
- **Cold path** — a VM has to be (re)created: every enabled script's source is
  cloned, JSON-encoded over the socket, SWC-compiled, evaluated in a fresh
  isolate, and bookended by two DB writes per script. Every step is linear in
  script count × size, and this is the path the bench (and any guild whose
  worker got stolen) pays.

---

## 1. Broker → scheduler process

1. The broker pushes `BrokerEvent::DiscordEvent` over a TCP connection using
   `simpleproto` (u32 length prefix + JSON body) —
   `components/scheduler/src/broker_client.rs:47`.
2. `BrokerConn::handle_broker_message` forwards it as
   `SchedulerCommand::DiscordEvent` onto the scheduler's unbounded mpsc channel
   and acks the broker (`broker_client.rs:73-88`). Note the broker connection
   is ack-per-message, so scheduler-side stalls back-pressure the whole broker
   stream, but nothing on this hop depends on scripts.

## 2. Scheduler task → per-guild handler

3. The single `Scheduler` task (`components/scheduler/src/scheduler.rs:74`)
   polls guild handles and the command channel via a hand-rolled future
   (`scheduler.rs:343`). For a `DiscordEvent` it checks suspension, then
   `send_or_queue_broker_evt` (`scheduler.rs:284`).
4. `get_or_start_guild` (`scheduler.rs:300`) lazily spawns a `GuildHandler`
   tokio task for the guild if none is running, and the event is sent as
   `GuildCommand::BrokerEvent` on the handler's unbounded channel.
5. **Guild activation** (only when the guild was inactive):
   `GuildHandler::setup` (`components/scheduler/src/guild_handler.rs:105`)
   runs before any event is processed:
   - `fetch_premium_tier` — 1 DB query (`guild_handler.rs:186`).
   - `VmSession::start` → `try_retry_load_guild_scripts` — `list_scripts`
     loads **the full source of every script** from the DB, filters to enabled
     (`components/scheduler/src/vm_session.rs:277`).
   - `start_fresh_vm` (`vm_session.rs:252`): if there are enabled scripts,
     sets `force_load_scripts_next = true` and claims a worker (§3).

## 3. Claiming a worker (VmSession)

6. Every dispatch goes through `dispatch_worker_evt` → `ensure_claim_worker`
   (`vm_session.rs:520`, `:566`). If the session already holds a worker this
   is free. Otherwise `VmWorkerPool::req_worker`
   (`components/scheduler/src/vmworkerpool.rs:79`):
   - first scans pools (own tier downwards) for a worker whose
     `last_active_guild` == this guild — **same-guild hit means the VM from
     last time is still alive in that worker process and can be reused**;
   - otherwise takes the least-recently-used worker from any tier ≤ own;
   - otherwise queues and waits for a return.
7. `should_send_scripts` (`vm_session.rs:698`): a `CreateScriptsVm` round trip
   is skipped **only** if the pool returned `SameGuild` *and*
   `force_load_scripts_next` is false *and* `no_reuse_vms` is off. Any other
   outcome → full VM creation (§4).
8. `send_create_scripts_vm` (`vm_session.rs:603`) sends
   `CreateScriptsVmReq { scripts: self.scripts.clone(), .. }`:
   - `self.scripts.clone()` — clones every script's full source + settings in
     the scheduler;
   - the message crosses the unix socket as length-prefixed **JSON**
     (`components/simpleproto/src/lib.rs`), so all sources are JSON-escaped,
     written, read and parsed again on the worker. Linear in total source size
     on every cold start.

## 4. Worker process: creating the VM

9. `Worker::handle_create_scripts_vm`
   (`components/vmworker/src/lib.rs:277`):
   - if the worker still hosts another guild's VM,
     `wait_shutdown_current_vm` first drives that VM to graceful completion
     (up to a 15 s timeout in `stop_vm`) — a cost your guild pays for the
     *previous* tenant;
   - `vm::vmthread::spawn_vm_thread` (`components/vm/src/vmthread.rs:13`)
     spawns a **new OS thread** with its own current-thread tokio runtime.
     The `CreateScriptsVm` ack (`lib.rs:349`) is sent as soon as the thread
     reports the shutdown handle — i.e. *before* any script has compiled.
10. On the VM thread, `Vm::create_init` (`components/vm/src/vm.rs:93`):
    - `create_isolate` — fresh `JsRuntime`/v8 isolate from the snapshot
      (roughly constant cost);
    - `compile_scripts` (`vm.rs:316`) — **sequentially SWC-compiles every
      script** (`components/tscompiler/src/compiler.rs:12`): TS parse +
      transform + codegen + source-map generation, plus an eager
      `sourcemap::SourceMap::from_slice` parse that is only ever needed for
      error reporting. There is **no compile cache at any level** — the same
      unchanged source is recompiled on every single VM creation;
    - `run_scripts` (`vm.rs:340`) — sequentially, per script:
      `load_side_es_module_from_code` (v8 parse/compile of the emitted JS —
      again no v8 code cache) + `mod_evaluate` + drive the event loop until
      that module (including its top-level async work) settles.
11. Only after `create_init` finishes does `Vm::run` (`vm.rs:208`) start
    reading `VmCommand`s. **A dispatched event therefore queues behind the
    compile+evaluate of every script**, even though the worker already acked
    the VM creation.

## 5. Per-script feedback into the scheduler (also on the ack path)

12. As each script's `Script.run()` executes, the runtime emits
    `ScriptStarted` → `WorkerMessage::ScriptStarted` → back over the socket to
    the `VmSession`, which calls `script_loaded` (`vm_session.rs:719`):
    - `update_db_contribs` does **two awaited DB writes per script**
      (`update_script_contributes` + `update_script` for settings,
      `vm_session.rs:750`);
    - plus `interval_timers_man.script_started` (more DB work) and a command
      manager send.
13. These are handled in the same per-guild session loop that processes acks,
    strictly in message order. So the dispatch ack for the triggering event is
    processed only after ~2·N DB round trips for N scripts. With a few ms per
    round trip this term alone is linear and can dominate the measured
    end-to-end latency.

## 6. Dispatching the event itself

14. `dispatch_worker_evt` (`vm_session.rs:520`) assigns a seq id, records a
    `PendingAck`, sends `SchedulerMessage::Dispatch` over the socket (the
    event payload is `data.clone()`d per attempt).
15. Worker forwards it as `VmCommand::DispatchEvent` to the VM thread
    (`vmworker/src/lib.rs:214`).
16. `Vm::dispatch_event` (`vm.rs:394`) sends `VmEvent::DispatchedEvent(seq)`
    **immediately — before calling into JS** — which becomes
    `WorkerMessage::Ack` back to the scheduler. It then serde-v8-serializes
    the payload and calls `BotloaderCore.dispatchWrapper`. So "acked" means
    "reached the VM command loop", not "handlers ran".
17. When the js event loop fully drains, `VmEvent::VmFinished` →
    `WorkerMessage::NonePending` → if the session has no pending acks it
    **returns the worker to the pool** (`vm_session.rs:373`). The VM stays
    alive inside the worker process; whether the next event is warm or cold
    depends entirely on whether the pool hands the same worker back (§3.6).

---

## Why time grows linearly with script count/size

Cold-start cost = Σ over scripts of (DB read of source + scheduler-side clone
+ JSON encode/decode over the socket + SWC compile + sourcemap parse + v8
parse/compile + module evaluation + 2 DB writes + timer/command bookkeeping),
all strictly sequential on one thread (VM side) or one task loop (scheduler
side). Every term is linear; nothing is cached or parallel.

Ranked by likely impact:

1. **Per-script DB writes on the ack path (§5)** — 2·N awaited round trips
   through the session loop before the event ack is observed. Pure overhead
   when contributions haven't changed; nothing diffs against the stored state.
2. **SWC compile per VM creation (§4.10)** — recompiling unchanged TS on every
   cold start. A content-hash → compiled-output cache (in the worker process,
   in the scheduler, or persisted in the DB next to the source) removes almost
   all of it. The eager source-map parse belongs behind a lazy init either way.
3. **Sequential module evaluation (§4.10-11)** — v8 re-parses the emitted JS
   each time (deno_core supports v8 code caching), and the dispatch waits for
   *all* scripts to finish top-level init.
4. **Full sources over the socket as JSON (§3.8)** — memory churn + encode +
   decode proportional to total source size, every cold start. Sending only
   hashes when the worker might already have the compiled artifacts (or a
   binary format) would shrink this.
5. **How often the cold path fires at all (§3.6-7, §6.17)** — the worker is
   returned to the pool as soon as a burst of events settles; any other guild
   claiming it evicts the VM. With more active guilds than workers, most
   events pay the full cold start. The single `force_load_scripts_next` flag
   also forces a rebuild after any session shutdown even if sources didn't
   change.

The bench (`components/scheduler/src/bench.rs::run_iteration`) intentionally
tears the session down each iteration, so it measures exactly this cold path:
`vm_requested` ≈ steps 5-8 (DB reads + claim + create req sent), and
`event_acked` additionally covers steps 9-16 — thread spawn, N compiles, N
module evals, N×2 DB writes and the dispatch reaching the VM loop.

---

## Update 2026-07-04: derived state split out (fixes #1 above)

The machine-derived columns were split out of `guild_scripts` into a new
`guild_script_derived` table (migration `20260704100000`), written only by the
scheduler and keyed for freshness by `source_hash` + `settings_hash` +
`runtime_version` (`stores::config::SCRIPT_RUNTIME_VERSION`, bump to
invalidate everything). `VmSession::script_loaded` now compares hashes against
the freshness map loaded at session start and **skips the DB writes entirely
when nothing changed** — the 2·N ack-path round trips only happen on the first
boot after a script/settings change. The command manager reads contributions
via `get_enabled_script_command_contributes` without dragging sources along.
Replacing the hashes with a trigger-maintained `guild_scripts.version` column,
and a content-addressed compile cache (SWC output shared across guilds per
plugin source), are the planned follow-ups.
