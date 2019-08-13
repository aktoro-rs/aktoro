use std::collections::VecDeque;

/// TODO: documentation
pub struct Priority {
    levels: VecDeque<(PriorityLevel, usize)>,
}

/// TODO: documentation
#[derive(Copy, Clone)]
pub enum PriorityLevel {
    Minimum,
    Low,
    Normal,
    High,
    Maximum,
}

impl Priority {
    /// TODO: documentation
    pub fn new() -> Priority {
        Priority { levels: VecDeque::new(), }
    }

    /// TODO: documentation
    pub fn minimum(mut self, iters: usize) -> Priority {
        self.levels.push_back((PriorityLevel::Minimum, iters));
        self
    }

    /// TODO: documentation
    pub fn low(mut self, iters: usize) -> Priority {
        self.levels.push_back((PriorityLevel::Low, iters));
        self
    }

    /// TODO: documentation
    pub fn normal(mut self, iters: usize) -> Priority {
        self.levels.push_back((PriorityLevel::Normal, iters));
        self
    }

    /// TODO: documentation
    pub fn high(mut self, iters: usize) -> Priority {
        self.levels.push_back((PriorityLevel::High, iters));
        self
    }

    /// TODO: documentation
    pub fn maximum(mut self, iters: usize) -> Priority {
        self.levels.push_back((PriorityLevel::Maximum, iters));
        self
    }

    /// TODO: documentation
    pub fn current(&self) -> PriorityLevel {
        self.levels.front()
            .map(|(priority, _)| *priority)
            .unwrap_or(PriorityLevel::Normal)
    }

    /// TODO: documentation
    pub fn itered(&mut self) {
        match self.levels.front_mut() {
            Some((_, iters)) if *iters == 0 => {
                self.levels.pop_front();
                self.itered();
            }
            Some((_, iters)) => *iters -= 1,
            None => (),
        }
    }
}
