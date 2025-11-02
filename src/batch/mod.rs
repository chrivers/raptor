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

pub enum BatchEvent {
    JobBegin(u64),
    JobEnd(u64, RaptorResult<()>),
}

pub struct ParallelRunner<'a> {
    maker: &'a Maker<'a>,
}

impl<'a> ParallelRunner<'a> {
    #[must_use]
    pub const fn new(maker: &'a Maker) -> Self {
        Self { maker }
    }

    fn handle_event(
        rx: &Receiver<BatchEvent>,
        jobctrl: &mut BatchJobController,
        alive: &Flag,
    ) -> ControlFlow<()> {
        loop {
            match rx.try_recv() {
                Ok(BatchEvent::JobBegin(id)) => {
                    trace!("[{id:016X}] Job started");
                    jobctrl.add_job(id);
                }

                Ok(BatchEvent::JobEnd(id, status)) => {
                    if status.is_ok() {
                        trace!("[{id:016X}] Job finished");
                        jobctrl.end_job(id);
                    } else {
                        trace!("[{id:016X}] Job failed!");
                        jobctrl.fail_job(id);
                    }
                }

                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    alive.set(false);
                    break;
                }
            }
        }

        ControlFlow::Continue(())
    }

    #[allow(clippy::unnecessary_wraps)]
    fn render_terminal(
        rx: &Receiver<BatchEvent>,
        planner: Planner,
        alive: &Flag,
    ) -> RaptorResult<()> {
        let mut jobctrl = BatchJobController::new();
        let joblist = JobList::new(planner);

        while alive.get() {
            if Self::handle_event(rx, &mut jobctrl, alive).is_break() {
                break;
            }

            let jobstats = joblist.stats(&jobctrl);

            if jobstats.complete() {
                break;
            }

            /* println!( */
            /*     "| {:5} jobs | {:5} planned | {:5} running | {:5} completed | {:5} failed |", */
            /*     jobstats.sum(), */
            /*     jobstats.planned, */
            /*     jobstats.running, */
            /*     jobstats.completed, */
            /*     jobstats.failed, */
            /* ); */
        }

        Ok(())
    }

    pub fn execute<'b: 'a>(&'b mut self, planner: Planner) -> RaptorResult<()> {
        let (tx, rx) = crossbeam::channel::unbounded::<BatchEvent>();
        let (plan, targetlist) = planner.clone().into_plan();

        if targetlist.is_empty() {
            return Ok(());
        }

        let alive = Flag::new(true);

        std::thread::scope(|s| {
            s.spawn(|| Self::render_terminal(&rx, planner, &alive));

            plan.into_par_iter().try_for_each_with(&tx, |tx, id| {
                tx.send(BatchEvent::JobBegin(*id))?;

                let res = match &targetlist[&id] {
                    Job::Build(build) => self.maker.build(build).map(drop),
                    Job::Run { job, .. } => self.maker.run_job(job).map(drop),
                };

                tx.send(BatchEvent::JobEnd(*id, res))?;

                Ok(())
            })
        })
    }
}
