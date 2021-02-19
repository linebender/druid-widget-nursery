use crate::animation::*;

#[derive(Clone, Debug, PartialEq)]
pub(in crate::animation) enum AnimationStatusInternal {
    PendingEvent(DelayNanos), // delay after ready
    Waiting(StartNanos),
    Enlisting(StartNanos),
    Running(StartNanos),
    Repeating(StartNanos), // Start of current repetition
    Retiring,
}

impl AnimationStatusInternal {
    fn is_active(&self) -> bool {
        matches!(
            self,
            AnimationStatusInternal::Enlisting(_)
                | AnimationStatusInternal::Running(_)
                | AnimationStatusInternal::Retiring
        )
    }

    pub(crate) fn add_delay(&self, cur_nanos: f64, delay_nanos: f64, duration: f64) -> Self {
        match self {
            AnimationStatusInternal::PendingEvent(delay) => {
                AnimationStatusInternal::PendingEvent(delay + delay_nanos)
            }
            AnimationStatusInternal::Waiting(start) => {
                let start = start + delay_nanos;

                if cur_nanos > start + duration {
                    // Skip entirely?
                    AnimationStatusInternal::Retiring
                } else {
                    AnimationStatusInternal::Waiting(start)
                }
            }
            AnimationStatusInternal::Enlisting(start)
            | AnimationStatusInternal::Repeating(start)
            | AnimationStatusInternal::Running(start) => {
                let start = start + delay_nanos;

                if start > cur_nanos {
                    AnimationStatusInternal::Running(start)
                } else {
                    // Could enlist twice - would need to have a WaitingEnlisted state to prevent
                    AnimationStatusInternal::Waiting(start)
                }
            }
            AnimationStatusInternal::Retiring => AnimationStatusInternal::Retiring,
            // Does this need to have a pre-retiring state to make sure interps run once
            // (to do their retirement actions)
        }
    }

    fn pending(&self, cur_nanos: f64) -> Self {
        match self {
            AnimationStatusInternal::Waiting(start)
            | AnimationStatusInternal::Enlisting(start)
            | AnimationStatusInternal::Running(start) => {
                AnimationStatusInternal::PendingEvent((cur_nanos - start).min(0.))
            }
            other => other.clone(),
        }
    }
}

#[derive(Debug)]
pub(in crate::animation) struct AnimationState {
    pub(in crate::animation) duration: Nanos,
    curve: AnimationCurve,
    direction: AnimationDirection,
    repeat_limit: Option<usize>,
    pub(in crate::animation) status: AnimationStatusInternal,
    since_start: Nanos,
    fraction: f64,
    pub(in crate::animation) progress: f64,
    repeat_count: usize,
}

impl AnimationState {
    pub(in crate::animation) fn is_active(&self) -> bool {
        self.status.is_active()
    }

    pub(in crate::animation) fn start_pending(&mut self, cur_nanos: Nanos) -> bool {
        if let AnimationStatusInternal::PendingEvent(delay) = &self.status {
            self.status = AnimationStatusInternal::Waiting(cur_nanos + delay);
            true
        } else {
            false
        }
    }
    pub(in crate::animation) fn change_status(&mut self, f: impl Fn(&mut AnimationStatusInternal)) {
        f(&mut self.status);
    }

    pub(in crate::animation) fn new(status: AnimationStatusInternal) -> Self {
        AnimationState {
            duration: 1.,
            curve: Default::default(),
            direction: Default::default(),
            repeat_limit: Some(1),
            status,
            since_start: 0.,
            fraction: 0.,
            progress: 0.,
            repeat_count: 0,
        }
    }

    pub(in crate::animation) fn calc(&mut self, cur_nanos: Nanos) {
        let before_end = self.since_start < self.duration; // Ask curve (e.g non duration based)

        let even_repeat = self.repeat_count % 2 == 0;

        if before_end {
            self.fraction = self
                .direction
                .translate(self.since_start / self.duration, even_repeat);
            self.progress = self.curve.translate(self.fraction);
        } else {
            // This animation will go through one more cycle to give users
            // a chance to recover from any discontinuous curves - i.e set things to the end state.

            self.repeat_count += 1;
            let allow_repeat = self
                .repeat_limit
                .map_or(true, |limit| self.repeat_count < limit);
            if allow_repeat {
                self.status = AnimationStatusInternal::Repeating(cur_nanos);
            } else {
                let end_fraction = self.direction.end_fraction(even_repeat);
                self.fraction = end_fraction;
                self.progress = end_fraction;
                self.status = AnimationStatusInternal::Retiring;
            }
        }
    }

    pub(in crate::animation) fn advance(&mut self, cur_nanos: f64) -> bool {
        use AnimationStatusInternal::*;
        match self.status.clone() {
            Waiting(start) => {
                self.since_start = cur_nanos - start;
                if self.since_start > 0. {
                    self.status = AnimationStatusInternal::Enlisting(start);
                    // TODO priming state for first run
                    self.calc(cur_nanos);
                }
                false
            }
            Enlisting(start) | Repeating(start) => {
                self.since_start = cur_nanos - start;
                self.status = AnimationStatusInternal::Running(start);
                self.calc(cur_nanos);
                false
            }
            Running(start) => {
                self.since_start = cur_nanos - start;
                self.calc(cur_nanos);
                false
            }
            Retiring => {
                log::info!("Retired anim {:?} ", self);
                true
            }
            PendingEvent(_) => false,
        }
    }

    pub(in crate::animation) fn status(&self) -> AnimationStatus {
        match self.status {
            AnimationStatusInternal::PendingEvent(_) => AnimationStatus::NotRunning,
            AnimationStatusInternal::Waiting(_) => AnimationStatus::NotRunning,
            AnimationStatusInternal::Enlisting(_) => AnimationStatus::Enlisting,
            AnimationStatusInternal::Repeating(_) => AnimationStatus::Repeating,
            AnimationStatusInternal::Running(_) => AnimationStatus::Running,
            AnimationStatusInternal::Retiring => AnimationStatus::Retiring,
        }
    }
}

/// A handle to an animation to allow configuring and controlling it.
pub struct AnimationHandle<'a> {
    id: AnimationId,
    animator: &'a mut Animator,
}

impl AnimationHandle<'_> {
    pub(in crate::animation) fn new(id: AnimationId, animator: &mut Animator) -> AnimationHandle {
        AnimationHandle { id, animator }
    }

    fn change_animation_state(self, f: impl FnOnce(&mut AnimationState)) -> Self {
        self.animator
            .storage
            .get_mut(self.id)
            .map(f)
            .unwrap_or_else(|| log::warn!("Attempt to modify retired segment {:?}", self.id));
        self
    }

    /// Set the delay before this animation starts.
    pub fn delay(self, delay: impl Into<Duration>) -> Self {
        let cur_nanos = self.animator.current_time();
        let delay_nanos = delay.into().as_nanos() as f64;
        self.change_animation_state(|seg| {
            let duration = seg.duration;
            seg.change_status(|status| {
                *status = status.add_delay(cur_nanos, delay_nanos, duration)
            });
        })
    }

    /// Set the duration of this animation
    pub fn duration(self, duration: impl Into<Duration>) -> Self {
        self.change_animation_state(|seg| seg.duration = duration.into().as_nanos() as f64)
    }

    /// Set the direction of this animation
    pub fn direction(self, direction: impl Into<AnimationDirection>) -> Self {
        let dir = direction.into();
        self.change_animation_state(|state| state.direction = dir)
    }

    /// Set how many times this animation should repeat.
    pub fn repeat_limit(self, limit: impl Into<Option<usize>>) -> Self {
        let limit = limit.into();
        self.change_animation_state(|state| state.repeat_limit = limit)
    }

    /// Set the animation curve
    pub fn curve(self, curve: impl Into<AnimationCurve>) -> Self {
        let curve = curve.into();
        self.change_animation_state(|seg| seg.curve = curve)
    }

    /// Set the event that this animation will run after.
    pub fn after(self, event: impl Into<AnimationEvent>) -> Self {
        self.animator.register_pending(event.into(), self.id);
        let cur_nanos = self.animator.current_time();

        self.change_animation_state(|seg| seg.status = seg.status.pending(cur_nanos))
    }

    /// The animation id
    pub fn id(&self) -> AnimationId {
        self.id
    }

    /// Is this handle pointing to an animation that exists within the animator
    pub fn is_valid(&self) -> bool {
        self.animator.storage.contains(self.id)
    }

    /// Get the status of this animation. If it is invalid, the status will be NotRunning
    pub fn status(&self) -> AnimationStatus {
        self.animator
            .storage
            .get(self.id)
            .map_or(AnimationStatus::NotRunning, |state| state.status())
    }
}
