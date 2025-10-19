use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Result as IoResult};
use std::os::fd::{AsFd, AsRawFd, RawFd};
use std::rc::Rc;

use itertools::Itertools;
use nix::poll::{PollFd, PollFlags};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders};
use tui_term::vt100;
use tui_term::widget::PseudoTerminal;

use crate::RaptorResult;
use crate::make::planner::Job;
use crate::tui::jobstate::JobState;
use crate::util::tty::TtyIoctl;

pub struct PtyJob {
    file: File,
    job: Job,
    parser: vt100::Parser,
    id: u64,
}

impl Debug for PtyJob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PtyJob")
            .field("file", &self.file)
            .field("job", &self.job)
            .field("parser", &"<parser>")
            .field("id", &self.id)
            .finish()
    }
}

impl PtyJob {
    #[must_use]
    pub fn new(file: File, job: Job, id: u64) -> Self {
        Self::custom(file, job, vt100::Parser::default(), id)
    }

    #[must_use]
    pub const fn custom(file: File, job: Job, parser: vt100::Parser, id: u64) -> Self {
        Self {
            file,
            job,
            parser,
            id,
        }
    }
}

#[derive(Default)]
pub struct PtyJobController {
    jobs: HashMap<RawFd, PtyJob>,
    resize: bool,
    boxes: Rc<[Rect]>,
    states: HashMap<u64, JobState>,
}

impl PtyJobController {
    #[must_use]
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
            resize: false,
            boxes: Rc::new([]),
            states: HashMap::new(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }

    #[must_use]
    pub fn job_state(&self, id: u64) -> JobState {
        *self.states.get(&id).unwrap_or(&JobState::Planned)
    }

    #[must_use]
    pub fn complete(&self) -> bool {
        self.states
            .values()
            .all(|state| *state == JobState::Completed)
    }

    fn poll_fds(&self) -> RaptorResult<Vec<RawFd>> {
        let mut pollfds = vec![];
        for job in self.jobs.values() {
            pollfds.push(PollFd::new(job.file.as_fd(), PollFlags::POLLIN));
        }

        nix::poll::poll(&mut pollfds, 10u16)?;

        let res = pollfds
            .iter()
            .filter(|fd| fd.any().unwrap_or(false))
            .map(AsFd::as_fd)
            .map(|fd| fd.as_raw_fd())
            .collect_vec();

        Ok(res)
    }

    fn process_fds(&mut self, fds: &[RawFd]) {
        let mut buf = [0u8; 1024 * 8];

        for fd in fds {
            let job = self.jobs.get_mut(fd).unwrap();
            let sz = job.file.read(&mut buf);
            match sz {
                Ok(0) | Err(_) => {
                    self.states.insert(job.id, JobState::Completed);
                    self.jobs.remove(fd);
                    self.resize = true;
                }
                Ok(sz) => {
                    job.parser.process(&buf[..sz]);
                }
            }
        }
    }

    fn make_layout(&mut self, area: Rect) -> IoResult<Rc<[Rect]>> {
        if self.resize {
            self.boxes =
                Layout::horizontal(self.jobs.iter().map(|_| Constraint::Fill(1))).split(area);

            for (job, tbox) in self.jobs.values_mut().zip(self.boxes.iter()) {
                job.parser.set_size(tbox.height, tbox.width);
                job.file.tty_set_size(tbox.height, tbox.width)?;
            }

            self.resize = false;
        }

        Ok(self.boxes.clone())
    }

    pub fn add_job(&mut self, job: PtyJob) {
        self.states.insert(job.id, JobState::Running);
        self.jobs.insert(job.file.as_raw_fd(), job);
        self.resize = true;
    }

    pub fn event(&mut self) -> RaptorResult<()> {
        let fds = self.poll_fds()?;

        self.process_fds(&fds);

        Ok(())
    }
}

pub struct PtyJobView<'a> {
    ctrl: &'a mut PtyJobController,
}

impl<'a> PtyJobView<'a> {
    #[must_use]
    pub const fn new(ctrl: &'a mut PtyJobController) -> Self {
        Self { ctrl }
    }

    pub fn render(self, frame: &mut Frame, area: Rect) -> IoResult<()> {
        let boxes = self.ctrl.make_layout(area)?;

        for (index, job) in self.ctrl.jobs.values().enumerate() {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(format!("{}", &job.job))
                .title_alignment(Alignment::Center)
                .style(Style::new().add_modifier(Modifier::BOLD).bg(Color::Blue));

            let screen = job.parser.screen();
            let pseudo_term = PseudoTerminal::new(screen).block(block);

            frame.render_widget(pseudo_term, boxes[index]);
        }

        Ok(())
    }
}
