use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::theme;
use druid::{
    BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, UpdateCtx,
    Widget,
};

/// Wedge is an arbitrary name for the arrow-like icon marking whether a node is expanded or collapsed.
pub struct Wedge;

// Is "Chevron" a better name?
impl Wedge {
    pub fn new() -> Self {
        Wedge {}
    }
}

impl Default for Wedge {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementing Widget for the wedge.
/// This widget's data is simply a boolean telling whether is is expanded or collapsed.
impl Widget<bool> for Wedge {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, expanded: &mut bool, _env: &Env) {
        match event {
            Event::MouseDown(_) => {
                ctx.set_active(true);
                ctx.request_paint();
            }
            Event::MouseUp(_) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    if ctx.is_hot() {
                        *expanded = !*expanded;
                    }
                    ctx.request_paint();
                }
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &bool, _env: &Env) {
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &bool, _data: &bool, _env: &Env) {}

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &bool,
        env: &Env,
    ) -> Size {
        let size = env.get(theme::BASIC_WIDGET_HEIGHT);
        bc.constrain(Size::new(size, size))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, expanded: &bool, env: &Env) {
        let y_offset = ((ctx.size().height - 8.0) / 2.0).floor();
        let stroke_color = if ctx.is_hot() {
            env.get(theme::FOREGROUND_LIGHT)
        } else {
            env.get(theme::FOREGROUND_DARK)
        };

        // Paint the wedge
        let mut path = BezPath::new();
        if *expanded {
            // expanded: 'V' shape
            path.move_to((5.0, y_offset + 2.0));
            path.line_to((9.0, y_offset + 8.0));
            path.line_to((13.0, y_offset + 2.0));
        } else {
            // collapsed: '>' shape
            path.move_to((7.0, y_offset));
            path.line_to((13.0, y_offset + 4.0));
            path.line_to((7.0, y_offset + 8.0));
        }
        let style = StrokeStyle::new()
            .line_cap(LineCap::Round)
            .line_join(LineJoin::Round);

        ctx.stroke_styled(path, &stroke_color, 2.5, &style);
    }
}
