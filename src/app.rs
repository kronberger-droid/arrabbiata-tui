use std::io::{self, Write};
use std::time::Instant;

use ratatui::style::Color;

use crate::api::ApiResponse;

#[derive(PartialEq)]
pub enum Phase {
    Initial,
    Running,
    Stopped,
}

pub struct App {
    pub phase: Phase,
    pub planned_time_sec: f64,
    pub timer_start: Option<Instant>,
    pub elapsed_at_stop_sec: f64,
    pub current_type: Option<i32>,
    pub notified: bool,
    pub last_user_id: Option<String>,
    pub total_runs: u64,
    pub stat_work_sec: f64,
    pub stat_pause_sec: f64,
    pub workouts: Vec<f64>,
    pub error: Option<String>,
    pub loading: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            phase: Phase::Initial,
            planned_time_sec: 0.0,
            timer_start: None,
            elapsed_at_stop_sec: 0.0,
            current_type: None,
            notified: false,
            last_user_id: None,
            total_runs: 0,
            stat_work_sec: 0.0,
            stat_pause_sec: 0.0,
            workouts: Vec::new(),
            error: None,
            loading: false,
        }
    }

    pub fn elapsed_sec(&self) -> f64 {
        match (&self.phase, self.timer_start) {
            (Phase::Running, Some(start)) => start.elapsed().as_secs_f64(),
            (Phase::Stopped, _) => self.elapsed_at_stop_sec,
            _ => 0.0,
        }
    }

    pub fn remaining_sec(&self) -> f64 {
        (self.planned_time_sec - self.elapsed_sec()).max(0.0)
    }

    pub fn progress_pct(&self) -> f64 {
        if self.planned_time_sec > 0.0 {
            ((self.elapsed_sec() / self.planned_time_sec) * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        }
    }

    pub fn status_text(&self) -> &str {
        match self.current_type {
            Some(0) => "Work",
            Some(1) => "Break",
            _ => "-",
        }
    }

    pub fn status_color(&self) -> Color {
        match self.current_type {
            Some(0) => Color::Blue,
            Some(1) => Color::Green,
            _ => Color::White,
        }
    }

    pub fn handle_response(&mut self, resp: ApiResponse) {
        self.loading = false;
        self.error = None;

        if let Some(w) = resp.workout {
            self.last_user_id = w.user_id;
            self.planned_time_sec = w.planned_time.unwrap_or(0.0);
            self.current_type = w.workout_type;
            self.timer_start = Some(Instant::now());
            self.notified = false;
            self.elapsed_at_stop_sec = 0.0;
            self.phase = Phase::Running;
        }

        if let Some(s) = resp.stats {
            self.total_runs = s.total_runs.unwrap_or(0);
            self.stat_work_sec = s.work_count.unwrap_or(0.0);
            self.stat_pause_sec = s.pause_count.unwrap_or(0.0);
        }

        if let Some(w) = resp.workouts {
            self.workouts = w;
        }
    }

    pub fn stop_timer(&mut self) {
        if let Some(start) = self.timer_start {
            self.elapsed_at_stop_sec = start.elapsed().as_secs_f64().floor();
        }
        self.phase = Phase::Stopped;
    }

    pub fn check_notify(&mut self) -> bool {
        if !self.notified
            && self.phase == Phase::Running
            && self.planned_time_sec > 0.0
            && self.elapsed_sec() >= self.planned_time_sec
        {
            self.notified = true;
            true
        } else {
            false
        }
    }

    pub fn run_work_sec(&self) -> f64 {
        self.workouts.iter().step_by(2).sum()
    }

    pub fn run_pause_sec(&self) -> f64 {
        self.workouts.iter().skip(1).step_by(2).sum()
    }
}

pub fn send_notification(msg: &str) {
    // Terminal bell via stderr (ratatui owns stdout)
    let _ = io::stderr().write_all(b"\x07");
    let _ = io::stderr().flush();

    // Desktop notification via D-Bus (mako, dunst, etc.)
    let _ = notify_rust::Notification::new()
        .summary("Arrabbiata")
        .body(msg)
        .icon("appointment-soon")
        .urgency(notify_rust::Urgency::Critical)
        .timeout(notify_rust::Timeout::Milliseconds(30000))
        .show();
}

pub fn fmt_dur(secs: f64) -> String {
    let s = if secs.is_finite() && secs >= 0.0 {
        secs as u64
    } else {
        0
    };
    format!("{}h {}m {}s", s / 3600, (s % 3600) / 60, s % 60)
}
