#[macro_export]
macro_rules! combine_code {
    ($($path:literal),+ $(,)?) => {{
        concat!($(
            include_str!($path),
        )+)
    }};
}