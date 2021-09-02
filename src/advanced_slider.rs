// Copyright 2021 The Druid Authors.
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

use ::std::time::Instant;

use druid::kurbo::{Point, Rect, RoundedRect};
use druid::widget::prelude::*;
use druid::{Color, RenderContext, TextLayout, Widget};

/// An advanced version of the slider, allowing interactive update of a numeric
/// value.
///
/// This slider implements `Widget<f64>`, and works on values clamped
/// in the range `min_val..max_val`. Additionally double clicks allows to enter
/// the precise value over the keyboard.
pub struct AdvancedSlider {
    min_val: f64,
    max_val: f64,
    start_val: f64,
    step_size: Option<f64>,
    signif_dig: usize,
    val_text: TextLayout<String>,
    last_click: Instant,
    input_mode: bool,
    input_string: String,
    keyboard_input_origin: bool,
    text_offset: f64,
}

impl AdvancedSlider {
    fn x_from_mouse(&self, mouse_event: &druid::MouseEvent) -> f64 {
        let mut perc_attempt: f64 = (mouse_event.pos.x - 2.0) / 120.0;
        if perc_attempt < 0.0 {
            perc_attempt = 0.0;
        } else if perc_attempt > 1.0 {
            perc_attempt = 1.0;
        }
        perc_attempt * (self.max_val - self.min_val) + self.min_val
    }

    fn data_from_attempt(&self, mut data_attempt: f64) -> (f64, String) {
        let mut modified = false;
        if data_attempt < self.min_val {
            data_attempt = self.min_val;
            modified = true;
        } else if data_attempt > self.max_val {
            data_attempt = self.max_val;
            modified = true;
        }
        let data: f64;
        let string: String;
        match self.step_size {
            Some(step_size) => {
                data = ((data_attempt - self.min_val) / step_size).floor() * step_size + self.min_val;
                string = String::from(format!("{:.*}", self.signif_dig, data));
            }
            None => {
                data = data_attempt;
                if self.keyboard_input_origin {
                    if modified {
                        string = String::from(format!("{:.*}", self.signif_dig, data));
                    } else {
                        string = self.input_string.clone();
                    }
                } else {
                    string = String::from(format!("{:.*}", self.signif_dig, data));
                }
            }
        }
        (data, string)
    }

    pub fn new() -> AdvancedSlider {
        AdvancedSlider {
            min_val: 0.0,
            max_val: 100.0,
            start_val: 50.0,
            step_size: None,
            signif_dig: 0,
            val_text: TextLayout::from_text("50"),
            last_click: Instant::now(),
            input_mode: false,
            input_string: String::from(""),
            keyboard_input_origin: false,
            text_offset: 0.0,
        }
    }

    pub fn with_range(mut self, min_val: f64, max_val: f64) -> AdvancedSlider {
        if min_val < max_val {
            self.min_val = min_val;
            self.max_val = max_val;
            self
        } else {
            self.min_val = max_val;
            self.max_val = min_val;
            self
        }
    }

    pub fn with_start_val(mut self, start_val: f64) -> AdvancedSlider {
        self.start_val = start_val;
        self.val_text = TextLayout::from_text(format!("{:.*}", self.signif_dig, start_val));
        self
    }

    pub fn with_step_size(mut self, step_size: f64) -> AdvancedSlider {
        if step_size <= 0.0 {
            self.step_size = None;
        } else {
            self.step_size = Some(step_size);
        }
        self
    }

    pub fn with_significant(mut self, signif_dig: usize) -> AdvancedSlider {
        self.signif_dig = signif_dig;
        self
    }

    pub fn with_text_offset(mut self, offset: f64) -> AdvancedSlider {
        self.text_offset = offset;
        self
    }
}

impl Widget<f64> for AdvancedSlider {
    /// Handles clicking and draging the slider bar, aswell as a double click
    /// for Keyboard input
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut f64, env: &Env) {
        match event {
            Event::WindowConnected => {
                *data = self.start_val;
                ctx.request_paint();
            }

            Event::KeyDown(key_event) => match &key_event.key {
                druid::keyboard_types::Key::Enter => {
                    ctx.resign_focus();
                    let try_parse = self.input_string.parse::<f64>();
                    match try_parse {
                        Ok(parsed_input) => {
                            self.keyboard_input_origin = true;
                            let data_tuple = self.data_from_attempt(parsed_input);
                            *data = data_tuple.0;
                            self.val_text = TextLayout::from_text(format!("{}", data_tuple.1));
                        }

                        Err(_) => {
                            self.val_text =
                                TextLayout::from_text(format!("{:.*}", self.signif_dig, data));
                        }
                    }
                    self.val_text.rebuild_if_needed(ctx.text(), env);
                    self.input_mode = false;
                    ctx.request_paint();
                }

                druid::keyboard_types::Key::Character(string) => match string.as_str() {
                    "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "." | "-" => {
                        self.input_string.push_str(&string);
                        self.val_text = TextLayout::from_text(format!("{}", self.input_string));
                        self.val_text.rebuild_if_needed(ctx.text(), env);
                        ctx.request_paint();
                    }
                    _ => {}
                }

                druid::keyboard_types::Key::Backspace => {
                    self.input_string.pop();
                    self.val_text = TextLayout::from_text(format!("{}", self.input_string));
                    self.val_text.rebuild_if_needed(ctx.text(), env);
                    ctx.request_paint();
                }

                _ => {}
            }

            Event::MouseDown(mouse_event) => {
                if !self.input_mode {
                    if self.last_click.elapsed().as_millis() < 100_u128 {
                        self.input_mode = true;
                        self.input_string = String::from("");
                        self.val_text = TextLayout::from_text(format!("{}", self.input_string));
                        self.val_text.rebuild_if_needed(ctx.text(), env);
                        ctx.request_focus();
                        ctx.request_paint();
                    } else {
                        ctx.set_active(true);
                        let data_attempt = self.x_from_mouse(mouse_event);
                        let data_tuple = self.data_from_attempt(data_attempt);
                        *data = data_tuple.0;
                        self.val_text = TextLayout::from_text(format!("{}", data_tuple.1));
                        self.val_text.rebuild_if_needed(ctx.text(), env);
                    }
                }
            }

            Event::MouseUp(_mouse_event) => {
                ctx.set_active(false);
                self.last_click = Instant::now();
            }

            Event::MouseMove(mouse_event) => {
                if !self.input_mode {
                    if ctx.is_active() {
                        let data_attempt = self.x_from_mouse(mouse_event);
                        let data_tuple = self.data_from_attempt(data_attempt);
                        *data = data_tuple.0;
                        self.val_text = TextLayout::from_text(format!("{}", data_tuple.1));
                        self.val_text.rebuild_if_needed(ctx.text(), env);
                    }
                }
            }

            _ => {}
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _data: &f64, _env: &Env) {
        match event {
            LifeCycle::WidgetAdded => {
                ctx.request_layout();
                ctx.request_paint();
            }
            _ => {}
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &f64, data: &f64, env: &Env) {
        if old_data != data {
            if self.keyboard_input_origin {
                self.keyboard_input_origin = false;
                ctx.request_layout();
                ctx.request_paint();
            } else {
                self.val_text = TextLayout::from_text(format!("{:.*}", self.signif_dig, data));
                self.val_text.rebuild_if_needed(ctx.text(), env);
                ctx.request_layout();
                ctx.request_paint();
            }
            
        }
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        _bc: &BoxConstraints,
        _data: &f64,
        _env: &Env,
    ) -> Size {
        Size::new(124.0, 24.0)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &f64, _env: &Env) {
        if self.input_mode {
            let boks = RoundedRect::new(2.0, 2.0, 122.0, 22.0, 2.0);
            ctx.fill(boks, &Color::rgb8(50, 50, 50));
            ctx.stroke(boks, &Color::rgb8(80, 80, 80), 1.0);

            let text_width = self.val_text.layout_metrics().size.width;
            self.val_text.draw(
                ctx,
                Point::new(62.0 - (text_width / 2.0), 2.0 + self.text_offset),
            );
        } else {
            let boks = RoundedRect::new(2.0, 2.0, 122.0, 22.0, 2.0);
            let percentage = (data - self.min_val) / (self.max_val - self.min_val) * 100.0;
            let blocker = Rect::new(percentage * 1.2 + 2.0, 2.0, 122.0, 22.0);

            if (data < &self.min_val) | (data > &self.max_val) {
                ctx.fill(boks, &Color::rgb8(212, 32, 35));
            } else {
                ctx.fill(boks, &Color::rgb8(41, 128, 186));
                ctx.fill(blocker, &Color::rgb8(80, 80, 80));
            }
            ctx.stroke(boks, &Color::rgb8(30, 30, 30), 1.0);

            let text_width = self.val_text.layout_metrics().size.width;
            self.val_text.draw(
                ctx,
                Point::new(62.0 - (text_width / 2.0), 2.0 + self.text_offset),
            );
        }
    }
}