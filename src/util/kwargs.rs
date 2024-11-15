use minijinja::value::{ArgType, Kwargs};
use minijinja::Error;

pub trait KwargsExt {
    fn get_option<'a, T>(&'a self, name: &'a str) -> Result<Option<T>, Error>
    where
        T: ArgType<'a, Output = T>;

    fn get_or_default<'a, T>(&'a self, name: &'a str, default: T) -> Result<T, Error>
    where
        T: ArgType<'a, Output = T>,
    {
        Ok(self.get_option(name)?.unwrap_or(default))
    }
}

impl KwargsExt for Kwargs {
    fn get_option<'a, T>(&'a self, name: &'a str) -> Result<Option<T>, Error>
    where
        T: ArgType<'a, Output = T>,
    {
        if self.has(name) {
            self.get(name)
        } else {
            Ok(None)
        }
    }
}
