use std::collections::VecDeque;
use crate::worker::admission::{AdmissionInput, JobClass};

pub struct LocalQueue {
    pub interactive: VecDeque<AdmissionInput>,
    pub background: VecDeque<AdmissionInput>,
    pub bulk: VecDeque<AdmissionInput>,
}

impl LocalQueue {
    pub fn new() -> Self {
        Self {
            interactive: VecDeque::new(),
            background: VecDeque::new(),
            bulk: VecDeque::new(),
        }
    }

    pub fn push(&mut self, input: AdmissionInput) {
        match input.job_class {
            JobClass::Interactive => self.interactive.push_back(input),
            JobClass::Background => self.background.push_back(input),
            JobClass::Bulk => self.bulk.push_back(input),
        }
    }

    pub fn pop_next(&mut self) -> Option<AdmissionInput> {
        self.interactive.pop_front()
            .or_else(|| self.background.pop_front())
            .or_else(|| self.bulk.pop_front())
    }

    pub fn len(&self) -> usize {
        self.interactive.len() + self.background.len() + self.bulk.len()
    }
}
