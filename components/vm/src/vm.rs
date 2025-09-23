use crate::moduleloader::{ModuleEntry, ModuleManager};
use crate::{
    bl_core, AnyError, ScriptLoadState, ScriptState, ScriptStateStoreWrapper, ScriptsStateStore,
    ScriptsStateStoreHandle,
};
use chrono::Utc;
use common::dispatch_event::VmDispatchEvent;
use cpu_time::ThreadTime;
use deno_core::v8::{self, CreateParams, IsolateHandle};
use deno_core::{Extension, FastString, JsRuntime, PollEventLoopOptions, RuntimeOptions};
use futures::{future::LocalBoxFuture, FutureExt};
use guild_logger::entry::CreateLogEntry;
use guild_logger::GuildLogSender;
use metrics::histogram;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt::Debug;
use std::future::Future;
use std::pin::{pin, Pin};
use std::{
    rc::Rc,
    sync::{atomic::AtomicBool, Arc, RwLock as StdRwLock},
    task::{Context, Poll, Wake},
};
use stores::config::Script;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tracing::{error, info, instrument};

#[derive(Debug, Clone)]
pub enum VmCommand {
    DispatchEvent(VmDispatchEvent),
    LoadScript(Script),

    // note that this also reloads the runtime, shutting it down and starting it again
    // we send a message when that has been accomplished
    UnloadScripts(Vec<Script>),
    UpdateScript(Script),
    Restart(Vec<Script>),
}

#[derive(Debug)]
pub enum VmEvent {
    DispatchedEvent(u64),
    VmFinished,
}

#[derive(Serialize)]
struct ScriptDispatchData {
    name: String,
    data: serde_json::Value,
}

pub struct Vm {
    runtime: JsRuntime,

    rx: UnboundedReceiver<VmCommand>,
    tx: UnboundedSender<VmEvent>,

    script_store: ScriptsStateStoreHandle,

    shutdown_handle: VmShutdownHandle,
    guild_logger: GuildLogSender,

    extension_factory: ExtensionFactory,
    module_manager: Rc<ModuleManager>,

    wakeup_rx: UnboundedReceiver<()>,
}

impl Vm {
    #[instrument(skip_all)]
    pub(crate) fn create_with_handles(
        create_req: CreateRt,
        vm_cpu_counter: metrics::Counter,
    ) -> CreateVmSuccess {
        let (wakeup_tx, wakeup_rx) = mpsc::unbounded_channel();
        let shutdown_handle = VmShutdownHandle::new(wakeup_tx);

        let shutdown_handle_clone = shutdown_handle.clone();

        let fut = Box::pin(VmCpuTracker::wrap(
            vm_cpu_counter,
            Vm::create_run(create_req, shutdown_handle_clone, wakeup_rx),
        ));

        CreateVmSuccess {
            future: fut,
            shutdown_handle,
        }
    }

    #[instrument(skip_all)]
    async fn create_init(
        create_req: CreateRt,
        shutdown_handle: VmShutdownHandle,
        wakeup_rx: UnboundedReceiver<()>,
    ) -> Self {
        let script_store = ScriptsStateStore::new_rc();

        let module_manager = Rc::new(ModuleManager {
            module_map: create_req.extension_modules,
            guild_scripts: script_store.clone(),
        });

        let sandbox = Self::create_isolate(
            &create_req.extension_factory,
            module_manager.clone(),
            script_store.clone(),
            shutdown_handle.clone(),
        );

        let mut rt = Self {
            guild_logger: create_req.guild_logger,
            rx: create_req.rx,
            tx: create_req.tx,
            script_store,

            shutdown_handle,
            runtime: sandbox,
            extension_factory: create_req.extension_factory,
            module_manager,
            wakeup_rx,
        };

        rt.guild_logger.log(CreateLogEntry::info(
            "starting fresh guild vm...".to_string(),
        ));

        rt.emit_isolate_handle();

        rt.compile_scripts(&create_req.load_scripts);
        rt.run_scripts(&create_req.load_scripts).await;

        rt
    }

    async fn create_run(
        create_req: CreateRt,
        timeout_handle: VmShutdownHandle,
        wakeup_rx: UnboundedReceiver<()>,
    ) {
        let mut rt = Self::create_init(create_req, timeout_handle, wakeup_rx).await;
        rt.run().await;
    }

    fn create_isolate(
        extension_factory: &ExtensionFactory,
        module_manager: Rc<ModuleManager>,
        script_load_states: ScriptsStateStoreHandle,
        shutdown_handle: VmShutdownHandle,
    ) -> JsRuntime {
        let mut extensions = extension_factory();
        let cloned_load_states = script_load_states.clone();

        extensions.insert(
            0,
            bl_core::init_ops_and_esm(crate::BlCoreOptions { cloned_load_states }),
        );

        let options = RuntimeOptions {
            extensions,
            module_loader: Some(module_manager),
            get_error_class_fn: Some(&|err| {
                deno_core::error::get_custom_error_class(err).unwrap_or("Error")
            }),
            // yeah i have no idea what these values needs to be aligned to, but this seems to work so whatever
            // if it breaks when you update deno or v8 try different values until it works, if only they'd document the alignment requirements somewhere...
            create_params: Some(
                CreateParams::default()
                    .heap_limits(512 * 1024, 50 * 1024 * 1024)
                    .allow_atomics_wait(false),
            ),
            startup_snapshot: Some(crate::BOTLOADER_CORE_SNAPSHOT),
            // js_error_create_fn: Some(create_err_fn),
            source_map_getter: Some(Rc::new(ScriptStateStoreWrapper(script_load_states))),
            ..Default::default()
        };

        let another_handle = shutdown_handle.clone();

        let mut rt = JsRuntime::new(options);
        {
            let op_state = rt.op_state();
            op_state.borrow_mut().put(another_handle);
        }
        rt.add_near_heap_limit_callback(move |current, initial| {
            info!(
                "near heap limit: current: {}, initial: {}",
                current, initial
            );
            shutdown_handle.shutdown_vm(ShutdownReason::OutOfMemory, true);
            if current != initial {
                current
            } else {
                current + initial
            }
        });
        rt
    }

    fn emit_isolate_handle(&mut self) {
        let handle = self.runtime.v8_isolate().thread_safe_handle();

        let mut th = self.shutdown_handle.inner.write().unwrap();
        th.isolate_handle = Some(handle);
    }

    pub async fn run(&mut self) {
        self.emit_isolate_handle();

        info!("running runtime");
        self.guild_logger
            .log(CreateLogEntry::info("guild vm started".to_string()));

        let mut completed = false;
        while !self.check_terminated() {
            let fut = TickFuture {
                rx: &mut self.rx,
                rt: &mut self.runtime,
                wakeup: &mut self.wakeup_rx,
                completed,
            };

            completed = false;

            match fut.await {
                TickResult::Command(Some(cmd)) => {
                    self.handle_cmd(cmd).await;
                }
                TickResult::Command(None) => {
                    // sender was dropped, shut ourselves down?
                }
                TickResult::Continue => {}
                TickResult::VmError(e) => {
                    self.guild_logger.log(CreateLogEntry::error(format!(
                        "Script error occurred: {}",
                        e
                    )));
                }
                TickResult::Completed => {
                    let _ = self.tx.send(VmEvent::VmFinished);
                    completed = true;
                }
            }
        }

        info!("terminating runtime for guild");

        let shutdown_reason = { self.shutdown_handle.read_shutdown_reason() };

        if let Some(ShutdownReason::Request) = shutdown_reason {
            // cleanly finish the futures
            self.stop_vm().await;
        }
    }

    fn check_terminated(&mut self) -> bool {
        self.shutdown_handle
            .terminated
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    async fn handle_cmd(&mut self, cmd: VmCommand) {
        match cmd {
            VmCommand::Restart(new_scripts) => {
                self.restart(new_scripts).await;
            }
            VmCommand::DispatchEvent(dispatch_event) => self.dispatch_event(dispatch_event),
            VmCommand::LoadScript(script) => {
                if let Some(script) = self.compile_script(script) {
                    self.run_script(script.script.id).await
                }
            }
            VmCommand::UpdateScript(script) => {
                let mut cloned_scripts = self
                    .script_store
                    .borrow()
                    .scripts
                    .iter()
                    .map(|v| v.script.clone())
                    .collect::<Vec<_>>();

                let mut need_reset = false;
                for old in &mut cloned_scripts {
                    if old.id == script.id {
                        *old = script.clone();
                        need_reset = true;
                    }
                }

                if need_reset {
                    self.restart(cloned_scripts).await;
                }
            }
            VmCommand::UnloadScripts(scripts) => {
                let new_scripts = self
                    .script_store
                    .borrow()
                    .scripts
                    .iter()
                    .filter_map(|sc| {
                        if !scripts.iter().any(|isc| isc.id == sc.script.id) {
                            Some(sc.script.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                self.restart(new_scripts).await;
            }
        };
        // self._dump_heap_stats();
    }

    #[instrument(skip_all, fields(num_scripts=scripts.len()))]
    fn compile_scripts(&self, scripts: &Vec<Script>) {
        for script in scripts {
            self.compile_script(script.clone());
        }
    }

    #[instrument(skip_all, fields(script_len=script.original_source.len()))]
    fn compile_script(&self, script: Script) -> Option<ScriptState> {
        let mut script_store = self.script_store.borrow_mut();

        let name = script.name.clone();
        match script_store.compile_add_script(script) {
            Ok(compiled) => Some(compiled),
            Err(e) => {
                self.guild_logger.log(CreateLogEntry::error(format!(
                    "Script compilation failed for {name}.ts: {e}"
                )));
                None
            }
        }
    }

    #[instrument(skip_all, fields(num_scripts=scripts.len()))]
    async fn run_scripts(&mut self, scripts: &Vec<Script>) {
        for script in scripts {
            self.run_script(script.id).await;
        }
    }

    #[instrument(skip(self))]
    async fn run_script(&mut self, script_id: u64) {
        let (script, compiled) = {
            let borrow = self.script_store.borrow();

            if let Some(script) = borrow.get_script(script_id) {
                if script.can_run() {
                    if let Some(source) = &script.compiled {
                        (script.clone(), source.clone())
                    } else {
                        error!("script marked as can run with no compiled source",);
                        return;
                    }
                } else {
                    info!("skipping loading script");
                    return;
                }
            } else {
                error!("tried to load non-existent script");
                return;
            }
        };

        {
            self.script_store
                .borrow_mut()
                .set_state(script_id, ScriptLoadState::Loaded);
        }

        let res = self
            .runtime
            .load_side_es_module_from_code(&script.url, FastString::from(compiled.output))
            .await;

        match res {
            Ok(id) => {
                let rcv = self.runtime.mod_evaluate(id);
                self.complete_module_eval(rcv).await;
            }
            Err(err) => {
                self.log_guild_err(err);
                self.script_store
                    .borrow_mut()
                    .set_state(script_id, ScriptLoadState::Failed);
            }
        }
    }

    fn dispatch_event(&mut self, event: VmDispatchEvent) {
        let _ = self.tx.send(VmEvent::DispatchedEvent(event.seq));

        let data = ScriptDispatchData {
            data: serde_json::to_value(event.value).unwrap(),
            name: event.name.to_string(),
        };

        let global_ctx = self.runtime.main_context();
        let ctx = global_ctx.open(self.runtime.v8_isolate());

        let mut scope = self.runtime.handle_scope();
        let globals = ctx.global(&mut scope);

        let core_obj: v8::Local<v8::Object> =
            if let Some(obj) = Self::get_property(&mut scope, globals, "BotloaderCore") {
                if let Ok(v) = TryFrom::try_from(obj) {
                    v
                } else {
                    error!("BotloaderCore is not an object, unable to dispatch events");
                    return;
                }
            } else {
                error!("BotloaderCore global not found, unable to dispatch events");
                return;
            };

        let dispatch_fn: v8::Local<v8::Function> = if let Some(field) =
            Self::get_property(&mut scope, core_obj, "dispatchWrapper")
        {
            if let Ok(v) = TryFrom::try_from(field) {
                v
            } else {
                error!(
                    "BotloaderCore.dispatchWrapper is not a function, unable to dispatch events"
                );
                return;
            }
        } else {
            error!("BotloaderCore.dispatchWrapper not defined, unable to dispatch events");
            return;
        };

        let v = deno_core::serde_v8::to_v8(&mut scope, &data).unwrap();
        let _ = dispatch_fn.call(&mut scope, globals.into(), &[v]);

        let elapsed = Utc::now().signed_duration_since(event.source_timestamp);
        let millis = elapsed.num_milliseconds();
        let class = match event.source {
            common::dispatch_event::EventSource::Discord => "discord",
            common::dispatch_event::EventSource::Timer => "timer",
        };

        histogram!("dispatch_event_latency", "event_source" => class).record(millis as f64)
    }

    fn get_property<'a>(
        scope: &mut v8::HandleScope<'a>,
        object: v8::Local<v8::Object>,
        key: &str,
    ) -> Option<v8::Local<'a, v8::Value>> {
        let key = v8::String::new(scope, key).unwrap();
        object.get(scope, key.into())
    }

    fn _dump_heap_stats(&mut self) {
        let iso = self.runtime.v8_isolate();
        let mut stats = v8::HeapStatistics::default();
        iso.get_heap_statistics(&mut stats);
        dbg!(stats.total_heap_size());
        dbg!(stats.total_heap_size_executable());
        dbg!(stats.total_physical_size());
        dbg!(stats.total_available_size());
        dbg!(stats.total_global_handles_size());
        dbg!(stats.used_global_handles_size());
        dbg!(stats.used_heap_size());
        dbg!(stats.heap_size_limit());
        dbg!(stats.malloced_memory());
        dbg!(stats.external_memory());

        let policy = iso.get_microtasks_policy();
        dbg!(policy);
        // iso.low_memory_notification();
    }

    #[instrument(skip(self))]
    async fn stop_vm(&mut self) {
        if tokio::time::timeout(
            tokio::time::Duration::from_secs(15),
            self.run_until_completion(),
        )
        .await
        .is_err()
        {
            self.guild_logger.log(CreateLogEntry::error(
                "shutting down your vm timed out after 15 sec, cancelling all pending promises \
                 and force-shutting down now instead..."
                    .to_string(),
            ));
        }
    }

    async fn run_until_completion(&mut self) {
        loop {
            let fut = RunUntilCompletion {
                rt: &mut self.runtime,
            };

            if let Err(err) = fut.await {
                self.log_guild_err(err);
            } else {
                return;
            }
        }
    }

    async fn complete_module_eval(&mut self, fut: impl Future<Output = Result<(), AnyError>>) {
        let mut pinned: Pin<&mut dyn Future<Output = Result<(), AnyError>>> = pin!(fut);
        loop {
            let fut = CompleteModuleEval {
                rt: &mut self.runtime,
                fut: &mut pinned,
            };

            match fut.await {
                CompleteModuleEvalResult::Completed(res) => {
                    match res {
                        Ok(_) => {}
                        Err(err) => self.log_guild_err(err),
                    };
                    break;
                }
                CompleteModuleEvalResult::VmError(err) => self.log_guild_err(err),
            }
        }
    }

    fn log_guild_err(&self, err: AnyError) {
        self.guild_logger.log(CreateLogEntry::error(format!(
            "Script error occurred: {}",
            err
        )));
    }

    async fn restart(&mut self, new_scripts: Vec<Script>) {
        self.guild_logger
            .log(CreateLogEntry::info("restarting guild vm...".to_string()));

        self.stop_vm().await;

        // create a new sandbox
        {
            let mut borrow = self.script_store.borrow_mut();
            borrow.clear();
        };

        for script in &new_scripts {
            self.compile_script(script.clone());
        }

        let new_rt = Self::create_isolate(
            &self.extension_factory,
            self.module_manager.clone(),
            self.script_store.clone(),
            self.shutdown_handle.clone(),
        );

        self.runtime = new_rt;
        self.emit_isolate_handle();

        for script in new_scripts {
            self.run_script(script.id).await;
        }

        self.guild_logger
            .log(CreateLogEntry::info("vm restarted".to_string()));
    }
}

pub enum TickResult {
    VmError(AnyError),
    Completed,
    Command(Option<VmCommand>),
    Continue,
}

struct TickFuture<'a> {
    rx: &'a mut UnboundedReceiver<VmCommand>,
    rt: &'a mut JsRuntime,
    wakeup: &'a mut UnboundedReceiver<()>,
    completed: bool,
}

// Future which drives the js event loop while at the same time retrieving commands
impl<'a> core::future::Future for TickFuture<'a> {
    type Output = TickResult;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let completed = self.completed;
        if self.wakeup.poll_recv(cx).is_ready() {
            return Poll::Ready(TickResult::Continue);
        }

        if let Poll::Ready(opt) = self.rx.poll_recv(cx) {
            return Poll::Ready(TickResult::Command(opt));
        }

        match self.rt.poll_event_loop(cx, Default::default()) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(_)) => {
                if completed {
                    Poll::Pending
                } else {
                    Poll::Ready(TickResult::Completed)
                }
            }
            Poll::Ready(Err(e)) => Poll::Ready(TickResult::VmError(e)),
        }
    }
}

// future that drives the vm to completion, acquiring the isolate guard when needed
struct RunUntilCompletion<'a> {
    rt: &'a mut JsRuntime,
}

impl<'a> core::future::Future for RunUntilCompletion<'a> {
    type Output = Result<(), AnyError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.rt.poll_event_loop(cx, Default::default()) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(_)) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
        }
    }
}

// future that drives the vm to completion, acquiring the isolate guard when needed
struct CompleteModuleEval<'a, 'b, 'c> {
    rt: &'a mut JsRuntime,
    fut: &'c mut Pin<&'b mut dyn Future<Output = Result<(), AnyError>>>,
}

enum CompleteModuleEvalResult {
    Completed(Result<(), AnyError>),
    VmError(AnyError),
}

impl<'a, 'b, 'c> core::future::Future for CompleteModuleEval<'a, 'b, 'c> {
    type Output = CompleteModuleEvalResult;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.fut.as_mut().poll(cx) {
            Poll::Ready(res) => return Poll::Ready(CompleteModuleEvalResult::Completed(res)),
            Poll::Pending => {}
        }

        match self.rt.poll_event_loop(cx, PollEventLoopOptions::default()) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Err(e)) => return Poll::Ready(CompleteModuleEvalResult::VmError(e)),
            Poll::Ready(_) => {}
        }

        // we might have gotten a result on the channel after polling the event loop
        // let pinned = pin!(*self.fut);
        match self.fut.as_mut().poll(cx) {
            Poll::Ready(res) => return Poll::Ready(CompleteModuleEvalResult::Completed(res)),
            Poll::Pending => {}
        }

        Poll::Ready(CompleteModuleEvalResult::Completed(Ok(())))
    }
}

#[derive(Clone)]
pub struct VmShutdownHandle {
    terminated: Arc<AtomicBool>,
    inner: Arc<StdRwLock<ShutdownHandleInner>>,
    wakeup: mpsc::UnboundedSender<()>,
}

impl Debug for VmShutdownHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VmShutdownHandle")
            .field("terminated", &self.terminated)
            .field("inner", &"NA")
            .field("wakeup", &self.wakeup)
            .finish()
    }
}

impl VmShutdownHandle {
    pub(crate) fn new(wakeup_tx: mpsc::UnboundedSender<()>) -> Self {
        Self {
            terminated: Arc::new(AtomicBool::new(false)),
            inner: Arc::new(StdRwLock::new(ShutdownHandleInner {
                isolate_handle: None,
                shutdown_reason: None,
            })),
            wakeup: wakeup_tx,
        }
    }

    pub fn shutdown_vm(&self, reason: ShutdownReason, force: bool) {
        let mut inner = self.inner.write().unwrap();

        // only write shutdown reason once
        if self
            .terminated
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            )
            .is_ok()
        {
            inner.shutdown_reason = Some(reason);
        };

        if let Some(iso_handle) = &inner.isolate_handle {
            if force {
                iso_handle.terminate_execution();
            }
        }

        // trigger a shutdown check if we weren't in the js runtime
        self.wakeup.send(()).ok();
    }

    pub fn read_shutdown_reason(&self) -> Option<ShutdownReason> {
        let inner = self.inner.read().unwrap();
        inner.shutdown_reason
    }
}

struct ShutdownHandleInner {
    shutdown_reason: Option<ShutdownReason>,
    isolate_handle: Option<IsolateHandle>,
}

pub struct CreateRt {
    pub guild_logger: GuildLogSender,
    pub rx: UnboundedReceiver<VmCommand>,
    pub tx: UnboundedSender<VmEvent>,
    pub load_scripts: Vec<Script>,
    pub extension_factory: ExtensionFactory,
    pub extension_modules: Vec<ModuleEntry>,
}

type ExtensionFactory = Box<dyn Fn() -> Vec<Extension> + Send>;

pub fn in_mem_source_load_fn(src: &'static str) -> Box<dyn Fn() -> Result<String, AnyError>> {
    Box::new(move || Ok(src.to_string()))
}

// struct NoOpWaker;

// impl Wake for NoOpWaker {
//     fn wake(self: Arc<Self>) {}
// }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ShutdownReason {
    Runaway,
    Request,
    OutOfMemory,
    DiscordInvalidRequestsRatelimit,
}

pub type VmCreateResult = Result<CreateVmSuccess, String>;

pub struct CreateVmSuccess {
    pub future: LocalBoxFuture<'static, ()>,
    pub shutdown_handle: VmShutdownHandle,
}

struct VmCpuTracker<F> {
    future: Pin<Box<F>>,
    counter: metrics::Counter,
}

impl<F> VmCpuTracker<F> {
    fn wrap(counter: metrics::Counter, fut: F) -> Self {
        Self {
            counter,
            future: Box::pin(fut),
        }
    }
}

impl<F: Future> Future for VmCpuTracker<F> {
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let started = ThreadTime::now();

        let res = self.future.poll_unpin(cx);

        let elapsed = started.elapsed();
        self.counter.increment(elapsed.as_micros() as u64);

        res
    }
}
