use std::io::Result as IoResult;
use std::time::Duration;

use crossbeam::channel::{Receiver, Sender};
use nix::pty::ForkptyResult;
use nix::sys::wait::waitpid;
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Layout};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::RaptorResult;
use crate::make::maker::Maker;
use crate::make::planner::{Job, Planner};
use crate::tui::joblist::{JobList, JobView};
use crate::tui::ptyctrl::{PtyJob, PtyJobController, PtyJobView};

pub mod joblist;
pub mod jobstate;
pub mod ptyctrl;

pub struct TerminalParallelRunner<'a> {
    maker: &'a Maker<'a>,
    terminal: &'a mut DefaultTerminal,
}

impl<'a> TerminalParallelRunner<'a> {
    pub const fn new(maker: &'a Maker, terminal: &'a mut DefaultTerminal) -> Self {
        Self { maker, terminal }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn spawn_pty_job(
        maker: &Maker,
        tx: &Sender<PtyJob>,
        id: u64,
        target: &Job,
    ) -> RaptorResult<()> {
        match unsafe { nix::pty::forkpty(None, None)? } {
            ForkptyResult::Parent { child, master } => {
                let job = PtyJob::new(master.into(), target.clone(), id);

                tx.send(job)?;
                waitpid(child, None)?;

                Ok(())
            }

            ForkptyResult::Child => {
                match target {
                    Job::Build(build) => maker.build(build).map(drop)?,
                    Job::Run(run_target) => maker.run_job(run_target).map(drop)?,
                }

                let delay = unsafe {
                    nix::libc::srand(nix::libc::getpid() as u32);
                    nix::libc::rand() % 500
                } + 300;
                std::thread::sleep(Duration::from_millis(delay as u64));

                std::process::exit(0);
            }
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn render_terminal(
        rx: Receiver<PtyJob>,
        planner: Planner,
        terminal: &'a mut DefaultTerminal,
    ) -> RaptorResult<()> {
        let mut jobctrl = PtyJobController::new(rx);
        let joblist = JobList::new(planner);

        let mut index = 0;
        let mut alive = true;

        while alive {
            if jobctrl.event()?.is_break() {
                alive = false;
            }

            terminal.try_draw(|f| -> IoResult<()> {
                let layout = Layout::vertical([
                    Constraint::Max(joblist.lines() as u16),
                    Constraint::Fill(1),
                ])
                .split(f.area());

                let job_view = JobView::new(&joblist, &jobctrl);
                f.render_stateful_widget(job_view, layout[0], &mut index);

                let pty_view = PtyJobView::new(&mut jobctrl);
                pty_view.render(f, layout[1])
            })?;
        }

        Ok(())
    }

    pub fn execute<'b: 'a>(&'b mut self, planner: Planner) -> RaptorResult<()> {
        let (tx, rx) = crossbeam::channel::unbounded::<PtyJob>();
        let (plan, targetlist) = planner.clone().into_plan();

        std::thread::scope(|s| {
            s.spawn(|| Self::render_terminal(rx, planner, self.terminal));

            plan.into_par_iter().try_for_each_with(tx, |tx, id| {
                Self::spawn_pty_job(self.maker, tx, *id, &targetlist[&id])
            })
        })
    }
}
