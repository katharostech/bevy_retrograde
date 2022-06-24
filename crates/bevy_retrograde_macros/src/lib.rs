//! Macros used in Bevy Retrograde

/// Utility to implement deref for single-element tuple structs
///
/// This is particularly useful when you need to create a Bevy
///
/// # Example
///
/// ```no-test
/// # use bevy_retrograde_macros::impl_deref;
///
/// #[derive(Component)]
/// struct Score(usize);
///
/// impl_deref!(Score, usize);
/// ```
#[macro_export]
macro_rules! impl_deref {
    ($struct:ident, $target:path) => {
        impl std::ops::Deref for $struct {
            type Target = $target;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $struct {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

/// Utility macro for adding an attribute to a batch of items
///
/// # Example
///
/// ```
/// # use bevy_retrograde_macros::items_attr;
/// // Only import these libraries for wasm targets
/// items_attr!(cfg(wasm), {
///     use web_sys;
///     use js_sys;
/// });
/// ```
#[macro_export]
macro_rules! items_attr {
    ($attr:ident($meta:meta), {
        $(
            $item:item
        )*
    }) => {
        $(
            #[$attr($meta)]
            $item
        )*
    };
}
