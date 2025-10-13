use chrono::{DateTime, Utc};
use eframe::egui;
use std::time::Duration;

use crate::jobs::manager::JobManager;
use crate::jobs::{Job, JobType, RepeatInterval, ScheduledJob};

pub struct JobSchedulingWidget {
    show_create_dialog: bool,
    new_job_name: String,
    new_job_type: JobType,
    schedule_time: DateTime<Utc>,
    repeat_interval: RepeatInterval,
    max_runs: Option<u32>,
    selected_dependencies: Vec<String>, // Job IDs that this job depends on
}

impl Default for JobSchedulingWidget {
    fn default() -> Self {
        Self {
            show_create_dialog: false,
            new_job_name: String::new(),
            new_job_type: JobType::GcodeFile,
            schedule_time: Utc::now() + chrono::Duration::hours(1), // Default to 1 hour from now
            repeat_interval: RepeatInterval::None,
            max_runs: None,
            selected_dependencies: Vec::new(),
        }
    }
}

impl JobSchedulingWidget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, ctx: &egui::Context, job_manager: &mut JobManager) {
        egui::Window::new("Job Scheduling")
            .open(&mut true)
            .show(ctx, |ui| {
                self.ui(ui, job_manager);
            });
    }

    fn ui(&mut self, ui: &mut egui::Ui, job_manager: &mut JobManager) {
        ui.heading("Scheduled Jobs");

        // Create new scheduled job button
        if ui.button("Create Scheduled Job").clicked() {
            self.show_create_dialog = true;
        }

        ui.separator();

        // List existing scheduled jobs
        self.show_scheduled_jobs_list(ui, job_manager);

        // Show create dialog if requested
        if self.show_create_dialog {
            self.show_create_dialog(ui, job_manager);
        }
    }

    fn show_scheduled_jobs_list(&mut self, ui: &mut egui::Ui, job_manager: &mut JobManager) {
        ui.heading("Current Schedules");

        let scheduled_jobs = job_manager.get_scheduled_jobs().clone();

        if scheduled_jobs.is_empty() {
            ui.label("No scheduled jobs");
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            for scheduled_job in &scheduled_jobs {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(&scheduled_job.job.name);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("❌").on_hover_text("Cancel schedule").clicked() {
                                if let Err(e) =
                                    job_manager.cancel_schedule(&scheduled_job.schedule_id)
                                {
                                    eprintln!("Failed to cancel schedule: {}", e);
                                }
                            }

                            if scheduled_job.enabled {
                                if ui.button("⏸️").on_hover_text("Disable").clicked() {
                                    if let Err(e) =
                                        job_manager.disable_schedule(&scheduled_job.schedule_id)
                                    {
                                        eprintln!("Failed to disable schedule: {}", e);
                                    }
                                }
                            } else {
                                if ui.button("▶️").on_hover_text("Enable").clicked() {
                                    if let Err(e) =
                                        job_manager.enable_schedule(&scheduled_job.schedule_id)
                                    {
                                        eprintln!("Failed to enable schedule: {}", e);
                                    }
                                }
                            }
                        });
                    });

                    ui.label(format!("Type: {:?}", scheduled_job.job.job_type));
                    ui.label(format!(
                        "Next run: {}",
                        scheduled_job
                            .next_run
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                            .unwrap_or_else(|| "Never".to_string())
                    ));

                    match &scheduled_job.repeat_interval {
                        RepeatInterval::None => {
                            ui.label("One-time job");
                        }
                        RepeatInterval::Minutes(m) => {
                            ui.label(format!("Repeats every {} minutes", m));
                        }
                        RepeatInterval::Hours(h) => {
                            ui.label(format!("Repeats every {} hours", h));
                        }
                        RepeatInterval::Days(d) => {
                            ui.label(format!("Repeats every {} days", d));
                        }
                        RepeatInterval::Weeks(w) => {
                            ui.label(format!("Repeats every {} weeks", w));
                        }
                        RepeatInterval::Months(m) => {
                            ui.label(format!("Repeats every {} months", m));
                        }
                    }

                    if let Some(max) = scheduled_job.max_runs {
                        ui.label(format!(
                            "Max runs: {} (run {} of {})",
                            max, scheduled_job.run_count, max
                        ));
                    } else {
                        ui.label(format!("Runs: {}", scheduled_job.run_count));
                    }

                    if !scheduled_job.dependencies.is_empty() {
                        ui.label(format!(
                            "Dependencies: {}",
                            scheduled_job.dependencies.len()
                        ));
                    }

                    ui.label(if scheduled_job.enabled {
                        "✅ Enabled"
                    } else {
                        "⏸️ Disabled"
                    });
                });
            }
        });

        ui.separator();

        // Process scheduled jobs button
        if ui.button("Process Scheduled Jobs").clicked() {
            match job_manager.process_scheduled_jobs() {
                Ok(executed_ids) => {
                    if executed_ids.is_empty() {
                        ui.label("No jobs were ready to execute");
                    } else {
                        ui.label(format!("Executed {} scheduled jobs", executed_ids.len()));
                    }
                }
                Err(e) => {
                    ui.label(format!("Error processing scheduled jobs: {}", e));
                }
            }
        }

        // Show upcoming jobs
        self.show_upcoming_jobs(ui, job_manager);
    }

    fn show_upcoming_jobs(&self, ui: &mut egui::Ui, job_manager: &JobManager) {
        ui.separator();
        ui.heading("Upcoming Jobs (Next 24 Hours)");

        let upcoming = job_manager.get_upcoming_scheduled_jobs(Duration::from_secs(24 * 60 * 60));

        if upcoming.is_empty() {
            ui.label("No jobs scheduled in the next 24 hours");
            return;
        }

        for job in upcoming {
            ui.horizontal(|ui| {
                ui.label(&job.job.name);
                ui.label(format!(
                    "at {}",
                    job.next_run
                        .map(|dt| dt.format("%H:%M:%S").to_string())
                        .unwrap_or_else(|| "Unknown".to_string())
                ));
            });
        }
    }

    fn show_create_dialog(&mut self, ui: &mut egui::Ui, job_manager: &mut JobManager) {
        let mut should_close = false;
        let mut should_create = false;

        // Create local copies of the mutable fields we need
        let mut new_job_name = self.new_job_name.clone();
        let mut new_job_type = self.new_job_type.clone();
        let mut repeat_interval = self.repeat_interval.clone();
        let mut max_runs = self.max_runs;
        let mut selected_dependencies = self.selected_dependencies.clone();

        egui::Window::new("Create Scheduled Job")
            .open(&mut self.show_create_dialog)
            .show(ui.ctx(), |ui| {
                ui.heading("Schedule New Job");

                egui::Grid::new("create_schedule_grid")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Job Name:");
                        ui.text_edit_singleline(&mut new_job_name);
                        ui.end_row();

                        ui.label("Job Type:");
                        egui::ComboBox::from_label("")
                            .selected_text(format!("{:?}", new_job_type))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut new_job_type,
                                    JobType::GcodeFile,
                                    "G-code File",
                                );
                                ui.selectable_value(
                                    &mut new_job_type,
                                    JobType::CAMOperation,
                                    "CAM Operation",
                                );
                                ui.selectable_value(&mut new_job_type, JobType::Probing, "Probing");
                                ui.selectable_value(
                                    &mut new_job_type,
                                    JobType::Calibration,
                                    "Calibration",
                                );
                                ui.selectable_value(
                                    &mut new_job_type,
                                    JobType::Maintenance,
                                    "Maintenance",
                                );
                            });
                        ui.end_row();

                        ui.label("Schedule Time:");
                        // For simplicity, just show the datetime as text
                        // In a real implementation, you'd want a proper date/time picker
                        let time_str = self
                            .schedule_time
                            .format("%Y-%m-%d %H:%M:%S UTC")
                            .to_string();
                        ui.label(time_str);
                        ui.end_row();

                        ui.label("Repeat:");
                        egui::ComboBox::from_label("")
                            .selected_text(match repeat_interval {
                                RepeatInterval::None => "Never".to_string(),
                                RepeatInterval::Minutes(m) => format!("Every {} minutes", m),
                                RepeatInterval::Hours(h) => format!("Every {} hours", h),
                                RepeatInterval::Days(d) => format!("Every {} days", d),
                                RepeatInterval::Weeks(w) => format!("Every {} weeks", w),
                                RepeatInterval::Months(m) => format!("Every {} months", m),
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut repeat_interval,
                                    RepeatInterval::None,
                                    "Never",
                                );
                                ui.selectable_value(
                                    &mut repeat_interval,
                                    RepeatInterval::Minutes(15),
                                    "Every 15 minutes",
                                );
                                ui.selectable_value(
                                    &mut repeat_interval,
                                    RepeatInterval::Minutes(30),
                                    "Every 30 minutes",
                                );
                                ui.selectable_value(
                                    &mut repeat_interval,
                                    RepeatInterval::Hours(1),
                                    "Every hour",
                                );
                                ui.selectable_value(
                                    &mut repeat_interval,
                                    RepeatInterval::Hours(6),
                                    "Every 6 hours",
                                );
                                ui.selectable_value(
                                    &mut repeat_interval,
                                    RepeatInterval::Days(1),
                                    "Daily",
                                );
                                ui.selectable_value(
                                    &mut repeat_interval,
                                    RepeatInterval::Weeks(1),
                                    "Weekly",
                                );
                                ui.selectable_value(
                                    &mut repeat_interval,
                                    RepeatInterval::Months(1),
                                    "Monthly",
                                );
                            });
                        ui.end_row();

                        ui.label("Max Runs (optional):");
                        let mut max_runs_str = max_runs.map(|n| n.to_string()).unwrap_or_default();
                        if ui.text_edit_singleline(&mut max_runs_str).changed() {
                            max_runs = max_runs_str.parse().ok();
                        }
                        ui.end_row();
                    });

                ui.separator();

                // Dependencies section
                ui.heading("Dependencies");
                ui.label("This job will only run after the selected jobs complete:");

                // Show available completed jobs to depend on
                let completed_jobs: Vec<_> = job_manager
                    .job_history
                    .completed_jobs
                    .iter()
                    .filter(|j| j.status == crate::jobs::JobStatus::Completed)
                    .collect();

                for job in &completed_jobs {
                    let mut selected = selected_dependencies.contains(&job.id);
                    let response = ui.checkbox(&mut selected, &job.name);
                    if response.clicked() || response.changed() {
                        if selected {
                            if !selected_dependencies.contains(&job.id) {
                                selected_dependencies.push(job.id.clone());
                            }
                        } else {
                            selected_dependencies.retain(|id| id != &job.id);
                        }
                    }
                }

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        should_close = true;
                    }

                    if ui.button("Create Schedule").clicked() {
                        should_create = true;
                    }
                });
            });

        // Handle actions after the window is closed
        if should_close {
            self.reset_create_dialog();
            self.show_create_dialog = false;
        } else if should_create {
            // Update self with the local values
            self.new_job_name = new_job_name;
            self.new_job_type = new_job_type;
            self.repeat_interval = repeat_interval;
            self.max_runs = max_runs;
            self.selected_dependencies = selected_dependencies;

            self.create_scheduled_job(job_manager);
            self.reset_create_dialog();
            self.show_create_dialog = false;
        }
    }

    fn create_scheduled_job(&self, job_manager: &mut JobManager) {
        // Create a basic job (in a real implementation, you'd want more job configuration)
        let job = Job::new(self.new_job_name.clone(), self.new_job_type.clone());

        // Create dependencies
        let dependencies = self
            .selected_dependencies
            .iter()
            .map(|job_id| crate::jobs::JobDependency {
                job_id: job_id.clone(),
                required_status: crate::jobs::JobStatus::Completed,
            })
            .collect();

        // Schedule the job
        let schedule_id = if matches!(self.repeat_interval, RepeatInterval::None) {
            job_manager.schedule_job(job, self.schedule_time)
        } else {
            job_manager.schedule_job_with_dependencies(job, self.schedule_time, dependencies)
        };

        println!("Created scheduled job with ID: {}", schedule_id);
    }

    fn reset_create_dialog(&mut self) {
        self.new_job_name.clear();
        self.new_job_type = JobType::GcodeFile;
        self.schedule_time = Utc::now() + chrono::Duration::hours(1);
        self.repeat_interval = RepeatInterval::None;
        self.max_runs = None;
        self.selected_dependencies.clear();
    }
}
