use super::*;

/// An animator. This keeps track of multiple running animations, and the dependencies between
/// animations and events.
#[derive(Default, Debug)]
pub struct Animator {
    cur_nanos: Nanos,
    pending_count: u32,
    pending_starts: HashMap<AnimationEvent, Vec<AnimationId>>,
    pub(in crate::animation) storage: AnimationStorage<AnimationState>,
}

impl Animator {
    pub(in crate::animation) fn current_time(&self) -> Nanos {
        self.cur_nanos
    }

    /// Advance the state of all running animations by the given number of nanoseconds.
    pub fn advance_by<V>(
        &mut self,
        nanos: Nanos,
        mut f: impl FnMut(&AnimationCtx) -> V,
    ) -> Option<V> {
        if self.storage.is_empty() {
            log::info!("Empty animator");
            None
        } else {
            self.cur_nanos += nanos;

            // Possibly this should be a small vec,
            // as usually not many events will be produced.
            // For now it is only Ended events that get produced.
            let mut pending_events = VecDeque::new();

            let res = {
                let cur_nanos = self.cur_nanos;

                self.storage.remove_if(|id, segment| {
                    let remove = segment.advance(cur_nanos);
                    if remove {
                        pending_events.push_back(AnimationEvent::Ended(id));
                    }
                    remove
                });

                let ctx = AnimationCtx::new(None, &self.storage, false);
                f(&ctx)
            };

            for event in pending_events.into_iter() {
                self.process_event(event)
            }

            if self.storage.is_empty() {
                self.cur_nanos = 0.;
            }
            Some(res)
        }
    }

    /// Process a named event.
    /// This can be used to trigger animations configured elsewhere.
    pub fn process_named_event(&mut self, name: AnimationEventName) {
        self.process_event(AnimationEvent::Named(name))
    }

    fn process_event(&mut self, event: AnimationEvent) {
        if let Some(ids) = self.pending_starts.remove(&event) {
            for id in ids {
                if let Some(seg) = self.storage.get_mut(id) {
                    if seg.start_pending(self.cur_nanos) {
                        self.pending_count -= 1;
                    }
                }
            }
        };
    }

    pub(in crate::animation) fn register_pending(
        &mut self,
        event: AnimationEvent,
        id: AnimationId,
    ) {
        self.pending_starts
            .entry(event)
            .or_insert_with(Vec::new)
            .push(id);
        self.pending_count += 1;
    }

    /// Is the animator running?
    pub fn running(&self) -> bool {
        // TODO: If we had waiting ones we could return a minimum time until one had to start
        // Could maintain a max wait time
        (self.storage.size() - self.pending_count) > 0
    }

    /// Are there any running or pending animations?
    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    /// Create a new animation and return a handle to allow its configuration.
    pub fn new_animation(&mut self) -> AnimationHandle {
        let id = self
            .storage
            .insert(AnimationState::new(AnimationStatusInternal::Waiting(
                self.cur_nanos,
            )));
        AnimationHandle::new(id, self)
    }

    /// Get a handle to an animation id. Note - this handle is not guaranteed to be valid.
    pub fn get(&mut self, id: AnimationId) -> AnimationHandle {
        AnimationHandle::new(id, self)
    }
}
