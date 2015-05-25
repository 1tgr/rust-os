pub type Test = (&'static str, fn());
pub type Fixture = (&'static str, &'static [Test]);

macro_rules! test {
    ($(fn $name:ident() $block:block)*) => {
        $(
            fn $name() $block
        )*

        pub const TESTS: ::test::Fixture = (module_path!(), &[
            $(
                (stringify!($name), $name),
            )*
        ]);
    }
}
