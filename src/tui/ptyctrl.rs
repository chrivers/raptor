use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Result as IoResult};
use std::ops::ControlFlow;
use std::os::fd::{AsFd, AsRawFd, RawFd};
use std::rc::Rc;

use crossbeam::channel::{Receiver, TryRecvError};
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

pub struct PtyJobController {
    panes: HashMap<RawFd, PtyJob>,
    resize: bool,
    rx: Receiver<PtyJob>,
    boxes: Rc<[Rect]>,
    states: HashMap<u64, JobState>,
}

impl PtyJobController {
    #[must_use]
    pub fn new(rx: Receiver<PtyJob>) -> Self {
        Self {
            panes: HashMap::new(),
            resize: false,
            rx,
            boxes: Rc::new([]),
            states: HashMap::new(),
        }
    }

    #[must_use]
    pub fn job_state(&self, id: u64) -> JobState {
        *self.states.get(&id).unwrap_or(&JobState::Planned)
    }

    fn poll_fds(&self) -> RaptorResult<Vec<RawFd>> {
        let mut pollfds = vec![];
        for pane in self.panes.values() {
            pollfds.push(PollFd::new(pane.file.as_fd(), PollFlags::POLLIN));
        }

        nix::poll::poll(&mut pollfds, 100u16)?;

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
            let pane = self.panes.get_mut(fd).unwrap();
            let sz = pane.file.read(&mut buf);
            match sz {
                Ok(0) | Err(_) => {
                    self.states.insert(pane.id, JobState::Completed);
                    self.panes.remove(fd);
                    self.resize = true;
                }
                Ok(sz) => {
                    pane.parser.process(&buf[..sz]);
                }
            }
        }
    }

    fn make_layout(&mut self, area: Rect) -> IoResult<Rc<[Rect]>> {
        if self.resize {
            self.boxes =
                Layout::horizontal(self.panes.iter().map(|_| Constraint::Fill(1))).split(area);

            for (pane, tbox) in self.panes.values_mut().zip(self.boxes.iter()) {
                pane.parser.set_size(tbox.height, tbox.width);
                pane.file.tty_set_size(tbox.height, tbox.width)?;
            }

            self.resize = false;
        }

        Ok(self.boxes.clone())
    }

    fn process_queue(&mut self) -> ControlFlow<()> {
        loop {
            match self.rx.try_recv() {
                Ok(pane) => {
                    self.states.insert(pane.id, JobState::Running);
                    self.panes.insert(pane.file.as_raw_fd(), pane);
                    self.resize = true;
                }

                Err(TryRecvError::Empty) => return ControlFlow::Continue(()),
                Err(TryRecvError::Disconnected) => return ControlFlow::Break(()),
            }
        }
    }

    pub fn event(&mut self) -> RaptorResult<ControlFlow<()>> {
        let fds = self.poll_fds()?;

        self.process_fds(&fds);

        Ok(self.process_queue())
    }
}

pub struct PaneView<'a> {
    ctrl: &'a mut PtyJobController,
}

impl<'a> PaneView<'a> {
    #[must_use]
    pub const fn new(ctrl: &'a mut PtyJobController) -> Self {
        Self { ctrl }
    }

    pub fn render(self, frame: &mut Frame, area: Rect) -> IoResult<()> {
        let boxes = self.ctrl.make_layout(area)?;

        for (index, pane) in self.ctrl.panes.values().enumerate() {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(format!("{:?}", &pane.job))
                .title_alignment(Alignment::Center)
                .style(Style::new().add_modifier(Modifier::BOLD).bg(Color::Blue));

            let screen = pane.parser.screen();
            let pseudo_term = PseudoTerminal::new(screen).block(block);

            frame.render_widget(pseudo_term, boxes[index]);
        }

        Ok(())
    }
}
