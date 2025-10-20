use std::collections::HashMap;

use dep_graph::DepGraph;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, StatefulWidget, Widget};

use crate::make::planner::{Job, Planner};
use crate::tui::jobstate::JobState;
use crate::tui::ptyctrl::PtyJobController;

pub struct JobList {
    jobs: Vec<(usize, u64)>,
    targetlist: HashMap<u64, Job>,
}

#[derive(Default)]
pub struct JobStats {
    pub planned: usize,
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
}

impl JobStats {
    #[must_use]
    pub const fn sum(&self) -> usize {
        self.planned + self.running + self.completed + self.failed
    }

    #[must_use]
    pub const fn complete(&self) -> bool {
        (self.planned + self.running) == 0
    }
}

impl JobList {
    #[must_use]
    pub fn new(planner: Planner) -> Self {
        let (plan, targetlist) = planner.into_plan();

        let mut jobs = vec![];
        for node in &plan.ready_nodes {
            Self::generate_sublist(&plan, *node, 1, &mut jobs);
        }

        Self { jobs, targetlist }
    }

    #[must_use]
    pub const fn lines(&self) -> usize {
        self.jobs.len()
    }

    #[must_use]
    pub fn complete(&self, ctrl: &PtyJobController) -> bool {
        self.targetlist
            .keys()
            .all(|id| ctrl.job_state(*id) == JobState::Completed)
    }

    #[must_use]
    pub fn stats(&self, ctrl: &PtyJobController) -> JobStats {
        let mut stats = JobStats::default();
        for key in self.targetlist.keys() {
            match ctrl.job_state(*key) {
                JobState::Planned => stats.planned += 1,
                JobState::Running => stats.running += 1,
                JobState::Completed => stats.completed += 1,
                JobState::Failed => stats.failed += 1,
            }
        }
        stats
    }

    fn generate_sublist(
        plan: &DepGraph<u64>,
        node: u64,
        indent: usize,
        list: &mut Vec<(usize, u64)>,
    ) {
        list.push((indent, node));

        let Ok(read) = plan.rdeps.read() else {
            return;
        };

        for node in read.get(&node).into_iter().flatten() {
            Self::generate_sublist(plan, *node, indent + 1, list);
        }
    }
}

pub struct JobView<'a> {
    pub list: &'a JobList,
    pub ctrl: &'a PtyJobController,
}

impl<'a> JobView<'a> {
    #[must_use]
    pub const fn new(list: &'a JobList, ctrl: &'a PtyJobController) -> Self {
        Self { list, ctrl }
    }
}

impl StatefulWidget for JobView<'_> {
    type State = usize;

    fn render(self, area: Rect, buf: &mut Buffer, index: &mut Self::State) {
        let mut lines = vec![];
        for (indent, id) in &self.list.jobs {
            let state = self.ctrl.job_state(*id);
            let mut line = Line::raw("  ".repeat(*indent));
            line.push_span(Span::styled(state.symbol(*index), state.color()));
            line.push_span(format!(" {}", &self.list.targetlist[id]));
            lines.push(line);
        }

        let block = Block::default()
            .padding(Padding::proportional(1))
            .borders(Borders::ALL)
            .style(Style::new().add_modifier(Modifier::BOLD).bg(Color::Black));
        let p = Paragraph::new(lines).block(block);

        p.render(area, buf);

        *index += 1;
    }
}
