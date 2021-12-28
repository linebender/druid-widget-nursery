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
    /// Takes a mouse event and returns the slider value at the specified
    /// x position.
    fn x_from_mouse(&self, mouse_event: &druid::MouseEvent) -> f64 {
        // Determines percentage regarding the slider size 120.0
        let mut perc_attempt: f64 = (mouse_event.pos.x - 2.0) / 120.0;
        // Make sure percentage is bounded between 0 and 1
        if perc_attempt < 0.0 {
            perc_attempt = 0.0;
        } else if perc_attempt > 1.0 {
            perc_attempt = 1.0;
        }
        // Convert to value
        perc_attempt * (self.max_val - self.min_val) + self.min_val
    }

    /// Takes a data attempt and converts it to valid data, that satisfies the
    /// stepping, as well as min and max value of the slider. Also returns a
    /// formated String which is used for the label and satisfies the
    /// significant digits option.
    ///
    /// When stepping is disabled the returned String is identical to the
    /// keyboard input and therefore allows more significant digits to be
    /// displayed to show the exact value.
    fn data_from_attempt(&self, mut data_attempt: f64) -> (f64, String) {
        // Track whether attempt was out of bounds to correct the input string
        // in the case of no stepping
        let mut modified = false;
        // Ensure data is bounded
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
                // Apply stepping
                data =
                    ((data_attempt - self.min_val) / step_size).round() * step_size + self.min_val;
                string = format!("{:.*}", self.signif_dig, data);
            }
            None => {
                data = data_attempt;
                if self.keyboard_input_origin {
                    if modified {
                        // Correct String when data was corrected
                        string = format!("{:.*}", self.signif_dig, data);
                    } else {
                        // Otherwise take input string from the keyboard
                        string = self.input_string.clone();
                    }
                } else {
                    // Format String when the origin wasn't keyboard input
                    string = format!("{:.*}", self.signif_dig, data);
                }
            }
        }
        (data, string)
    }

    /// Builder style method for constructing a new slider
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

    /// Builder style method for setting the range.
    pub fn with_range(mut self, min_val: f64, max_val: f64) -> AdvancedSlider {
        if min_val < max_val {
            self.min_val = min_val;
            self.max_val = max_val;
        } else {
            self.min_val = max_val;
            self.max_val = min_val;
        }
        self
    }

    /// Builder style method for setting the start value.
    pub fn with_start_val(mut self, start_val: f64) -> AdvancedSlider {
        self.start_val = start_val;
        self.val_text = TextLayout::from_text(format!("{:.*}", self.signif_dig, start_val));
        self
    }

    /// Builder style method to set the stepping size. Zero coresponds to no
    /// (infinite small) stepping.
    pub fn with_step_size(mut self, step_size: f64) -> AdvancedSlider {
        if step_size <= 0.0 {
            self.step_size = None;
        } else {
            self.step_size = Some(step_size);
        }
        self
    }

    /// Builder style method to set the significant digits for displaying.
    pub fn with_significant(mut self, signif_dig: usize) -> AdvancedSlider {
        self.signif_dig = signif_dig;
        self
    }

    /// Builder style method to give the label and offset. Sometimes necessary
    /// on different operating systems with different fonts ? <- Not sure about that
    pub fn with_text_offset(mut self, offset: f64) -> AdvancedSlider {
        self.text_offset = offset;
        self
    }
}

impl Default for AdvancedSlider {
    fn default() -> Self {
        Self::new()
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

            Event::MouseDown(mouse_event) => {
                // Make sure the widget only reacts when not in input mode
                if !self.input_mode {
                    // Check whether a double click happened
                    if self.last_click.elapsed().as_millis() < 100_u128 {
                        // Enter input mode
                        self.input_mode = true;
                        self.input_string = String::from("");
                        self.val_text = TextLayout::from_text(self.input_string.to_string());
                        self.val_text.rebuild_if_needed(ctx.text(), env);
                        ctx.request_focus();
                        ctx.request_paint();
                    } else {
                        // Handle simple click
                        ctx.set_active(true);
                        let data_attempt = self.x_from_mouse(mouse_event);
                        let data_tuple = self.data_from_attempt(data_attempt);
                        *data = data_tuple.0;
                    }
                }
            }

            // Handle mouse up and set time instance for double click
            Event::MouseUp(_mouse_event) => {
                ctx.set_active(false);
                self.last_click = Instant::now();
            }

            // Handle mouse movement for dragging of the slider
            Event::MouseMove(mouse_event) => {
                // Make sure the widget only reacts if not in input mode
                if !self.input_mode {
                    // Make sure widget only reacts when active
                    if ctx.is_active() {
                        let data_attempt = self.x_from_mouse(mouse_event);
                        let data_tuple = self.data_from_attempt(data_attempt);
                        *data = data_tuple.0;
                    }
                }
            }

            Event::KeyDown(key_event) => match &key_event.key {
                // Enter to confirm keyboard input
                druid::keyboard_types::Key::Enter => {
                    ctx.resign_focus();
                    let try_parse = self.input_string.parse::<f64>();
                    match try_parse {
                        // When parsable -> specify keyboard origin and convert to data
                        Ok(parsed_input) => {
                            self.keyboard_input_origin = true;
                            let data_tuple = self.data_from_attempt(parsed_input);
                            *data = data_tuple.0;
                            self.val_text = TextLayout::from_text(data_tuple.1);
                            self.val_text.rebuild_if_needed(ctx.text(), env);
                            self.input_mode = false;
                        }

                        // When not parsable -> revert to old data
                        Err(_) => {
                            self.val_text =
                                TextLayout::from_text(format!("{:.*}", self.signif_dig, data));
                            self.val_text.rebuild_if_needed(ctx.text(), env);
                            self.input_mode = false;
                            ctx.request_paint();
                        }
                    }
                }

                // Handle allowed input characters
                druid::keyboard_types::Key::Character(string) => match string.as_str() {
                    "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "." | "-" => {
                        self.input_string.push_str(string);
                        self.val_text = TextLayout::from_text(self.input_string.to_string());
                        self.val_text.rebuild_if_needed(ctx.text(), env);
                        ctx.request_paint();
                    }
                    _ => {}
                },

                // Handle deleting chararcters of the input sting
                druid::keyboard_types::Key::Backspace => {
                    self.input_string.pop();
                    self.val_text = TextLayout::from_text(self.input_string.to_string());
                    self.val_text.rebuild_if_needed(ctx.text(), env);
                    ctx.request_paint();
                }

                _ => {}
            },

            _ => {}
        }
    }

    // Handle initialisation
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _data: &f64, _env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            ctx.request_layout();
            ctx.request_paint();
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &f64, data: &f64, env: &Env) {
        // Not really nessecary but != gives a clippy error ;)
        if (old_data - data).abs() > f64::EPSILON {
            // For the case data gets modified while in input mode
            self.input_mode = false;
            if self.keyboard_input_origin {
                self.keyboard_input_origin = false;
            } else {
                self.val_text = TextLayout::from_text(format!("{:.*}", self.signif_dig, data));
                self.val_text.rebuild_if_needed(ctx.text(), env);
            }
            ctx.request_layout();
            ctx.request_paint();
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
        let rounded_box = RoundedRect::new(2.0, 2.0, 122.0, 22.0, 2.0);
        // Handle in which mode to draw the widget
        if self.input_mode {
            ctx.fill(rounded_box, &Color::rgb8(50, 50, 50));
            ctx.stroke(rounded_box, &Color::rgb8(80, 80, 80), 1.0);
        } else {
            let percentage = (data - self.min_val) / (self.max_val - self.min_val) * 100.0;
            let blocker = Rect::new(percentage * 1.2 + 2.0, 2.0, 122.0, 22.0);

            // Constrain blocker to within the slider. A blocker is used to make
            // sure the slider is flat on one side and rounded on the other side.
            if (data < &self.min_val) | (data > &self.max_val) {
                ctx.fill(rounded_box, &Color::rgb8(212, 32, 35));
            } else {
                ctx.fill(rounded_box, &Color::rgb8(41, 128, 186));
                ctx.fill(blocker, &Color::rgb8(80, 80, 80));
            }
            ctx.stroke(rounded_box, &Color::rgb8(30, 30, 30), 1.0);
        }
        // Center Text and draw it
        let text_width = self.val_text.layout_metrics().size.width;
        self.val_text.draw(
            ctx,
            Point::new(62.0 - (text_width / 2.0), 2.0 + self.text_offset),
        );
    }
}
