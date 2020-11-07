#[derive(Default)]
pub(crate) struct Generation {
    gen: i32
}

impl Generation {
    fn invalidate(&mut self) {
        assert!(self.is_valid());
        self.gen = -self.gen - 1;
    }

    fn validate(&mut self) {
        assert!(!self.is_valid());
        self.gen = -self.gen
    }

    fn is_valid(&self) -> bool {
        return self.gen >= 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::generation::Generation;

    #[test]
    fn validate_gen() {
        let mut gen = Generation::default();
        assert!(gen.is_valid());
        gen.gen = i32::max_value();
        gen.invalidate();
        assert!(!gen.is_valid());
        gen.validate();
        assert!(!gen.is_valid()); // 因为溢出了，所以不会变为有效
    }
}
