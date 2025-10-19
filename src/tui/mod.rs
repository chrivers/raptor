use std::io::Result as IoResult;
use std::ops::ControlFlow;
use std::time::Duration;

use crossbeam::channel::{Receiver, Sender, TryRecvError};
use nix::pty::ForkptyResult;
use nix::sys::wait::{WaitStatus, waitpid};
use nix::unistd::Pid;
use rand::Rng;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::make::maker::Maker;
use crate::make::planner::{Job, Planner};
use crate::tui::joblist::{JobList, JobView};
use crate::tui::logo::RaptorLogo;
use crate::tui::ptyctrl::{PtyJob, PtyJobController, PtyJobView};
use crate::tui::statusbar::StatusBar;
use crate::util::flag::Flag;
use crate::{RaptorError, RaptorResult};

pub mod joblist;
pub mod jobstate;
pub mod logo;
pub mod ptyctrl;
pub mod statusbar;

pub enum TerminalEvent {
    JobBegin(Pid, Box<PtyJob>),
    JobEnd(Pid, i32),
    Event(Event),
}

pub struct TerminalParallelRunner<'a> {
    maker: &'a Maker<'a>,
    terminal: &'a mut DefaultTerminal,
}

impl<'a> TerminalParallelRunner<'a> {
    pub const fn new(maker: &'a Maker, terminal: &'a mut DefaultTerminal) -> Self {
        Self { maker, terminal }
    }

    fn spawn_pty_job(
        maker: &Maker,
        tx: &Sender<TerminalEvent>,
        id: u64,
        target: &Job,
    ) -> RaptorResult<Pid> {
        match unsafe { nix::pty::forkpty(None, None)? } {
            ForkptyResult::Parent { child, master } => {
                let job = PtyJob::new(master.into(), target.clone(), id);

                tx.send(TerminalEvent::JobBegin(child, Box::new(job)))?;

                Ok(child)
            }

            ForkptyResult::Child => {
                let res = match target {
                    Job::Build(build) => maker.build(build).map(drop),
                    Job::Run { job, .. } => maker.run_job(job).map(drop),
                };

                // random delay to debug timing issues and timing-related
                // presentation quirks
                let amount = rand::rng().random_range(300..800);
                let duration = Duration::from_millis(amount);
                std::thread::sleep(duration);

                match res {
                    Ok(()) => std::process::exit(0),
                    Err(err) => {
                        error!("Child process failed: {err:?}");
                        std::process::exit(1)
                    }
                }
            }
        }
    }

    fn run_pty_job(
        maker: &Maker,
        tx: &Sender<TerminalEvent>,
        id: u64,
        target: &Job,
    ) -> RaptorResult<()> {
        let pid = Self::spawn_pty_job(maker, tx, id, target)?;

        loop {
            match waitpid(pid, None)? {
                WaitStatus::Exited(pid, result) => {
                    tx.send(TerminalEvent::JobEnd(pid, result))?;
                    return Err(RaptorError::LayerBuildError);
                }
                WaitStatus::Signaled(_pid, _signal, _) => {
                    tx.send(TerminalEvent::JobEnd(pid, -1))?;
                    return Err(RaptorError::LayerBuildError);
                }
                WaitStatus::Stopped(_pid, _signal) => {}
                WaitStatus::PtraceEvent(_pid, _signal, _) => {}
                WaitStatus::PtraceSyscall(_pid) => {}
                WaitStatus::Continued(_pid) => {}
                WaitStatus::StillAlive => unreachable!(),
            }
        }
    }

    fn handle_terminal_event(
        rx: &Receiver<TerminalEvent>,
        jobctrl: &mut PtyJobController,
        alive: &Flag,
    ) -> ControlFlow<()> {
        loop {
            match rx.try_recv() {
                Ok(TerminalEvent::JobBegin(pid, job)) => {
                    jobctrl.add_job(pid, *job);
                }

                Ok(TerminalEvent::JobEnd(pid, status)) => {
                    if status == 0 {
                        jobctrl.end_job(pid);
                    } else {
                        jobctrl.fail_job(pid);
                    }
                }

                Ok(TerminalEvent::Event(evt)) => {
                    if let Event::Key(key) = evt
                        && key.code == KeyCode::Char('q')
                    {
                        alive.set(false);
                        return ControlFlow::Break(());
                    }
                }

                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    if jobctrl.is_empty() {
                        alive.set(false);
                    }
                    break;
                }
            }
        }

        ControlFlow::Continue(())
    }

    #[allow(clippy::cast_possible_truncation)]
    fn render_terminal(
        rx: &Receiver<TerminalEvent>,
        planner: Planner,
        terminal: &'a mut DefaultTerminal,
        alive: &Flag,
    ) -> RaptorResult<()> {
        let mut jobctrl = PtyJobController::new();
        let joblist = JobList::new(planner);

        let mut index = 0;

        while alive.get() {
            jobctrl.event()?;
            if Self::handle_terminal_event(rx, &mut jobctrl, alive).is_break() {
                break;
            }

            terminal.try_draw(|f| -> IoResult<()> {
                let layout = Layout::vertical([
                    Constraint::Length(joblist.lines() as u16 + 4),
                    Constraint::Fill(1),
                    Constraint::Length(1),
                ])
                .split(f.area());

                let job_view = JobView::new(&joblist, &jobctrl);
                f.render_stateful_widget(job_view, layout[0], &mut index);

                let jobstats = joblist.stats(&jobctrl);

                if jobstats.complete() {
                    let logo = if jobstats.failed == 0 {
                        RaptorLogo::complete()
                    } else {
                        RaptorLogo::failed()
                    };
                    f.render_widget(logo, layout[1]);
                } else {
                    let pty_view = PtyJobView::new(&mut jobctrl);
                    pty_view.render(f, layout[1])?;
                }

                let mut status = StatusBar::new();

                status.add(" Raptor".bold());
                status.counter(jobstats.sum(), "jobs");
                status.counter(jobstats.planned, "planned");
                status.counter(jobstats.running, "running");
                status.counter(jobstats.completed, "completed");
                status.counter(jobstats.failed, "failed");

                f.render_widget(status, layout[2]);

                Ok(())
            })?;
        }

        Ok(())
    }

    fn read_events(tx: &Sender<TerminalEvent>, alive: &Flag) -> RaptorResult<()> {
        while alive.get() {
            if event::poll(Duration::from_millis(100))? {
                tx.send(TerminalEvent::Event(event::read()?))?;
            }
        }
        Ok(())
    }

    pub fn execute<'b: 'a>(&'b mut self, planner: Planner) -> RaptorResult<()> {
        let (tx, rx) = crossbeam::channel::unbounded::<TerminalEvent>();
        let (plan, targetlist) = planner.clone().into_plan();

        if targetlist.is_empty() {
            return Ok(());
        }

        let alive = Flag::new(true);

        std::thread::scope(|s| {
            s.spawn(|| Self::render_terminal(&rx, planner, self.terminal, &alive));

            s.spawn(|| Self::read_events(&tx, &alive));

            plan.into_par_iter().try_for_each_with(&tx, |tx, id| {
                Self::run_pty_job(self.maker, tx, *id, &targetlist[&id])
            })
        })
    }
}
