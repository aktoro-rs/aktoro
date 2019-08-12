/// TODO: documentation
pub struct Priority(Vec<(PriorityLevel, usize)>);

/// TODO: documentation
pub(crate) enum PriorityLevel {
    Minimum,
    Low,
    Normal,
    High,
    Maximum,
}

impl Priority {
    /// TODO: documentation
    pub fn new() -> Priority {
        Priority(vec![])
    }

    /// TODO: documentation
    pub fn minimum(mut self, during: usize) -> Priority {
        self.0.push((PriorityLevel::Minimum, during));
        self
    }

    /// TODO: documentation
    pub fn low(mut self, during: usize) -> Priority {
        self.0.push((PriorityLevel::Low, during));
        self
    }

    /// TODO: documentation
    pub fn normal(mut self, during: usize) -> Priority {
        self.0.push((PriorityLevel::Normal, during));
        self
    }

    /// TODO: documentation
    pub fn high(mut self, during: usize) -> Priority {
        self.0.push((PriorityLevel::High, during));
        self
    }

    /// TODO: documentation
    pub fn maximum(mut self, during: usize) -> Priority {
        self.0.push((PriorityLevel::Maximum, during));
        self
    }
}
