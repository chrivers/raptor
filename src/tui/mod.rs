use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::Result as IoResult;
use std::ops::ControlFlow;
use std::os::fd::{AsFd, AsRawFd, RawFd};
use std::rc::Rc;
use std::time::Duration;

use crossbeam::channel::{Receiver, Sender, TryRecvError};
use itertools::Itertools;
use nix::poll::{PollFd, PollFlags};
use nix::pty::ForkptyResult;
use nix::sys::wait::waitpid;
use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tui_term::vt100;
use tui_term::widget::PseudoTerminal;

use crate::RaptorResult;
use crate::make::maker::Maker;
use crate::make::planner::{Job, Planner};
use crate::util::tty::TtyIoctl;

struct Pane {
    file: File,
    job: Job,
    parser: vt100::Parser,
}

struct PaneController {
    panes: HashMap<RawFd, Pane>,
    resize: bool,
    rx: Receiver<Pane>,
    boxes: Rc<[Rect]>,
}

impl PaneController {
    fn new(rx: Receiver<Pane>) -> Self {
        Self {
            panes: HashMap::new(),
            resize: false,
            rx,
            boxes: Rc::new([]),
        }
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
                    self.panes.insert(pane.file.as_raw_fd(), pane);
                    self.resize = true;
                }

                Err(TryRecvError::Empty) => return ControlFlow::Continue(()),
                Err(TryRecvError::Disconnected) => return ControlFlow::Break(()),
            }
        }
    }

    fn event(&mut self) -> RaptorResult<ControlFlow<()>> {
        let fds = self.poll_fds()?;

        self.process_fds(&fds);

        Ok(self.process_queue())
    }
}

struct PaneView<'a> {
    ctrl: &'a mut PaneController,
}

impl<'a> PaneView<'a> {
    #[must_use]
    const fn new(ctrl: &'a mut PaneController) -> Self {
        Self { ctrl }
    }

    fn render(self, frame: &mut Frame, area: Rect) -> IoResult<()> {
        let boxes = self.ctrl.make_layout(area)?;

        for (index, pane) in self.ctrl.panes.values_mut().enumerate() {
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

pub struct TerminalParallelRunner<'a> {
    maker: &'a Maker<'a>,
    terminal: &'a mut DefaultTerminal,
}

impl<'a> TerminalParallelRunner<'a> {
    pub const fn new(maker: &'a Maker, terminal: &'a mut DefaultTerminal) -> Self {
        Self { maker, terminal }
    }

    fn spawn_pty_job(maker: &Maker, tx: &Sender<Pane>, target: &Job) -> RaptorResult<()> {
        match unsafe { nix::pty::forkpty(None, None)? } {
            ForkptyResult::Parent { child, master } => {
                let pane = Pane {
                    file: master.into(),
                    job: target.clone(),
                    parser: vt100::Parser::default(),
                };

                tx.send(pane)?;
                waitpid(child, None)?;

                Ok(())
            }

            ForkptyResult::Child => {
                match target {
                    Job::Build(build) => {
                        maker.builder().build_layer(
                            &build.layers,
                            &build.target,
                            &build.layerinfo,
                        )?;
                    }

                    Job::Run(run_target) => {
                        maker.run_job(run_target)?;
                    }
                }

                std::thread::sleep(Duration::from_millis(250));

                std::process::exit(0);
            }
        }
    }

    fn render_terminal(rx: Receiver<Pane>, terminal: &'a mut DefaultTerminal) -> RaptorResult<()> {
        let mut panectrl = PaneController::new(rx);

        while panectrl.event()?.is_continue() {
            terminal.try_draw(|f| -> IoResult<()> {
                let chunks = Layout::vertical([Constraint::Percentage(100), Constraint::Min(1)])
                    .margin(1)
                    .split(f.area());

                let paneview = PaneView::new(&mut panectrl);
                paneview.render(f, chunks[0])?;

                let explanation = "Ctrl+q to quit";
                let explanation = Paragraph::new(explanation)
                    .style(Style::default().add_modifier(Modifier::BOLD))
                    .alignment(Alignment::Center);
                f.render_widget(explanation, chunks[1]);

                Ok(())
            })?;
        }

        Ok(())
    }

    pub fn execute<'b: 'a>(&'b mut self, planner: Planner) -> RaptorResult<()> {
        let (tx, rx) = crossbeam::channel::unbounded::<Pane>();
        let (plan, targetlist) = planner.into_plan();

        std::thread::scope(|s| {
            s.spawn(|| Self::render_terminal(rx, self.terminal));

            plan.into_par_iter().try_for_each_with(tx, |tx, id| {
                Self::spawn_pty_job(self.maker, tx, &targetlist[&id])
            })
        })
    }
}
