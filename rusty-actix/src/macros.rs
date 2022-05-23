// #[macro_export]
// The macro_use attribute has two purposes. First, it can be used to make a module's
// macro scope not end when the module is closed, by applying it to a module:
// So in essence we can either use #[macro_use] on this module which extrends scope to crate root
// or #[macro_export] which also does the same. In addition to that #[macro_export] also allows
// other crates to import and use your macro instead of using crate::{macro_name}
// See rocket macros for instance
#[allow(unused)]
macro_rules! person {
    ($name:expr, $num:expr) => {{
        Person {
            name: String::from($name),
            number: String::from($num),
            ..Default::default()
        }
    }};
}

