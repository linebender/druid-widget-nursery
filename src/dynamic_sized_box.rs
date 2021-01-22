use druid::widget::prelude::*;
use druid::Data;

/// A widget that changes size dynamically; the dynamic analogue to [`SizedBox`].
///
/// If given a child, this widget forces the child to have a variable width and/or height.
///
/// If not given a child, The box will try to size itself as a fraction of the parent's
/// box constraints. If height or width is not set, it will be treated as zero.
///
/// [`SizedBox`]: druid::widget::SizedBox
pub struct DynamicSizedBox<T> {
    inner: Option<Box<dyn Widget<T>>>,
    width: Option<f64>,
    height: Option<f64>,
}

impl<T> DynamicSizedBox<T> {
    /// Create container with child, and both width and height not set.
    pub fn new(inner: impl Widget<T> + 'static) -> Self {
        Self {
            inner: Some(Box::new(inner)),
            width: None,
            height: None,
        }
    }

    /// Create container without child, and both width and height not set.
    pub fn empty() -> Self {
        Self {
            inner: None,
            width: None,
            height: None,
        }
    }

    /// Builder-style method for setting the fractional width.
    ///
    /// The width has to be a value between 0 and 1. It will be clamped
    /// to those values if they exceed the bounds.
    pub fn with_width(mut self, width: f64) -> Self {
        // clamp between 0 and 1
        // until clamp function stabilizes for f64
        self.width = Some(width.max(0.0).min(1.0));
        self
    }

    /// Builder-style method for setting the fractional height.
    ///
    /// The height has to be a value between 0 and 1. It will be clamped
    /// to those values if they exceed the bounds.
    pub fn with_height(mut self, height: f64) -> Self {
        self.height = Some(height.max(0.0).min(1.0));
        self
    }

    /// Set the fractional width of the dynamic box.
    ///
    /// The width has to be a value between 0 and 1. It will be clamped
    /// to those values if they exceed the bounds.
    pub fn set_width(&mut self, width: f64) {
        self.width = Some(width.max(0.0).min(1.0));
    }

    /// Set the fractional height of the dynamic box.
    ///
    /// The height has to be a value between 0 and 1. It will be clamped
    /// to those values if they exceed the bounds.
    pub fn set_height(&mut self, height: f64) {
        self.height = Some(height.max(0.0).min(1.0));
    }

    /// Expand container to fit the parent
    ///
    /// Only call this method if you want your widget to occupy all available
    /// space. If you only care about expanding in one of width or height, use
    /// [`expand_width`] or [`expand_height`] instead.
    ///
    /// [`expand_width`]: crate::DynamicSizedBox::expand_width
    /// [`expand_height`]: crate::DynamicSizedBox::expand_height
    pub fn expand(mut self) -> Self {
        self.width = Some(1.0);
        self.height = Some(1.0);
        self
    }

    /// Expand the container on the x-axis.
    ///
    /// This will force the child to have maximum width.
    pub fn expand_width(mut self) -> Self {
        self.width = Some(1.0);
        self
    }

    /// Expand the container on the y-axis.
    ///
    /// This will force the child to have maximum height.
    pub fn expand_height(mut self) -> Self {
        self.height = Some(1.0);
        self
    }

    /// Determine the constraints that will be used for inner widget.
    fn inner_constraints(&self, bc: &BoxConstraints) -> BoxConstraints {
        // if we have a width/height, multiply it by bc.max to get new width/height
        // of widget and clamp on that value
        // if we don't have width/height, box constraints stay the same
        let (min_width, max_width) = match self.width {
            Some(width) => {
                let w = width * bc.max().width;
                (w, w)
            }
            None => (bc.min().width, bc.max().width),
        };

        let (min_height, max_height) = match self.height {
            Some(height) => {
                let h = height * bc.max().height;
                (h, h)
            }
            None => (bc.min().height, bc.max().height),
        };

        BoxConstraints::new(
            Size::new(min_width, min_height),
            Size::new(max_width, max_height),
        )
    }
}

impl<T: Data> Widget<T> for DynamicSizedBox<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Some(ref mut inner) = self.inner {
            inner.event(ctx, event, data, env);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let Some(ref mut inner) = self.inner {
            inner.lifecycle(ctx, event, data, env)
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        if let Some(ref mut inner) = self.inner {
            inner.update(ctx, old_data, data, env);
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        bc.debug_check("DynamicSizedBox");

        let bc = bc.loosen();

        let inner_bc = self.inner_constraints(&bc);
        let size = match self.inner.as_mut() {
            Some(inner) => inner.layout(ctx, &inner_bc, data, env),
            None => bc.constrain((
                self.width.unwrap_or(0.0) * bc.max().width,
                self.height.unwrap_or(0.0) * bc.max().height,
            )),
        };

        if size.width.is_infinite() {
            log::warn!("DynamicSizedBox is returning an infinite width.");
        }

        if size.height.is_infinite() {
            log::warn!("DynamicSizedBox is returning an infinite height.");
        }

        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        if let Some(ref mut inner) = self.inner {
            inner.paint(ctx, data, env);
        }
    }

    fn id(&self) -> Option<WidgetId> {
        self.inner.as_ref().and_then(|inner| inner.id())
    }
}
