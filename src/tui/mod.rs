use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::os::fd::{AsFd, AsRawFd, OwnedFd, RawFd};
use std::time::Duration;

use crossbeam::channel::{Receiver, Sender, TryRecvError};
use itertools::Itertools;
use nix::poll::{PollFd, PollFlags};
use nix::pty::ForkptyResult;
use nix::sys::wait::waitpid;
use ratatui::DefaultTerminal;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tui_term::vt100;
use tui_term::widget::PseudoTerminal;

use crate::RaptorResult;
use crate::build::RaptorBuilder;
use crate::make::maker::Maker;
use crate::make::planner::{Job, Planner};
use crate::util::tty::TtyIoctl;

struct Pane {
    file: File,
    job: Job,
    parser: vt100::Parser,
}

pub struct TerminalParallelRunner<'a> {
    builder: &'a RaptorBuilder<'a>,
    maker: &'a Maker,
    terminal: &'a mut DefaultTerminal,
}

impl<'a> TerminalParallelRunner<'a> {
    pub const fn new(
        builder: &'a RaptorBuilder,
        maker: &'a Maker,
        terminal: &'a mut DefaultTerminal,
    ) -> Self {
        Self {
            builder,
            maker,
            terminal,
        }
    }

    fn run_job_in_pty(
        builder: &'a RaptorBuilder,
        maker: &Maker,
        tx: &Sender<(OwnedFd, Job)>,
        target: &Job,
    ) -> RaptorResult<()> {
        match unsafe { nix::pty::forkpty(None, None)? } {
            ForkptyResult::Parent { child, master } => {
                tx.send((master, target.clone())).expect("tx");
                waitpid(child, None)?;

                Ok(())
            }

            ForkptyResult::Child => {
                match target {
                    Job::Build(build) => {
                        builder.build_layer(&build.layers, &build.target, &build.layerinfo)?;
                    }

                    Job::Run(run_target) => {
                        maker.run_job(builder, run_target)?;
                    }
                }

                std::thread::sleep(Duration::from_millis(250));

                std::process::exit(0);
            }
        }
    }

    fn poll_fds<'b>(panes: impl Iterator<Item = &'b Pane>) -> RaptorResult<Vec<RawFd>> {
        let mut pollfds = vec![];
        for pane in panes {
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

    #[allow(clippy::cast_possible_truncation)]
    fn run_terminal_display(
        rx: &Receiver<(OwnedFd, Job)>,
        terminal: &'a mut DefaultTerminal,
    ) -> RaptorResult<()> {
        let mut panes: HashMap<RawFd, Pane> = HashMap::new();
        let mut need_resize = false;

        loop {
            terminal.try_draw(|f| -> Result<(), std::io::Error> {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Percentage(100), Constraint::Min(4)].as_ref())
                    .split(f.area());

                let pane_width = if panes.is_empty() {
                    chunks[0].width
                } else {
                    (chunks[0].width.saturating_sub(1)) / panes.len() as u16
                };

                if need_resize {
                    for pane in panes.values_mut() {
                        let rows = chunks[0].height;
                        let cols = pane_width;
                        pane.parser.set_size(rows, cols);
                        pane.file.tty_set_size(rows, cols)?;
                    }

                    need_resize = false;
                }

                for (index, pane) in panes.values_mut().enumerate() {
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title(format!("{:?}", &pane.job))
                        .title_alignment(Alignment::Center)
                        .style(Style::new().add_modifier(Modifier::BOLD).bg(Color::Blue));

                    let screen = pane.parser.screen();
                    let pseudo_term = PseudoTerminal::new(screen).block(block);

                    let pane_chunk = Rect {
                        x: chunks[0].x + (index as u16 * pane_width),
                        y: chunks[0].y,
                        width: pane_width,
                        height: chunks[0].height,
                    };

                    f.render_widget(pseudo_term, pane_chunk);
                }

                let explanation = "Ctrl+q to quit";
                let explanation = Paragraph::new(explanation)
                    .style(Style::default().add_modifier(Modifier::BOLD))
                    .alignment(Alignment::Center);
                f.render_widget(explanation, chunks[1]);

                Ok(())
            })?;

            let fds = Self::poll_fds(panes.values())?;

            let mut buf = [0u8; 1024 * 8];

            for fd in fds {
                let pane = panes.get_mut(&fd).unwrap();
                let sz = pane.file.read(&mut buf);
                match sz {
                    Ok(0) | Err(_) => {
                        panes.remove(&fd);
                        need_resize = true;
                    }
                    Ok(sz) => {
                        pane.parser.process(&buf[..sz]);
                    }
                }
            }

            match rx.try_recv() {
                Ok((fd, job)) => {
                    let raw_fd = fd.as_raw_fd();
                    let file = fd.into();
                    let parser = vt100::Parser::new(25, 80, 0);

                    let pane = Pane { job, file, parser };
                    panes.insert(raw_fd, pane);
                    need_resize = true;
                }

                Err(TryRecvError::Empty) => {}

                Err(TryRecvError::Disconnected) => {
                    if panes.is_empty() {
                        return Ok(());
                    }
                }
            }
        }
    }

    pub fn execute<'b: 'a>(&'b mut self, planner: Planner) -> RaptorResult<()> {
        let (tx, rx) = crossbeam::channel::unbounded::<(OwnedFd, Job)>();
        let (plan, targetlist) = planner.into_plan();

        std::thread::scope(|s| {
            s.spawn(|| Self::run_terminal_display(&rx, self.terminal));

            plan.into_par_iter().try_for_each_with(tx, |tx, id| {
                Self::run_job_in_pty(self.builder, self.maker, tx, &targetlist[&id])
            })
        })
    }
}
