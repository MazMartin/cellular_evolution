/// Macro to concatenate the contents of multiple source files into a single string.
///
/// # Usage
/// ```
/// let combined_code = combine_code!("file1.rs", "file2.rs");
/// ```
///
/// Accepts one or more string literals representing file paths.
/// Trailing comma is optional.
#[macro_export]
macro_rules! combine_code {
    ($($path:literal),+ $(,)?) => {{
        concat!(
            $(
                include_str!($path),
            )+
        )
    }};
}