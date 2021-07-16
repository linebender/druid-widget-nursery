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
            $(#[$inner:ident $($args:tt)*])*
            $name:ident $( : $ty:ty)?
        ),* $(,)?
    ) => {
        $(
            $(#[$inner $($args)*])*
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
            $(#[$inner:ident $($args:tt)*])*
            $name:ident : $ty:ty
        ),* $(,)?
    ) => {
        $(
            $(#[$inner $($args)*])*
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
