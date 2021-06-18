use std::cmp::Ordering;

use druid::widget::{Axis, ListIter};
use druid::{
    theme, BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx,
    PaintCtx, Point, Rect, Size, UpdateCtx, Widget, WidgetPod,
};
use druid::{widget::prelude::*, Cursor};
use log::trace;

/// Split meet List, with resizable width/height, use like a List
pub struct Splits<T> {
    closure: Box<dyn Fn() -> Box<dyn Widget<T>>>,
    children: Vec<WidgetPod<T, Box<dyn Widget<T>>>>,
    axis: Axis,
    draggable: bool,
    major_pos_vec: Vec<f64>,
    bar_selected: i16,
    min_size: f64,
    bar_size: f64,
}

impl<T: Data> Splits<T> {
    pub fn new<W: Widget<T> + 'static>(closure: impl Fn() -> W + 'static) -> Self {
        Splits {
            closure: Box::new(move || Box::new(closure())),
            children: Vec::new(),
            axis: Axis::Vertical,
            min_size: 0.0,
            bar_size: 6.0,
            draggable: false,
            major_pos_vec: vec![],
            bar_selected: 0,
        }
    }

    pub fn horizontal(mut self) -> Self {
        self.axis = Axis::Horizontal;
        self
    }

    fn update_child_count(&mut self, data: &impl ListIter<T>, _env: &Env, index: i8) -> bool {
        let len = self.children.len();
        match len.cmp(&data.data_len()) {
            Ordering::Greater => {
                if index >= 0 && (index as usize) < self.children.len() {
                    self.children.remove(index as usize);
                    self.major_pos_vec.truncate(data.data_len());
                    // TODO: recalculate positions after removal
                }
            }
            Ordering::Less => data.for_each(|_, i| {
                if i >= len {
                    let child = WidgetPod::new((self.closure)());
                    self.children.push(child);
                    let new_major_pos = match self.major_pos_vec.last() {
                        Some(v) => v + self.min_size + self.bar_size,
                        None => self.min_size + self.bar_size,
                    };
                    self.major_pos_vec.push(new_major_pos);
                }
            }),
            Ordering::Equal => (),
        }
        len != data.data_len()
    }

    pub fn bar_size(mut self, value: f64) -> Self {
        assert!(value >= 0.0);
        self.bar_size = value;
        self
    }
    pub fn min_size(mut self, value: f64) -> Self {
        assert!(value >= 0.0);
        self.min_size = value.ceil();
        self
    }

    pub fn draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }

    fn paint_bar(&mut self, ctx: &mut PaintCtx, env: &Env) {
        let splitter_color = env.get(theme::BORDER_LIGHT);
        let size = ctx.size();
        let minor = match self.axis {
            Axis::Horizontal => size.height,
            Axis::Vertical => size.width,
        };
        self.major_pos_vec.iter().for_each(|pos| {
            let bar_start = pos - self.bar_size;
            let bar_end = pos;
            let bar_start_point: Point = self.axis.pack(bar_start, 0.).into();
            let bar_end_point: Point = self.axis.pack(*bar_end, minor).into();
            let bar_rect = Rect::from_points(bar_start_point, bar_end_point);
            ctx.fill(bar_rect, &splitter_color);
        });
    }

    fn bar_hit_test(&self, mouse_pos: Point) -> i16 {
        for (idx, edge) in self.major_pos_vec.iter().enumerate() {
            let mouse_pos_axis = match self.axis {
                Axis::Horizontal => mouse_pos.x,
                Axis::Vertical => mouse_pos.y,
            };
            if mouse_pos_axis > (edge - self.bar_size) && mouse_pos_axis < *edge {
                return idx as i16;
            }
        }
        -1
    }

    fn update_bar_pos(&mut self, mouse_pos: Point) {
        if self.bar_selected < 0 {
            return;
        }
        let mut diff = 0.0;
        let mut previous_end = 0.0;
        let mouse_pos_axis = match self.axis {
            Axis::Horizontal => mouse_pos.x,
            Axis::Vertical => mouse_pos.y,
        };
        for i in 0..self.major_pos_vec.len() {
            let cur_point = self.major_pos_vec[i];
            if (self.bar_selected as usize) > i {
                previous_end = cur_point;
                continue;
            }
            if i == self.bar_selected as usize {
                diff = mouse_pos_axis - cur_point;
            }
            let new_pos = cur_point + diff;
            if (new_pos - previous_end) < self.min_size {
                break;
            }
            self.major_pos_vec[i as usize] = new_pos;
            previous_end = new_pos;
        }
    }
}

// Copy of Axis.constraints() because is crate only
fn axis_constraints(axis: Axis, bc: &BoxConstraints, min_major: f64, major: f64) -> BoxConstraints {
    match axis {
        Axis::Horizontal => BoxConstraints::new(
            Size::new(min_major, bc.min().height),
            Size::new(major, bc.max().height),
        ),
        Axis::Vertical => BoxConstraints::new(
            Size::new(bc.min().width, min_major),
            Size::new(bc.max().width, major),
        ),
    }
}

impl<C: Data, T: ListIter<C>> Widget<T> for Splits<C> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        let mut children = self.children.iter_mut();
        data.for_each_mut(|child_data, _| {
            if let Some(child) = children.next() {
                child.event(ctx, event, child_data, env);
            }
        });

        if self.draggable {
            match event {
                Event::MouseDown(mouse) => {
                    if mouse.button.is_left() {
                        let bar_idx = self.bar_hit_test(mouse.pos);
                        if bar_idx >= 0 {
                            self.bar_selected = bar_idx;
                            ctx.set_active(true);
                            ctx.set_handled();
                        }
                    }
                }
                Event::MouseUp(mouse) => {
                    if mouse.button.is_left() && ctx.is_active() {
                        ctx.set_active(false);
                        self.update_bar_pos(mouse.pos);
                        ctx.request_paint();
                        self.bar_selected = 0;
                    }
                }
                Event::MouseMove(mouse) => {
                    if ctx.is_active() {
                        self.update_bar_pos(mouse.pos);
                        ctx.request_layout();
                    }

                    if ctx.is_hot() || ctx.is_active() {
                        if self.bar_hit_test(mouse.pos) >= 0 {
                            match self.axis {
                                Axis::Horizontal => ctx.set_cursor(&Cursor::ResizeLeftRight),
                                Axis::Vertical => ctx.set_cursor(&Cursor::ResizeUpDown),
                            }
                        } else {
                            ctx.clear_cursor();
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            if self.update_child_count(data, env, -1) {
                ctx.children_changed();
            }
        }

        let mut children = self.children.iter_mut();
        data.for_each(|child_data, _| {
            if let Some(child) = children.next() {
                child.lifecycle(ctx, event, child_data, env);
            }
        });
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        let mut children = self.children.iter_mut();
        data.for_each(|child_data, _| {
            if let Some(child) = children.next() {
                child.update(ctx, child_data, env);
            }
        });

        let mut removed: i8 = -1;
        if old_data.data_len() > data.data_len() {
            old_data.for_each(|child_data, index| {
                if removed == -1 {
                    data.for_each(|ch, i| {
                        if i == index && !child_data.same(ch) {
                            removed = i as i8;
                        }
                    });
                }
            });
            if removed == -1 {
                removed = (old_data.data_len() as i8) - 1;
            }
        }
        if self.update_child_count(data, env, removed) {
            ctx.children_changed();
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let axis = self.axis;
        let bar_size = self.bar_size;
        let mut minor = axis.minor(bc.min());
        let mut major_pos = 0.0;
        let mut paint_rect = Rect::ZERO;
        let mut children = self.children.iter_mut();
        let mut major_pos_vec_iter = self.major_pos_vec.iter_mut();
        data.for_each(|child_data, _| {
            let child = match children.next() {
                Some(child) => child,
                None => {
                    return;
                }
            };

            let child_pos: Point = axis.pack(major_pos, 0.).into();
            let last_pos = major_pos;

            major_pos = match major_pos_vec_iter.next() {
                Some(p) => *p,
                None => {
                    return;
                }
            };
            let new_major_size = major_pos - last_pos - bar_size;
            let child_bc = axis_constraints(axis, bc, new_major_size, new_major_size);
            let child_size = child.layout(ctx, &child_bc, child_data, env);

            child.set_origin(ctx, child_data, env, child_pos);

            paint_rect = paint_rect.union(child.paint_rect());
            minor = minor.max(axis.minor(child_size));
        });

        let my_size = bc.constrain(Size::from(axis.pack(major_pos, minor)));
        let insets = paint_rect - my_size.to_rect();
        ctx.set_paint_insets(insets);
        trace!("Computed layout: size={}, insets={:?}", my_size, insets);
        my_size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.paint_bar(ctx, env);
        let mut children = self.children.iter_mut();
        data.for_each(|child_data, _| {
            if let Some(child) = children.next() {
                child.paint(ctx, child_data, env);
            }
        });
    }
}
