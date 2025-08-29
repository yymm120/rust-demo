// use crate::{ClassBuilder};

#[macro_export]
macro_rules! __internal_apply_methods_loop {
    ((), $this:expr, {}) => {
        $this
    };

    ((), $this:expr, { . $name:ident ( $($args:expr),*) $($rest:tt)* }) => {{
        let this = $this.$name($($args),*);
        $crate::__internal_apply_methods_loop!((), this, { $(rest)* })
    }};
}

#[macro_export]
macro_rules! apply_methods {
    ($($args:tt)*) => {
        $crate::__internal_apply_methods_loop!((), $($args)*)
    };
}

#[macro_export]
macro_rules! class {
    // (#[prefix = $name:literal] $($methods:tt)*) => {{
    //     $crate::ClassBuilder::__internal_done($crate::apply_methods!($crate::ClassBuilder::__internal_new(Some($name)), { $(methods)* }))
    // }};

    ($($methods:tt)*) => {{
        $crate::ClassBuilder::__internal_done($crate::apply_methods!($crate::ClassBuilder::__internal_new(None), { $($methods)* }))
    }}
}