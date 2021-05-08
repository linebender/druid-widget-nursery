use crate::animation::state::AnimationState;
use crate::animation::*;

#[derive(Debug)]
/// An animation context.
/// This provides information about the progress of currently running animations.
/// It can be focused on one particular animation at a time, and can be refocused using an AnimationId.
/// This is useful for descending an animated data structure without every level needing to know about animation ids.
pub struct AnimationCtx<'a> {
    content: AnimationCtxContent<'a>,
}

#[derive(Debug)]
struct AnimationCtxFull<'a> {
    focus: Option<AnimationId>,
    additive: bool,
    animations: &'a Animations,
}

impl AnimationCtxFull<'_> {
    fn with_focused<V>(&self, f: impl Fn(&AnimationState) -> V) -> Option<V> {
        self.focus
            .and_then(|focus| self.animations.get(focus))
            .map(f)
    }
}

#[derive(Debug)]
enum AnimationCtxContent<'a> {
    Full(AnimationCtxFull<'a>),
    Immediate(f64, AnimationStatus, bool),
}

impl AnimationCtx<'_> {
    pub(in crate::animation) fn new(
        focus: Option<AnimationId>,
        animations: &Animations,
        additive: bool,
    ) -> AnimationCtx {
        match focus {
            Some(id) if !animations.contains(id) => {
                panic!("animation segment out of range {:?} {:?}", id, animations)
            }
            _ => AnimationCtx {
                content: AnimationCtxContent::Full(AnimationCtxFull {
                    focus,
                    additive,
                    animations,
                }),
            },
        }
    }

    /// Make a light weight context representing a single running animation at the specified fraction.
    pub fn running(frac: f64) -> AnimationCtx<'static> {
        AnimationCtx {
            content: AnimationCtxContent::Immediate(frac, AnimationStatus::Running, false),
        }
    }

    /// What is the progress of the currently focused animation.
    /// If nothing is focused the progress will be 0.0
    pub fn progress(&self) -> f64 {
        match &self.content {
            AnimationCtxContent::Full(full) => full.with_focused(|seg| seg.progress).unwrap_or(0.),
            AnimationCtxContent::Immediate(progress, ..) => *progress,
        }
    }

    /// What is the progress of the currently focused animation clamped to the unit interval.
    /// If nothing is focused the progress will be 0.0
    pub fn clamped(&self) -> f64 {
        clamp_fraction(self.progress())
    }

    pub fn additive(&self) -> bool {
        match &self.content {
            AnimationCtxContent::Full(full) => full.additive,
            AnimationCtxContent::Immediate(_, _, additive) => *additive,
        }
    }

    /// Return the status of the focused animation. If nothing is Focused it will be NotRunning
    pub fn status(&self) -> AnimationStatus {
        match &self.content {
            AnimationCtxContent::Full(full) => full
                .with_focused(|seg| seg.status())
                .unwrap_or(AnimationStatus::NotRunning),
            AnimationCtxContent::Immediate(_, status, _) => *status,
        }
    }

    /// Focus on a particular animation id, if present.
    pub fn with_animation<V>(
        &self,
        id: impl Into<Option<AnimationId>>,
        f: impl FnMut(&AnimationCtx) -> V,
    ) -> Option<V> {
        self.with_animation_full(id, false, f)
    }

    /// Focus on a particular animation id, if present.
    pub fn with_animation_full<V>(
        &self,
        id: impl Into<Option<AnimationId>>,
        additive: bool,
        mut f: impl FnMut(&AnimationCtx) -> V,
    ) -> Option<V> {
        let id_opt = id.into();
        match &self.content {
            AnimationCtxContent::Full(AnimationCtxFull { animations, .. })
                if id_opt
                    .and_then(|ai| animations.get(ai).map(|s| s.is_active()))
                    .unwrap_or(false) =>
            {
                Some(f(&Self::new(id_opt, animations, additive)))
            }
            _ => None,
        }
    }
}
