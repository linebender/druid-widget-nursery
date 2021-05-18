// Copyright 2018 The Druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! widgets that can run async tasks

// TODO: add StreamWidget

use std::{any::Any, future::Future, pin::Pin, thread};

use druid::widget::prelude::*;
use druid::{
    AppDelegate, AppLauncher, Data, ExtEventSink, Handled, Selector, SingleUse, Target, WidgetPod,
};
use flume::{Receiver, Sender};
use futures::future::{self, BoxFuture};
use futures::prelude::*;
use tokio::runtime;

struct Request {
    future: Pin<Box<dyn Future<Output = Box<dyn Any + Send>> + Send>>,
    sender: WidgetId,
}

struct Response {
    value: Box<dyn Any + Send>,
}

const ASYNC_RESPONSE: Selector<SingleUse<Response>> = Selector::new("druid-async.async-response");
const SPAWN_ASYNC: Selector<SingleUse<Request>> = Selector::new("druid-async.spawn-async");

pub struct Delegate {
    tx: Sender<Request>,
}

impl Delegate {
    pub fn new<T: Data + 'static>(launcher: AppLauncher<T>) -> AppLauncher<T> {
        let sink = launcher.get_external_handle();
        let (tx, rx) = flume::unbounded();
        thread::spawn(move || {
            other_thread(sink, rx);
        });

        launcher.delegate(Self { tx })
    }
}

impl<T: Data> AppDelegate<T> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut druid::DelegateCtx,
        _target: druid::Target,
        cmd: &druid::Command,
        _data: &mut T,
        _env: &Env,
    ) -> Handled {
        if let Some(req) = cmd.get(SPAWN_ASYNC) {
            let req = req.take().expect("Someone stole our SPAWN_ASYNC command.");
            self.tx.send(req).unwrap();
            Handled::Yes
        } else {
            Handled::No
        }
    }
}

// TODO: make this work with other runtimes
fn other_thread(sink: ExtEventSink, rx: Receiver<Request>) {
    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let rx = rx.stream();
        rx.for_each(|req| {
            let sink = sink.clone();
            rt.spawn(async move {
                let res = req.future.await;
                let res = Response { value: res };
                let sender = req.sender;

                sink.submit_command(ASYNC_RESPONSE, SingleUse::new(res), Target::Widget(sender))
                    .unwrap();
            });
            future::ready(())
        })
        .await;
    });
}

pub type FutureWidgetAction<T> =
    Box<dyn FnOnce(&T, &Env) -> BoxFuture<'static, Box<dyn Any + Send>>>;
pub type FutureWidgetDone<T, U> = Box<dyn FnOnce(Box<U>, &mut T, &Env) -> Box<dyn Widget<T>>>;

pub struct FutureWidget<T, U> {
    future: Option<FutureWidgetAction<T>>,
    inner: WidgetPod<T, Box<dyn Widget<T>>>,
    on_done: Option<FutureWidgetDone<T, U>>,
}

impl<T, U> FutureWidget<T, U> {
    pub fn new<FMaker, Fut, Done>(
        future_maker: FMaker,
        pending: impl Widget<T> + 'static,
        on_done: Done,
    ) -> Self
    where
        U: Send + 'static,
        FMaker: FnOnce(&T, &Env) -> Fut + 'static,
        Fut: Future<Output = U> + 'static + Send,
        Done: FnOnce(Box<U>, &mut T, &Env) -> Box<dyn Widget<T>> + 'static,
    {
        Self {
            future: Some(Box::new(move |data, env| {
                let fut = future_maker(data, env);
                Box::pin(async move { Box::new(fut.await) as _ })
            })),
            inner: WidgetPod::new(Box::new(pending)),
            on_done: Some(Box::new(on_done)),
        }
    }
}

impl<T: Data, U: 'static> Widget<T> for FutureWidget<T, U> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Event::Command(cmd) = event {
            if let Some(res) = cmd.get(ASYNC_RESPONSE) {
                let res = res.take().unwrap();
                let value = res.value.downcast::<U>().unwrap();
                let on_done = self.on_done.take().unwrap();
                self.inner = WidgetPod::new((on_done)(value, data, env));
                ctx.children_changed();
                return;
            }
            #[cfg(debug_assertions)]
            if cmd.is(SPAWN_ASYNC) {
                // SPAWN_ASYNC should always be handled by the delegate
                panic!("FutureWidget used without using druid_async::Delegate");
            }
        }
        self.inner.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            ctx.submit_command(SPAWN_ASYNC.with(SingleUse::new(Request {
                future: (self.future.take().unwrap())(data, env),
                sender: ctx.widget_id(),
            })));
        }
        self.inner.lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        self.inner.update(ctx, data, env)
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &T,
        env: &Env,
    ) -> druid::Size {
        self.inner.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.inner.paint(ctx, data, env)
    }
}
