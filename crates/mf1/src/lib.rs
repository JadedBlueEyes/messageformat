use std::{borrow::Cow, error::Error};

#[cfg(feature = "macros")]
pub use mf1_macros::{load_locales, t_l_string};

/// This is used to call `.build` on `&str` when building interpolations.
///
/// If it's a `&str` it will just return the str,
/// but if it's a builder `.build` will either emit an error for a missing key or if all keys
/// are supplied it will return the correct value
///
/// It has no uses outside of macro internals.
#[doc(hidden)]
pub trait BuildStr: Sized {
    #[inline]
    fn build(self) -> Self {
        self
    }

    #[inline]
    fn build_display(self) -> Self {
        self
    }

    fn build_string(self) -> Cow<'static, str>;
}

impl BuildStr for &'static str {
    #[inline]
    fn build_string(self) -> Cow<'static, str> {
        Cow::Borrowed(self)
    }
}

#[doc(hidden)]
pub trait Formatable<'a> {
    // type Error;
    fn write_str(&mut self, data: &str) -> Result<(), Box<dyn Error>>;
    fn write_fmt(&mut self, args: std::fmt::Arguments) -> Result<(), Box<dyn Error>>;
}

impl<'a> Formatable<'a> for core::fmt::Formatter<'a> {
    // type Error = std::fmt::Error;

    fn write_str(&mut self, data: &str) -> Result<(), Box<dyn Error>> {
        Ok(self.write_str(data)?)
    }

    fn write_fmt(&mut self, args: std::fmt::Arguments) -> Result<(), Box<dyn Error>> {
        Ok(self.write_fmt(args)?)
    }
}
