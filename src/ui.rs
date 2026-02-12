use ratatui::{prelude::*, widgets::*};

use crate::app::{fmt_dur, App, Phase};

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let outer = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(" Arrabbiata ")
        .title_alignment(Alignment::Center);
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let chunks = Layout::vertical([
        Constraint::Length(3), // timer text
        Constraint::Length(1), // gauge
        Constraint::Length(1), // spacer
        Constraint::Length(2), // current run header + stats
        Constraint::Min(3),   // bar chart
        Constraint::Length(1), // spacer
        Constraint::Length(3), // history
        Constraint::Length(1), // spacer
        Constraint::Length(1), // keys
        Constraint::Length(1), // error
    ])
    .split(inner);

    let color = app.status_color();
    let elapsed = app.elapsed_sec();

    // Timer info
    let timer = Paragraph::new(vec![
        Line::from(vec![
            Span::raw(" Status: "),
            Span::styled(
                format!("{} ({})", app.status_text(), fmt_dur(app.planned_time_sec)),
                Style::default().fg(color).bold(),
            ),
        ]),
        Line::from(format!(
            " Elapsed: {}    Remaining: {}",
            fmt_dur(elapsed),
            fmt_dur(app.remaining_sec()),
        )),
        Line::from(format!(" Progress: {:.1}%", app.progress_pct())),
    ]);
    frame.render_widget(timer, chunks[0]);

    // Gauge
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(color).bg(Color::DarkGray))
        .percent(app.progress_pct() as u16)
        .label(format!("{:.0}%", app.progress_pct()));
    frame.render_widget(gauge, chunks[1]);

    // Current run
    let run = Paragraph::new(vec![
        Line::from(Span::styled(
            " Current Run",
            Style::default().bold().fg(Color::Yellow),
        )),
        Line::from(format!(
            " Work: {}  |  Break: {}",
            fmt_dur(app.run_work_sec()),
            fmt_dur(app.run_pause_sec()),
        )),
    ]);
    frame.render_widget(run, chunks[3]);

    // Bar chart
    if !app.workouts.is_empty() {
        let bars: Vec<Bar> = app
            .workouts
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let (label, c) = if i % 2 == 0 {
                    ("W", Color::Blue)
                } else {
                    ("B", Color::Green)
                };
                Bar::default()
                    .value(v as u64)
                    .label(Line::from(label))
                    .style(Style::default().fg(c))
            })
            .collect();
        let chart = BarChart::default()
            .data(BarGroup::default().bars(&bars))
            .bar_width(3)
            .bar_gap(1);
        frame.render_widget(chart, chunks[4]);
    }

    // History
    let history = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(" History", Style::default().bold().fg(Color::Yellow)),
            Span::raw(format!("  Runs: {}", app.total_runs)),
        ]),
        Line::from(format!(" Work: {}", fmt_dur(app.stat_work_sec))),
        Line::from(format!(" Breaks: {}", fmt_dur(app.stat_pause_sec))),
    ]);
    frame.render_widget(history, chunks[6]);

    // Keybindings
    let keys = if app.loading {
        Line::from(Span::styled(
            " Loading...",
            Style::default().fg(Color::Yellow),
        ))
    } else {
        let mut spans: Vec<Span> = Vec::new();
        match app.phase {
            Phase::Initial => spans.extend([
                Span::styled(" [s]", Style::default().fg(Color::Cyan).bold()),
                Span::raw(" Start  "),
                Span::styled("[c]", Style::default().fg(Color::Cyan).bold()),
                Span::raw(" Continue  "),
            ]),
            Phase::Running => spans.extend([
                Span::styled(" [f]", Style::default().fg(Color::Cyan).bold()),
                Span::raw(" Finish  "),
            ]),
            Phase::Stopped => spans.extend([
                Span::styled(" [n]", Style::default().fg(Color::Cyan).bold()),
                Span::raw(" Next  "),
            ]),
        }
        spans.extend([
            Span::styled("[q]", Style::default().fg(Color::Red).bold()),
            Span::raw(" Quit"),
        ]);
        Line::from(spans)
    };
    frame.render_widget(Paragraph::new(keys), chunks[8]);

    // Error
    if let Some(ref err) = app.error {
        frame.render_widget(
            Paragraph::new(format!(" Error: {err}")).style(Style::default().fg(Color::Red)),
            chunks[9],
        );
    }
}
