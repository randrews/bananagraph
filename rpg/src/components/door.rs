pub struct Door {
    open: bool
}

impl Door {
    pub fn closed() -> Self {
        Self { open: false }
    }
}