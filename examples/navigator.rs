// Copyright 2021 the Druid Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::sync::Arc;

use druid::{
    widget::{
        Button, Container, Flex, Label, List, ListIter, MainAxisAlignment, Painter, Scope,
        ScopeTransfer, Scroll, TextBox,
    },
    AppLauncher, Color, Command, Env, LensExt, RenderContext, Selector, Target, WidgetExt,
    WindowDesc,
};
use druid::{
    widget::{Controller, CrossAxisAlignment},
    Data, Event,
};
use druid::{Lens, Widget};

use druid_widget_nursery::navigator::{Navigator, View, ViewController};
fn main() {
    let window = WindowDesc::new(navigator()).title("Navigation");

    let contacts = get_data();

    AppLauncher::with_window(window)
        .log_to_console()
        .launch(AppState {
            app_name: "This is a paragraph about the Navigator.".to_string(),
            nav_state: Arc::new(vec![UiView::new("contacts".to_string())]),
            contacts: Arc::new(contacts),
            selected: None,
        })
        .unwrap();
}

// creates the navigator widget responsible for changing views
pub fn navigator() -> impl Widget<AppState> {
    Navigator::new(UiView::new("contacts".to_string()), contacts)
        .with_view_builder(UiView::new("contact details".to_string()), contact_details)
        .with_view_builder(UiView::new("contact edit".to_string()), contact_edit)
        .controller(NavigatorController)
}

// this controller will handle commands like POP_VIEW whenever a child widget does not
// have access to AppState
struct NavigatorController;
impl Controller<AppState, Navigator<AppState, UiView>> for NavigatorController {
    fn event(
        &mut self,
        child: &mut Navigator<AppState, UiView>,
        ctx: &mut druid::EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        match event {
            Event::Command(selector) if selector.is(POP_VIEW) => {
                data.pop_view();
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }
}

#[derive(Clone, Data, Lens, Debug)]
pub struct AppState {
    app_name: String,
    // this will act as the backing data for your navigation state
    // this should always be initialized with one view and should
    // ideally never be empty, otherwise things might not work correctly
    nav_state: Arc<Vec<UiView>>,
    contacts: Arc<Vec<Contact>>,
    selected: Option<usize>,
}

#[derive(Clone, Data, Lens, Debug)]
pub struct Contact {
    name: String,
    email: String,
    favorite_food: String,
    age: u32,
}

impl Contact {
    pub fn new(
        name: impl Into<String>,
        email: impl Into<String>,
        age: u32,
        favorite_food: impl Into<String>,
    ) -> Self {
        let name = name.into();
        let email = email.into();
        let favorite_food = favorite_food.into();
        Self {
            name,
            email,
            favorite_food,
            age,
        }
    }
}

// Here you define your view. It can be any type that implements `Hash`. You can define an Enum
// instead and use that to define your views instead of a string
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct UiView {
    name: String,
}

impl UiView {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

// implements the view trait for your view type
impl View for UiView {}

// Here you define Viewcontroller for your AppState. The navigator widget will
// only accept AppStates that implement this trait. The methods here are used
// handle modifying your navigation state without manually doing that with your
// own methods. Look at the docs to see what each method is useful for.
impl ViewController<UiView> for AppState {
    fn add_view(&mut self, view: UiView) {
        let views: &mut Vec<UiView> = Arc::make_mut(&mut self.nav_state);
        views.push(view);
        let views = Arc::new(views.clone());
        self.nav_state = views;
    }

    fn pop_view(&mut self) {
        let views = Arc::make_mut(&mut self.nav_state);
        views.pop();
        let views = Arc::new(views.clone());
        self.nav_state = views;
    }

    fn current_view(&self) -> &UiView {
        self.nav_state.last().unwrap()
    }

    fn len(&self) -> usize {
        self.nav_state.len()
    }

    fn is_empty(&self) -> bool {
        self.nav_state.is_empty()
    }
}

// main page and contains list view of contacts
// notice that this must return Box<dyn Widget<YourState>> instead of impl Widget<YourState>
// navigator needs Boxed widgets in order to store the widgets
pub fn contacts() -> Box<dyn Widget<AppState>> {
    let list = List::new(|| {
        let name_text = Label::new(
            |(_views, contact, _selection, _idx): &(
                Arc<Vec<UiView>>,
                Contact,
                Option<usize>,
                usize,
            ),
             _env: &_| { contact.name.clone() },
        )
        .with_text_color(Color::BLACK)
        .with_text_size(20.);
        let email_text = Label::new(
            |(_views, contact, _selected, _idx): &(
                Arc<Vec<UiView>>,
                Contact,
                Option<usize>,
                usize,
            ),
             _env: &_| contact.email.clone(),
        )
        .with_text_color(Color::BLACK)
        .with_text_size(20.);

        let details = Flex::column().with_child(name_text).with_child(email_text);
        let layout = Flex::row().with_child(details);
        let layout = layout.on_click(|event, data, _env| {
            let new_views = Arc::make_mut(&mut data.0);
            new_views.push(UiView::new("contact details".to_string()));
            data.0 = Arc::new(new_views.to_owned());
            data.2 = Some(data.3);
            event.submit_command(Command::new(CONTACT_DETAIL, data.3, Target::Auto));
        });

        layout.background(Painter::new(|ctx, _data, _env| {
            let is_hot = ctx.is_hot();
            let is_active = ctx.is_active();
            let rect = ctx.size().to_rect();
            let background_color = if is_active {
                Color::rgb8(0x88, 0x88, 0x88)
            } else if is_hot {
                Color::rgb8(0xdd, 0xdd, 0xdd)
            } else {
                Color::WHITE
            };
            ctx.stroke(rect, &background_color, 0.);
            ctx.fill(rect, &background_color);
        }))
    });

    let layout = Flex::row()
        .with_flex_child(Scroll::new(list.with_spacing(20.)).center(), 1.)
        .must_fill_main_axis(true)
        .expand_width();

    Box::new(Container::new(layout).background(Color::WHITE))
}

// details views - this is the second view after clicking on a contact
pub fn contact_details() -> Box<dyn Widget<AppState>> {
    let name = Label::dynamic(|data: &AppState, _env: &Env| {
        if let Some(idx) = data.selected {
            format!("Name: {}", data.contacts[idx].name)
        } else {
            "".to_string()
        }
    })
    .with_text_size(20.);

    let email = Label::new(|data: &AppState, _env: &Env| {
        if let Some(idx) = data.selected {
            format!("Email: {}", data.contacts[idx].email)
        } else {
            "".to_string()
        }
    })
    .with_text_size(20.);

    let age = Label::new(|data: &AppState, _env: &Env| {
        if let Some(idx) = data.selected {
            format!("Age: {}", data.contacts[idx].age)
        } else {
            "".to_string()
        }
    })
    .with_text_size(20.);

    let favorite_food = Label::new(|data: &AppState, _env: &Env| {
        if let Some(idx) = data.selected {
            format!("Favorite food: {}", data.contacts[idx].favorite_food)
        } else {
            "".to_string()
        }
    })
    .with_text_size(20.);

    // you might want to define a command that pops a view so that you may scope down your AppState
    let back_button = Button::new("Back").on_click(|_event, data: &mut AppState, _env| {
        data.pop_view();
    });

    let edit_button = Button::new("Edit").on_click(|event, data: &mut AppState, _env| {
        let views = Arc::make_mut(&mut data.nav_state);
        views.push(UiView::new("contact edit".to_string()));
        data.nav_state = Arc::new(views.to_owned());
        event.submit_command(Command::new(
            CONTACT_EDIT,
            data.selected.unwrap(),
            Target::Auto,
        ));
    });

    let layout = Flex::column()
        .with_child(name)
        .with_child(email)
        .with_child(age)
        .with_child(favorite_food)
        .cross_axis_alignment(CrossAxisAlignment::Start);
    let layout = Flex::column()
        .with_child(back_button)
        .with_child(layout)
        .with_child(edit_button)
        .must_fill_main_axis(true)
        .main_axis_alignment(MainAxisAlignment::SpaceAround);

    let container = Container::new(layout.center()).background(Color::GRAY);

    Box::new(container)
}

pub fn contact_edit() -> Box<dyn Widget<AppState>> {
    let back_button = Button::new("Back").on_click(|_event, data: &mut AppState, _env| {
        data.pop_view();
    });
    let name_input = Flex::column()
        .with_child(
            Label::new("Name")
                .with_text_color(Color::BLACK)
                .with_text_size(20.),
        )
        .with_child(
            TextBox::new()
                .with_text_size(20.)
                .fix_width(300.)
                .lens(Contact::name),
        )
        .cross_axis_alignment(CrossAxisAlignment::Start);

    let email_input = Flex::column()
        .with_child(
            Label::new("Email")
                .with_text_color(Color::BLACK)
                .with_text_size(20.),
        )
        .with_child(
            TextBox::new()
                .with_text_size(20.)
                .fix_width(300.)
                .lens(Contact::email),
        )
        .cross_axis_alignment(CrossAxisAlignment::Start);

    let age_input = Flex::column()
        .with_child(
            Label::new("Age")
                .with_text_color(Color::BLACK)
                .with_text_size(20.),
        )
        .with_child(
            TextBox::new()
                .with_text_size(20.)
                .fix_width(300.)
                .lens(Contact::age.map(
                    |age| age.to_string(),
                    |age, age_string| {
                        // FIX THIS: make this so that it doesn't panic on invalid inputs
                        *age = age_string.parse().unwrap_or(*age);
                    },
                )),
        )
        .cross_axis_alignment(CrossAxisAlignment::Start);

    let favorite_food_input = Flex::column()
        .with_child(
            Label::new("Favorite Food")
                .with_text_color(Color::BLACK)
                .with_text_size(20.),
        )
        .with_child(
            TextBox::new()
                .with_text_size(20.)
                .fix_width(300.)
                .lens(Contact::favorite_food),
        )
        .cross_axis_alignment(CrossAxisAlignment::Start);

    let layout = Flex::column()
        .with_child(name_input)
        .with_child(email_input)
        .with_child(age_input)
        .with_child(favorite_food_input)
        .must_fill_main_axis(true)
        .main_axis_alignment(MainAxisAlignment::Center)
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .lens(EditState::contact);

    let save_button = Button::new("Save").on_click(|event, data: &mut EditState, _env| {
        data.was_saved = true;
        // use a command here because EditState does not have access to the navigation state
        event.submit_command(POP_VIEW);
    });

    let layout = Flex::column()
        .with_flex_child(layout, 1.0)
        .with_child(save_button)
        .main_axis_alignment(MainAxisAlignment::SpaceAround);

    // use this scope widget to independently update data used for this view
    // if a lens is used the data would update automatically.
    // using a scope allows you to control when to update the AppState such as only
    // when the save button is clicked
    let layout = Scope::from_function(EditState::new, EditTransfer, layout);
    let layout = Flex::column()
        .with_child(back_button)
        .with_flex_child(layout, 1.0)
        .main_axis_alignment(MainAxisAlignment::SpaceAround);

    let container = Container::new(layout).background(Color::WHITE);

    Box::new(container)
}

// this holds state that will be used when on the edit page
#[derive(Clone, Data, Lens, Debug)]
pub struct EditState {
    contact: Contact,
    index: usize,
    was_saved: bool,
}

impl EditState {
    pub fn new(data: AppState) -> Self {
        let (contact, index) = if let Some(idx) = data.selected {
            (data.contacts[idx].clone(), idx)
        } else {
            (
                Contact::new("".to_owned(), "".to_owned(), 31, "".to_owned()),
                0,
            )
        };
        Self {
            contact,
            index,
            was_saved: false,
        }
    }
}

pub struct EditTransfer;

impl ScopeTransfer for EditTransfer {
    type In = AppState;

    type State = EditState;

    fn read_input(&self, state: &mut Self::State, inner: &Self::In) {
        // only read data in if the input was saved
        if state.was_saved {
            let selected = inner.selected;
            let idx = if let Some(idx) = selected { idx } else { 0 };
            state.contact = inner.contacts[idx].clone();
            state.index = idx;
            state.was_saved = false;
        }
    }

    fn write_back_input(&self, state: &Self::State, inner: &mut Self::In) {
        if state.was_saved {
            let contacts = Arc::make_mut(&mut inner.contacts);
            contacts[state.index] = state.contact.clone();
            inner.contacts = Arc::new(contacts.to_owned());
        }
    }
}

const CONTACT_DETAIL: Selector<usize> = Selector::new("contact detail");
const CONTACT_EDIT: Selector<usize> = Selector::new("contact edit");
const POP_VIEW: Selector<()> = Selector::new("navigator.pop-view");

// a little special implementation to give the list view all that it needs
impl ListIter<(Arc<Vec<UiView>>, Contact, Option<usize>, usize)> for AppState {
    fn for_each(
        &self,
        mut cb: impl FnMut(&(Arc<Vec<UiView>>, Contact, Option<usize>, usize), usize),
    ) {
        for (idx, contact) in self.contacts.iter().enumerate() {
            let nav_state = self.nav_state.clone();
            cb(&(nav_state, contact.clone(), self.selected, idx), idx);
        }
    }

    fn for_each_mut(
        &mut self,
        mut cb: impl FnMut(&mut (Arc<Vec<UiView>>, Contact, Option<usize>, usize), usize),
    ) {
        let mut any_shared_changed = false;
        for (idx, contact) in self.contacts.iter().enumerate() {
            let mut d = (self.nav_state.clone(), contact.clone(), self.selected, idx);

            cb(&mut d, idx);
            if !any_shared_changed && !self.nav_state.same(&d.0) {
                any_shared_changed = true;
            }
            if any_shared_changed {
                self.nav_state = d.0;
                self.selected = d.2;
            }
        }
    }

    fn data_len(&self) -> usize {
        self.contacts.len()
    }
}

pub fn get_data() -> Vec<Contact> {
    vec![
        Contact {
            name: "Billy Bob".to_string(),
            email: "Billybob@gmail.com".to_string(),
            favorite_food: "Curry".to_string(),
            age: 39,
        },
        Contact {
            name: "Waka waka".to_string(),
            email: "wakaka@gmail.com".to_string(),
            favorite_food: "Fried Rice".to_string(),
            age: 65,
        },
        Contact {
            name: "Chance Rapper".to_string(),
            email: "chancerapper@gmail.com".to_string(),
            favorite_food: "Brussel Sprouts".to_string(),
            age: 22,
        },
        Contact {
            name: "Vincente Fernandez".to_string(),
            email: "VFernandez@gmail.com".to_string(),
            favorite_food: "Rice and Beans".to_string(),
            age: 51,
        },
    ]
}
