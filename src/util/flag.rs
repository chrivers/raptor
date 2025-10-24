use std::sync::atomic::{AtomicBool, Ordering};

pub struct Flag(AtomicBool);

impl Flag {
    #[must_use]
    pub const fn new(value: bool) -> Self {
        Self(AtomicBool::new(value))
    }

    #[must_use]
    pub fn get(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    pub fn set(&self, value: bool) {
        self.0.store(value, Ordering::Relaxed);
    }
}

#[allow(clippy::bool_assert_comparison)]
#[cfg(test)]
mod tests {
    use crate::util::flag::Flag;

    #[test]
    fn flag_new() {
        let flag = Flag::new(false);
        assert_eq!(flag.get(), false);

        let flag = Flag::new(true);
        assert_eq!(flag.get(), true);
    }

    #[test]
    fn flag_set() {
        let flag = Flag::new(false);
        assert_eq!(flag.get(), false);
        flag.set(true);
        assert_eq!(flag.get(), true);
    }
}
