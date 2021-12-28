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

use std::{any::Any, future::Future, pin::Pin};

use druid::widget::prelude::*;
use druid::{Data, Selector, SingleUse, Target, WidgetPod};

struct Response {
    value: Box<dyn Any + Send>,
}

const ASYNC_RESPONSE: Selector<SingleUse<Response>> = Selector::new("druid-async.async-response");

#[cfg(target_arch = "wasm32")]
pub type FutureWidgetAction<T> =
    Box<dyn FnOnce(&T, &Env) -> Pin<Box<dyn Future<Output = Box<dyn Any + Send>>>>>;

#[cfg(not(target_arch = "wasm32"))]
pub type FutureWidgetAction<T> =
    Box<dyn FnOnce(&T, &Env) -> Pin<Box<dyn Send + Future<Output = Box<dyn Any + Send>>>>>;

pub type FutureWidgetDone<T, U> = Box<dyn FnOnce(Box<U>, &mut T, &Env) -> Box<dyn Widget<T>>>;

pub struct FutureWidget<T, U> {
    future: Option<FutureWidgetAction<T>>,
    inner: WidgetPod<T, Box<dyn Widget<T>>>,
    on_done: Option<FutureWidgetDone<T, U>>,
}

#[cfg(target_arch = "wasm32")]
impl<T, U> FutureWidget<T, U> {
    pub fn new<FMaker, Fut, Done>(
        future_maker: FMaker,
        pending: impl Widget<T> + 'static,
        on_done: Done,
    ) -> Self
    where
        U: Send + 'static,
        FMaker: FnOnce(&T, &Env) -> Fut + 'static,
        Fut: Future<Output = U> + 'static,
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

#[cfg(not(target_arch = "wasm32"))]
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
        }
        self.inner.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            let sink = ctx.get_external_handle();
            let widget_id = ctx.widget_id();
            let future = self.future.take().unwrap()(data, env);
            let task = async move {
                let data = future.await;
                sink.submit_command(
                    ASYNC_RESPONSE,
                    SingleUse::new(Response { value: data }),
                    Target::Widget(widget_id),
                )
                .unwrap();
            };

            #[cfg(target_arch = "wasm32")]
            wasm_bindgen_futures::spawn_local(task);

            #[cfg(not(target_arch = "wasm32"))]
            tokio::spawn(task);
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
