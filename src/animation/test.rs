use super::*;

#[test]
fn test_animator() {
    let mut animator: Animator = Default::default();

    let ai_0 = animator
        .new_animation()
        .duration(Duration::from_nanos(100))
        .id();

    let ai_1 = animator
        .new_animation()
        .duration(Duration::from_nanos(100))
        .after(AnimationEvent::Ended(ai_0))
        .id();

    assert_eq!(
        AnimationStatusInternal::PendingEvent(0.),
        animator.storage.get(ai_1).unwrap().status
    );

    let advance = |animator: &mut Animator, nanos: f64| -> (Option<f64>, Option<f64>) {
        let res = animator.advance_by(nanos, |ctx| {
            (
                ctx.with_animation(ai_0, |ctx| ctx.progress()),
                ctx.with_animation(ai_1, |ctx| ctx.progress()),
            )
        });
        res.unwrap()
    };

    assert_eq!((Some(0.5), None), advance(&mut animator, 50.0));

    assert_eq!(
        AnimationStatusInternal::PendingEvent(0.),
        animator.storage.get(ai_1).unwrap().status
    );

    // Advance just beyond the first animations end.
    // It will be retiring (and forced to 1.0)
    // The second will still be waiting
    assert_eq!((Some(1.0), None), advance(&mut animator, 50.1));

    assert_eq!(
        AnimationStatusInternal::Retiring,
        animator.storage.get(ai_0).unwrap().status
    );
    assert_eq!(
        AnimationStatusInternal::PendingEvent(0.),
        animator.storage.get(ai_1).unwrap().status
    );

    advance(&mut animator, 1.);
    // Second animation is now
    assert_eq!(
        AnimationStatusInternal::Waiting(101.1),
        animator.storage.get(ai_1).unwrap().status
    );

    assert_eq!((None, Some(0.1)), advance(&mut animator, 10.));
}

// Curves
// Events
// Loops
// Removal
