pub use druid_material_icons::{normal, IconPaths};

use druid::{
    kurbo::{Affine, Size},
    widget::prelude::*,
    Color, KeyOrValue,
};

/// A widget that draws one of the material icons.
///
/// # Examples
///
/// ```
/// # use druid::Color;
/// use druid_widget_nursery::material_icons::{Icon, normal::action::ALARM_ADD};
/// let icon = Icon::new(ALARM_ADD)
///     // optional - defaults to text color
///     .with_color(Color::WHITE);
/// // use `icon` as you would any widget...
/// ```
#[derive(Debug, Clone)]
pub struct Icon {
    paths: IconPaths,
    color: KeyOrValue<Color>,
}

impl Icon {
    #[inline]
    pub fn new(paths: IconPaths) -> Self {
        Self {
            paths,
            color: KeyOrValue::from(druid::theme::TEXT_COLOR),
        }
    }

    pub fn with_color(mut self, color: impl Into<KeyOrValue<Color>>) -> Self {
        self.color = color.into();
        self
    }
}

impl<T: Data> Widget<T> for Icon {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut T, _env: &Env) {
        // no events
    }
    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &T, _env: &Env) {
        // no lifecycle
    }
    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &T, _data: &T, _env: &Env) {
        // no update
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &T, _env: &Env) -> Size {
        // Try to preserve aspect ratio if possible, but if not then allow non-uniform scaling.
        bc.constrain_aspect_ratio(self.paths.size.aspect_ratio(), self.paths.size.width)
    }
    fn paint(&mut self, ctx: &mut PaintCtx, _data: &T, env: &Env) {
        let color = self.color.resolve(env);
        let Size { width, height } = ctx.size();
        let Size {
            width: icon_width,
            height: icon_height,
        } = self.paths.size;
        ctx.transform(Affine::scale_non_uniform(
            width * icon_width.recip(),
            height * icon_height.recip(),
        ));
        for path in self.paths.paths {
            ctx.fill(path, &color);
        }
    }
}
