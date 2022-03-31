use std::{
    cell::Cell,
    ops::{Deref, DerefMut},
    time::Duration,
};

use cpu_time::ThreadTime;
use deno_core::{JsRuntime, RuntimeOptions};
use tracing::span;

/// IsolateCell is a tracker for wether someone has entered a isolate or not
/// this removed the need for manual unsafe management of the enter and exit states of isolates
///
/// WARNING: Holding guards across awaits will probably lead to collisions, and the below assertion will fail
#[derive(Default)]
pub struct IsolateCell {
    entered: Cell<bool>,
    updated_at: Cell<Option<ThreadTime>>,
    on_held: Option<Box<dyn Fn(Duration)>>,
}

impl IsolateCell {
    pub fn new_with_tracker(f: Box<dyn Fn(Duration)>) -> Self {
        Self {
            on_held: Some(f),
            ..Default::default()
        }
    }

    pub fn enter_isolate<'a, 'b>(&'a self, rt: &'b mut ManagedIsolate) -> IsolateGuard<'a, 'b> {
        self._enter();

        // SAFETY: we only allow a single isolate to be entered per the above guard
        // Also managed isolates are exited after creation
        unsafe {
            rt.inner.v8_isolate().enter();
        }

        IsolateGuard {
            cell: self,
            _tracing_span: span!(tracing::Level::TRACE, "isolate").entered(),
            rt,
        }
    }

    fn _enter(&self) {
        assert!(!self.entered.get());

        self.updated_at.set(Some(ThreadTime::now()));
        self.entered.set(true);
    }

    fn exit(&self) {
        assert!(self.entered.get());

        self.emit_used_cpu();
        self.entered.set(false);
    }

    fn emit_used_cpu(&self) {
        let now = ThreadTime::now();
        let last = if let Some(t) = self.updated_at.get() {
            t
        } else {
            return;
        };

        let elapsed = now.duration_since(last);
        self.updated_at.set(Some(now));

        if let Some(held_cb) = &self.on_held {
            held_cb(elapsed);
        }
    }
}

pub struct IsolateGuard<'a, 'b> {
    cell: &'a IsolateCell,
    rt: &'b mut ManagedIsolate,
    _tracing_span: tracing::span::EnteredSpan,
}

impl<'a, 'b> Drop for IsolateGuard<'a, 'b> {
    fn drop(&mut self) {
        // SAFETY: there's no way to construct a guard without entering the isolate
        unsafe { self.rt.inner.v8_isolate().exit() };

        self.cell.exit();
    }
}

impl Deref for IsolateGuard<'_, '_> {
    type Target = JsRuntime;

    fn deref(&self) -> &Self::Target {
        &self.rt.inner
    }
}

impl DerefMut for IsolateGuard<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rt.inner
    }
}

/// ManagedIsolate is a isolate where the enter and exit state is managed by the IsolateCell
/// this removed the need for manualFutOutafe management of the enter and exit states
pub struct ManagedIsolate {
    inner: JsRuntime,
}

impl ManagedIsolate {
    pub fn new(opts: RuntimeOptions) -> Self {
        let mut rt = JsRuntime::new(opts);

        // SAFETY: new enters the isolate
        unsafe { rt.v8_isolate().exit() }

        Self { inner: rt }
    }

    pub fn new_with_state<T: 'static>(opts: RuntimeOptions, initial_state: T) -> Self {
        let mut rt = JsRuntime::new(opts);
        {
            let op_state = rt.op_state();
            op_state.borrow_mut().put(initial_state);
        }

        // SAFETY: new enters the isolate
        unsafe { rt.v8_isolate().exit() }

        Self { inner: rt }
    }

    pub fn new_with_oom_handler<C: FnMut(usize, usize) -> usize + 'static>(
        opts: RuntimeOptions,
        cb: C,
    ) -> Self {
        let mut rt = JsRuntime::new(opts);
        rt.add_near_heap_limit_callback(cb);

        // SAFETY: new enters the isolate
        unsafe { rt.v8_isolate().exit() }

        Self { inner: rt }
    }

    pub fn new_with_oom_handler_and_state<C: FnMut(usize, usize) -> usize + 'static, T: 'static>(
        opts: RuntimeOptions,
        cb: C,
        initial_state: T,
    ) -> Self {
        let mut rt = JsRuntime::new(opts);
        {
            let op_state = rt.op_state();
            op_state.borrow_mut().put(initial_state);
        }
        rt.add_near_heap_limit_callback(cb);

        // SAFETY: new enters the isolate
        unsafe { rt.v8_isolate().exit() }

        Self { inner: rt }
    }
}

impl Drop for ManagedIsolate {
    fn drop(&mut self) {
        // SAFETY: it's dropped right after we enter it so there should be no lingering side effects
        unsafe { self.inner.v8_isolate().enter() }
    }
}
