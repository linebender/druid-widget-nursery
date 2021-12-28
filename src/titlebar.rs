use druid::{Data, Event, Widget};

/// A TitleBar widget.
///
/// Handles mouse clicks over itself like a titlebar - you can drag the window around by it.
/// It does prevent some events from being passed into inner, like mouse click.
/// Presumably, this is because clicking means you are now dragging the window,
/// not interacting with it.
///
/// It uses the layout of its inner widget.
///
/// This only works on windows due to it's use of [`handle_titlebar`](<https://docs.rs/druid-shell/0.7.0/druid_shell/struct.WindowHandle.html#method.handle_titlebar>)
///
/// ## Example
/// ```ignore
/// use druid::widget::{Button, Flex, Label};
/// use druid::{AppLauncher, Application, Widget, WidgetExt, WindowDesc};
/// use druid_widget_nursery::TitleBar;
///
/// fn build_titlebar() -> impl Widget<u32> {
///     // Make a row of buttons and title for the titlebar.
///     Flex::row()
///         // Increases the state by one each time clicked.
///         .with_child(Button::new("+1").on_click(|_event, t, _env| *t += 1))
///         // Flex child so it fills up all the space, which means it has to be limited.
///         // This is done by the column in build_ui()
///         .with_flex_child(
///             // New titlebar with a label
///             TitleBar::new(
///                 // label is dynamic, using the value from data
///                 Label::dynamic(|data: &u32, _env| format!("TitleBar: Value = {}", data))
///                 // Center the label in the titlebar.
///                 .center()
///             ), 1.0
///         )
///         // Button to close the window.
///         // There isn't a normal titlebar, so something like this has to exist.
///         .with_child(Button::new("Close").on_click(|_,_,_| Application::global().quit()))
/// }
///
/// fn build_ui() -> impl Widget<u32> {
///     // A column with a titlebar and then whatever body the app has beneath it.
///     Flex::column()
///         .with_child(build_titlebar())
///         .with_child(Label::new(
///             "Whatever is beneath the titlebar.\nYou can also drag the window around by the title! Try it"
///         ))
/// }
///
/// fn main() {
///     let window = WindowDesc::new(build_ui())
///     // Set show_titlebar to false, this hides the normal titlebar.
///         .show_titlebar(false);
///     // Start titlebar value at 0
///     let data = 0u32;
///     // Launch
///     AppLauncher::with_window(window)
///         .launch(data)
///         .expect("Failed to launch");
/// }
/// ```
pub struct TitleBar<T> {
    inner: Box<dyn Widget<T>>,
}

impl<T: Data> TitleBar<T> {
    pub fn new(inner: impl Widget<T> + 'static) -> TitleBar<T> {
        TitleBar {
            inner: Box::new(inner),
        }
    }
}

// Feed all calls through into inner.
impl<T: Data> Widget<T> for TitleBar<T> {
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut T,
        env: &druid::Env,
    ) {
        // On any mousemove event in this tell the window to handle the titlebar.
        if let Event::MouseMove(_) = event {
            ctx.window().handle_titlebar(true);
        }
        self.inner.event(ctx, event, data, env);
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &T,
        env: &druid::Env,
    ) {
        self.inner.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, old_data: &T, data: &T, env: &druid::Env) {
        self.inner.update(ctx, old_data, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &T,
        env: &druid::Env,
    ) -> druid::Size {
        self.inner.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &T, env: &druid::Env) {
        self.inner.paint(ctx, data, env);
    }
}
