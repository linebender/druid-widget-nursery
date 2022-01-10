use druid::commands::CLOSE_WINDOW;
use druid::widget::prelude::*;
use druid::widget::{Controller, Label, LabelText};
use druid::{
    Color, Data, Point, TimerToken, Vec2, Widget, WidgetExt, WindowConfig, WindowId, WindowLevel,
    WindowSizePolicy,
};
use std::time::{Duration, Instant};

use crate::WidgetExt as _;

#[derive(Clone)]
pub(crate) enum TooltipState {
    Off,
    Waiting {
        timer: TimerToken,
        last_mouse_move: Instant,
        last_mouse_pos: Point,
    },
    Showing {
        id: WindowId,
        // We store last_mouse_pos here because we seem to sometimes get a synthesized MouseMove
        // event after showing the tooltip (maybe because the mouse leaves the window?). By storing
        // the last mouse position, we can filter out these spurious moves.
        last_mouse_pos: Point,
    },
}

/// A [`Controller`] responsible for listening to mouse hovers and launching tooltip windows.
///
/// Instead of constructing this widget explicitly, you probably want to use
/// [`TooltipExt::tooltip`].
///
/// [`Controller`]: druid::widget::Controller
pub struct TooltipController<T> {
    pub(crate) text: LabelText<T>,
    pub(crate) state: TooltipState,
}

impl<T: Data, W: Widget<T>> Controller<T, W> for TooltipController<T> {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, ev: &Event, data: &mut T, env: &Env) {
        self.state = match self.state {
            TooltipState::Waiting {
                timer,
                last_mouse_move,
                last_mouse_pos,
            } => match ev {
                Event::MouseMove(ev) if ctx.is_hot() => TooltipState::Waiting {
                    timer,
                    last_mouse_move: Instant::now(),
                    last_mouse_pos: ev.window_pos,
                },
                Event::MouseDown(_) | Event::MouseUp(_) | Event::MouseMove(_) => TooltipState::Off,
                Event::Timer(tok) if tok == &timer => {
                    ctx.set_handled();
                    let elapsed = Instant::now().duration_since(last_mouse_move);
                    if elapsed > TOOLTIP_DELAY_CHECK {
                        self.text.resolve(data, env);
                        let tooltip_position_in_window_coordinates =
                            last_mouse_pos + TOOLTIP_OFFSET;
                        let win_id = ctx.new_sub_window(
                            WindowConfig::default()
                                .show_titlebar(false)
                                .window_size_policy(WindowSizePolicy::Content)
                                .set_level(WindowLevel::Tooltip(ctx.window().clone()))
                                .set_position(tooltip_position_in_window_coordinates),
                            // FIXME: we'd like to use the actual label text instead of
                            // resolving, but LabelText isn't Clone
                            Label::new(self.text.display_text())
                                .border(TOOLTIP_BORDER_COLOR, TOOLTIP_BORDER_WIDTH)
                                .on_monitor(ctx.window()),
                            data.clone(),
                            env.clone(),
                        );
                        TooltipState::Showing {
                            id: win_id,
                            last_mouse_pos,
                        }
                    } else {
                        TooltipState::Waiting {
                            timer: ctx.request_timer(TOOLTIP_DELAY - elapsed),
                            last_mouse_move,
                            last_mouse_pos,
                        }
                    }
                }
                _ => self.state.clone(),
            },
            TooltipState::Off => match ev {
                Event::MouseMove(ev) if ctx.is_hot() => TooltipState::Waiting {
                    timer: ctx.request_timer(TOOLTIP_DELAY),
                    last_mouse_move: Instant::now(),
                    last_mouse_pos: ev.window_pos,
                },
                _ => TooltipState::Off,
            },
            TooltipState::Showing { id, last_mouse_pos } => match ev {
                Event::MouseMove(ev) if ctx.is_hot() => {
                    // This is annoying. On GTK, after showing a window we instantly get a new
                    // MouseMove event, with a mouse position that tends to be slightly different
                    // than the previous one. If we don't test the positions, this causes the
                    // tooltip to immediately close.
                    if (ev.window_pos - last_mouse_pos).hypot2() > 1.0 {
                        ctx.submit_command(CLOSE_WINDOW.to(id));
                        TooltipState::Waiting {
                            timer: ctx.request_timer(TOOLTIP_DELAY),
                            last_mouse_move: Instant::now(),
                            last_mouse_pos: ev.window_pos,
                        }
                    } else {
                        self.state.clone()
                    }
                }
                Event::MouseMove(_) | Event::MouseUp(_) | Event::MouseDown(_) => {
                    ctx.submit_command(CLOSE_WINDOW.to(id));
                    self.state.clone()
                }
                _ => self.state.clone(),
            },
        };
        child.event(ctx, ev, data, env);
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        ev: &LifeCycle,
        data: &T,
        env: &Env,
    ) {
        if let LifeCycle::HotChanged(false) = ev {
            if let TooltipState::Showing { id, .. } = self.state {
                ctx.submit_command(CLOSE_WINDOW.to(id));
                self.state = TooltipState::Off;
            }
        }
        child.lifecycle(ctx, ev, data, env);
    }
}

const TOOLTIP_DELAY: Duration = Duration::from_millis(350);
const TOOLTIP_DELAY_CHECK: Duration = Duration::from_millis(320);
const TOOLTIP_BORDER_COLOR: Color = Color::BLACK;
const TOOLTIP_BORDER_WIDTH: f64 = 1.0;
// It looks better if we don't put the tooltip *right* on the tip of the mouse,
// because the mouse obstructs it.
// FIXME: this should depend on the actual cursor size.
const TOOLTIP_OFFSET: Vec2 = Vec2::new(15.0, 15.0);
