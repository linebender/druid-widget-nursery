use std::time::Duration;

use druid::piet::PietText;
use druid::widget::prelude::*;
use druid::{
    Command, Cursor, ExtEventSink, Point, Rect, TimerToken, WindowConfig, WindowHandle, WindowId,
};

macro_rules! impl_context_trait{
    ($tr:ty => $ty:ty,  { $($method:item)+ } ) => {
        impl $tr for $ty { $($method)+ }
    };
    ($tr:ty => $ty:ty, $($more:ty),+, { $($method:item)+ } ) => {
        impl_context_trait!($tr => $ty, { $($method)+ });
        impl_context_trait!($tr => $($more),+, { $($method)+ });
    };
}

/// Convenience trait for methods available on all contexts.
pub trait AnyCtx {
    /// get the `WidgetId` of the current widget.
    fn widget_id(&self) -> WidgetId;

    /// Returns a reference to the current `WindowHandle`.
    fn window(&self) -> &WindowHandle;

    /// Get the `WindowId` of the current window.
    fn window_id(&self) -> WindowId;

    /// Get an object which can create text layouts.
    fn text(&mut self) -> &mut PietText;
}

impl_context_trait!(
   AnyCtx => EventCtx<'_, '_>, UpdateCtx<'_, '_>, LifeCycleCtx<'_, '_>,PaintCtx<'_, '_, '_>, LayoutCtx<'_, '_>,
   {
        fn widget_id(&self) -> WidgetId {
            Self::widget_id(self)
        }

        fn window(&self) -> &WindowHandle {
            Self::window(self)
        }

        fn window_id(&self) -> WindowId {
            Self::window_id(self)
        }

        fn text(&mut self) -> &mut PietText {
            Self::text(self)
        }
   }
);

/// Convenience trait for methods related to geometry and window position, available after [`layout`].
///
/// These methods are available on [`EventCtx`], [`LifeCycleCtx`], [`UpdateCtx`], and [`PaintCtx`].
///
/// [`layout`]: Widget::layout
pub trait LaidOutCtx {
    /// The layout size. See [`size`].
    ///
    /// [`size`]: EventCtx::size
    fn size(&self) -> Size;
    /// The origin of the widget in window coordinates. See [`window_origin`].
    ///
    /// [`window_origin`]: EventCtx::window_origin
    fn window_origin(&self) -> Point;
    /// Convert a point from the widget's coordinate space to the window's. See [`to_window`].
    ///
    /// [`to_window`]: EventCtx::to_window
    fn to_window(&self, widget_point: Point) -> Point;
    /// Convert a point from the widget's coordinate space to the screen's. See [`to_screen`].
    ///
    /// [`to_screen`]: EventCtx::to_screen
    fn to_screen(&self, widget_point: Point) -> Point;
    /// The "hot" status of a widget. See [`is_hot`].
    ///
    /// [`is_hot`]: EventCtx::is_hot
    fn is_hot(&self) -> bool;
    /// The active status of a widget. See [`is_active`]
    ///
    /// [`is_active`]: EventCtx::is_active
    fn is_active(&self) -> bool;
    /// The focus status of a widget. See [`is_focused`].
    ///
    /// [`is_focused`]: EventCtx::is_focused
    fn is_focused(&self) -> bool;
    /// The focus status of a widget or any of its descendents. See [`has_focus`].
    ///
    /// [`has_focus`]: EventCtx::has_focus
    fn has_focus(&self) -> bool;
}

impl_context_trait!(
   LaidOutCtx => EventCtx<'_, '_>, UpdateCtx<'_, '_>, LifeCycleCtx<'_, '_>,PaintCtx<'_, '_, '_>,
   {
        fn size(&self) -> Size{
            Self::size(self)
        }

        fn window_origin(&self) -> Point{
            Self::window_origin(self)
        }

        fn to_window(&self, widget_point: Point) -> Point{
            Self::to_window(self, widget_point)
        }

        fn to_screen(&self, widget_point: Point) -> Point{
            Self::to_screen(self, widget_point)
        }

        fn is_hot(&self) -> bool{
            Self::is_hot(self)
        }

        fn is_active(&self) -> bool{
            Self::is_active(self)
        }

        fn is_focused(&self) -> bool{
            Self::is_focused(self)
        }

        fn has_focus(&self) -> bool{
            Self::has_focus(self)
        }
   }
);

/// Convenience trait for cursor manipulation methods available on multiple contexts.
///
/// Available on [`EventCtx`] and [`UpdateCtx`].
pub trait CursorCtx {
    /// Set the cursor icon. See [`set_cursor`].
    ///
    /// ['set_cursor']: EventCtx::set_cursor
    fn set_cursor(&mut self, cursor: &Cursor);
    /// Override the cursor icon. See [`override_cursor`].
    ///
    /// [`override_cursor`]: EventCtx::override_cursor
    fn override_cursor(&mut self, cursor: &Cursor);
    /// Clear the cursor icon. See [`clear_cursor`]
    ///
    /// [`clear_cursor`]: EventCtx::clear_cursor
    fn clear_cursor(&mut self);
}

impl_context_trait!(
    CursorCtx => EventCtx<'_, '_>, UpdateCtx<'_, '_>,
    {
        fn set_cursor(&mut self, cursor: &Cursor){
            Self::set_cursor(self, cursor)
        }
        fn override_cursor(&mut self, cursor: &Cursor){
            Self::override_cursor(self, cursor)
        }
        fn clear_cursor(&mut self){
            Self::clear_cursor(self)
        }
    }
);

/// Convenience trait for invalidation and request methods available on multiple contexts.
///
/// These methods are available on [`EventCtx`], [`LifeCycleCtx`], and [`UpdateCtx`].
pub trait RequestCtx {
    /// Request a [`paint`] pass. See ['request_paint']
    ///
    /// ['request_paint']: EventCtx::request_paint
    fn request_paint(&mut self);
    /// Request a [`paint`] pass for redrawing a rectangle. See [`request_paint_rect`].
    ///
    /// [`request_paint_rect`]: EventCtx::request_paint_rect
    /// [`paint`]: Widget::paint
    fn request_paint_rect(&mut self, rect: Rect);
    /// Request a layout pass. See [`request_layout`].
    ///
    /// [`request_layout`]: EventCtx::request_layout
    fn request_layout(&mut self);
    /// Request an animation frame. See [`request_anim_frame`].
    ///
    /// [`request_anim_frame`]: EventCtx::request_anim_frame
    fn request_anim_frame(&mut self);
    /// Indicate that your children have changed. See [`children_changed`].
    ///
    /// [`children_changed`]: EventCtx::children_changed
    fn children_changed(&mut self);
    /// Create a new sub-window. See [`new_sub_window`].
    ///
    /// [`new_sub_window`]: EventCtx::new_sub_window
    fn new_sub_window<W: Widget<U> + 'static, U: Data>(
        &mut self,
        window_config: WindowConfig,
        widget: W,
        data: U,
        env: Env,
    ) -> WindowId;
}

impl_context_trait!(
    RequestCtx => EventCtx<'_, '_>, UpdateCtx<'_, '_>, LifeCycleCtx<'_, '_>,
    {
        fn request_paint(&mut self){
            Self::request_paint(self)
        }
        fn request_paint_rect(&mut self, rect: Rect){
            Self::request_paint_rect(self, rect)
        }
        fn request_layout(&mut self){
            Self::request_layout(self)
        }
        fn request_anim_frame(&mut self){
            Self::request_anim_frame(self)
        }
        fn children_changed(&mut self){
            Self::children_changed(self)
        }
        fn new_sub_window<W: Widget<U> + 'static, U: Data>(
            &mut self,
            window_config: WindowConfig,
            widget: W,
            data: U,
            env: Env,
        ) -> WindowId{
            Self::new_sub_window(self, window_config, widget, data, env)
        }
    }
);

/// Convenience trait for code generic over contexts.
///
/// Methods to do with commands and timers.
/// Available to all contexts but PaintCtx.
pub trait CommandCtx {
    /// Submit a [`Command`] to be run after this event is handled. See [`submit_command`].
    ///
    /// [`submit_command`]: EventCtx::submit_command
    fn submit_command(&mut self, cmd: impl Into<Command>);
    /// Returns an [`ExtEventSink`] for submitting commands from other threads. See ['get_external_handle'].
    ///
    /// [`get_external_handle`]: EventCtx::get_external_handle
    fn get_external_handle(&self) -> ExtEventSink;
    /// Request a timer event. See [`request_timer`]
    ///
    /// [`request_timer`]: EventCtx::request_timer
    fn request_timer(&mut self, deadline: Duration) -> TimerToken;
}

impl_context_trait!(
    CommandCtx => EventCtx<'_, '_>, UpdateCtx<'_, '_>, LifeCycleCtx<'_, '_>, LayoutCtx<'_, '_>,
    {

        fn submit_command(&mut self, cmd: impl Into<Command>) {
            Self::submit_command(self, cmd)
        }

        fn get_external_handle(&self) -> ExtEventSink {
            Self::get_external_handle(self)
        }

        fn request_timer(&mut self, deadline: Duration) -> TimerToken {
            Self::request_timer(self, deadline)
        }
    }
);
