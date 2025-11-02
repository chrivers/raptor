use std::collections::HashMap;
use std::ops::ControlFlow;

use crossbeam::channel::{Receiver, TryRecvError};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::RaptorResult;
use crate::make::maker::Maker;
use crate::make::planner::{Job, Planner};
use crate::tui::joblist::JobList;
use crate::tui::jobstate::JobState;
use crate::util::flag::Flag;

pub trait JobController {
    fn job_state(&self, id: u64) -> JobState;
}

#[derive(Default)]
pub struct BatchJobController {
    states: HashMap<u64, JobState>,
}

impl JobController for BatchJobController {
    fn job_state(&self, id: u64) -> JobState {
        *self.states.get(&id).unwrap_or(&JobState::Planned)
    }
}

impl BatchJobController {
    #[must_use]
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    pub fn add_job(&mut self, id: u64) {
        self.states.insert(id, JobState::Running);
    }

    pub fn end_job(&mut self, id: u64) {
        self.states.insert(id, JobState::Completed);
    }

    pub fn fail_job(&mut self, id: u64) {
        self.states.insert(id, JobState::Failed);
    }
}
