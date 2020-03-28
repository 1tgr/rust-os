pub type Test = (&'static str, fn());
pub type Fixture = (&'static str, &'static [Test]);

macro_rules! test {
    ($(fn $name:ident() $block:block)*) => {
        $(
            fn $name() $block
        )*

        #[doc(hidden)]
        pub const TESTS: crate::test::Fixture = (module_path!(), &[
            $(
                (stringify!($name), $name),
            )*
        ]);
    }
}
