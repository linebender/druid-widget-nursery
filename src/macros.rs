/// Generates `Selector`s based on module, line and column
/// ```
/// # use druid_widget_nursery::selectors;
/// selectors! {
///     /// Foo the baz
///     FOO,
///     /// Bar the qux yay much
///     BAR: usize,
/// }
/// ```
/// expands to
/// ```
/// # use druid::Selector;
/// /// Foo the baz
/// pub const FOO: Selector = Selector::new("path::to::module::FOO@0:0");
/// /// Bar the qux yay much
/// pub const BAR: Selector<usize> = Selector::new("path::to::module::BAR@0:0");
/// ```
#[macro_export]
macro_rules! selectors {
    (
        $(
            $(#[$attr:meta])*
            $name:ident $( : $ty:ty)?
        ),* $(,)?
    ) => {
        $(
            $(#[$attr])*
            pub const $name: ::druid::Selector<$($ty)?> = ::druid::Selector::new(concat!(
                module_path!(),
                "::",
                stringify!($name),
                "@",
                line!(),
                ":",
                column!()
            ));
        )*
    };
}

/// Generates `Key`s based on module, line and column
/// ```
/// # use druid_widget_nursery::keys;
/// keys! {
///     /// height of the bar
///     BAR: usize,
/// }
/// ```
/// expands to
/// ```
/// # use druid::Key;
/// /// height of the bar
/// pub const BAR: Key<usize> = Key::new("path::to::module::BAR@0:0");
/// ```
#[macro_export]
macro_rules! keys {
    (
        $(
            $(#[$attr:meta])*
            $name:ident : $ty:ty
        ),* $(,)?
    ) => {
        $(
            $(#[$attr])*
            pub const $name: ::druid::Key<$ty> = ::druid::Key::new(concat!(
                module_path!(),
                "::",
                stringify!($name),
                "@",
                line!(),
                ":",
                column!()
            ));
        )*
    };
}

/// Matches on a command or notification.
///
/// # Example
///
/// ```
/// use druid_widget_nursery::{selectors, match_command};
/// selectors!(FOO: i32, BAR);
/// # let cmd = FOO.with(0);
/// match_command!(cmd => {
///     FOO(i) => {
///         // do something
///     },
///     BAR => { todo!() },
/// });
/// ```
#[macro_export]
macro_rules! match_command {
    ($val:expr => {$($selector:ident $(($bind:ident))? => $body:expr),+ $(,)? }) => {
        match $val {
            val => match () {
                $(
                    // TODO: druid should expose the raw selector so we can match it directly
                    () if val.is($selector) => {
                        $(let $bind = val.get_unchecked($selector);)?
                        $body
                    }
                )+
                    _ => {}
            }
        }
    };
}
