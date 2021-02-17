use druid::lens::Map;
use druid::widget::{Button, Flex, Label, LabelText, Split, TextBox};
use druid::{AppLauncher, Data, Env, Lens, Point, Size, Widget, WidgetExt, WindowDesc};
use druid_widget_nursery::{Canvas, CanvasWrap};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::str::FromStr;

fn main() {
    let main_window = WindowDesc::new(ui_builder()).window_size(Size::new(600., 400.));

    let data = AppData::new();

    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
        .expect("launch failed")
}

fn ui_builder() -> impl Widget<AppData> {
    Split::columns(canvas_panel(), sidebar()).split_point(0.66)
}
fn canvas_panel() -> impl Widget<AppData> {
    let label = Label::dynamic(|data: &AppData, _: &Env| data.label_text.clone());
    let text_box = TextBox::new().fix_width(150.).lens(AppData::text_box_text);
    let button = Button::new("Click me!")
        .on_click(|_ctx, data: &mut AppData, _env| data.btn_click_count += 1);
    let canvas: Canvas<AppData> = Canvas::new()
        .with_child(CanvasWrap::new(label, |data| data.label_pos))
        .with_child(CanvasWrap::new(text_box, |data| data.text_box_pos))
        .with_child(CanvasWrap::new(button, |data| data.button_pos));

    canvas
}
fn sidebar() -> impl Widget<AppData> {
    Flex::column()
        .with_child(Label::new("Label text:").padding(4.0))
        .with_child(
            TextBox::new()
                .expand_width()
                .padding((4., 0.))
                .lens(AppData::label_text),
        )
        .with_child(position_cluster("Label position:").lens(AppData::label_pos))
        .with_child(Label::new("Text box text:").padding(4.))
        .with_child(Label::dynamic(|data: &AppData, _env| data.text_box_text.clone()).padding(4.))
        .with_child(position_cluster("Text box position:").lens(AppData::text_box_pos))
        .with_child(Label::new("Button click count:").padding(4.))
        .with_child(
            Label::dynamic(|data: &AppData, _env| data.btn_click_count.to_string()).padding(4.),
        )
        .with_child(position_cluster("Button position:").lens(AppData::button_pos))
}

fn position_cluster(text: impl Into<LabelText<Point>>) -> impl Widget<Point> {
    Flex::column()
        .with_child(Label::new(text).padding(4.0))
        .with_child(
            Flex::row()
                .with_child(Label::new("X:").padding((4., 0.)))
                .with_flex_child(
                    TextBox::new()
                        .lens(FloatToString::new())
                        .lens(Map::new(|p: &Point| p.x, |p, v| p.x = v)),
                    1.0,
                )
                .with_child(Label::new("Y:").padding((4., 0.)))
                .with_flex_child(
                    TextBox::new()
                        .lens(FloatToString::new())
                        .lens(Map::new(|p: &Point| p.y, |p, v| p.y = v)),
                    1.0,
                )
                .must_fill_main_axis(true),
        )
}

struct FloatToString {
    string: RefCell<Option<Rc<String>>>,
    float: Cell<f64>,
}

impl FloatToString {
    fn new() -> Self {
        Self {
            string: RefCell::new(None),
            float: Cell::new(0.0),
        }
    }
}

impl Lens<f64, String> for FloatToString {
    fn with<V, F: FnOnce(&String) -> V>(&self, data: &f64, f: F) -> V {
        println!("starting with");
        let s;
        let mut flag = false;
        {
            if let Some(st) = &*self.string.borrow() {
                s = Rc::clone(st);
            } else {
                flag = true;
                s = Rc::new(data.to_string());
            }
        }
        if flag {
            *self.string.borrow_mut() = Some(s.clone());
        }
        let v = f(&s);
        println!("ending with");
        v
    }

    fn with_mut<V, F: FnOnce(&mut String) -> V>(&self, data: &mut f64, f: F) -> V {
        println!("starting with_mut");
        //Okay, so, lets outline this.

        //Either way, we need to have a valid f64 to put back in data, whether parsing fails or not.
        //If parsing fails, we ensure we 'restore' data to this value
        //we definitely should eliminate this backup if it proves unnecessary.
        self.float.set(*data);

        let parse_closure = |s: &mut String, data: &mut f64| {
            let v = f(s);
            //Now, we may have mutated the string. We should see if it parses.
            if let Ok(n) = f64::from_str(s) {
                *data = n;
                self.float.set(n);
            }
            //If it doesn't, we still want the mutated string, but to leave our internal float
            //and returned data alone.
            v
        };
        //We need to see if we have a string to mutate. If we do, then take that branch.
        let v = if let Some(st) = &mut *self.string.borrow_mut() {
            let s = Rc::make_mut(st);
            //We have a string. Run the closure.
            parse_closure(s, data)
        } else {
            //If we're here, we have no string yet. We should create one from the f64.
            *self.string.borrow_mut() = Some(Rc::new(data.to_string()));
            let mut b = self.string.borrow_mut();
            let st = b.as_mut().unwrap();
            let s = Rc::make_mut(st);
            parse_closure(s, data)
        };
        println!("ending with_mut");
        v
    }
}

#[derive(Clone, Data, Lens)]
struct AppData {
    pub label_text: String,
    pub label_pos: Point,
    pub text_box_text: String,
    pub text_box_pos: Point,
    pub btn_click_count: u32,
    pub button_pos: Point,
}

impl AppData {
    fn new() -> Self {
        Self {
            label_text: "Edit this text box:".into(),
            label_pos: Point::new(150., 50.),
            text_box_text: "".into(),
            text_box_pos: Point::new(130., 90.),
            btn_click_count: 0,
            button_pos: Point::new(100., 120.),
        }
    }
}
