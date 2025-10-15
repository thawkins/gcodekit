//! Job management and queuing system.
//!
//! This module provides comprehensive job management functionality including
//! job creation, queuing, scheduling, history tracking, and performance analytics
//! for CNC machining operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;

pub mod manager;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JobType {
    GcodeFile,
    CAMOperation,
    Probing,
    Calibration,
    Maintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub job_type: JobType,
    pub status: JobStatus,
    pub priority: i32, // 1-10, higher = more important
    pub estimated_duration: Option<std::time::Duration>,
    pub actual_duration: Option<std::time::Duration>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub gcode_path: Option<PathBuf>,
    pub gcode_content: String,
    pub material: Option<String>,
    pub tool: Option<String>,
    pub notes: String,
    pub progress: f32, // 0.0 to 1.0
    pub error_message: Option<String>,
    pub last_completed_line: Option<usize>, // For job resumption
    pub can_resume: bool,                   // Whether this job can be resumed after interruption
    pub interrupted_at: Option<DateTime<Utc>>, // When the job was interrupted

    // Performance tracking fields
    pub total_lines: usize,                        // Total G-code lines in job
    pub lines_processed: usize,                    // Lines successfully processed
    pub bytes_processed: usize,                    // Bytes of G-code processed
    pub feed_rate_avg: Option<f32>,                // Average feed rate during job
    pub spindle_speed_avg: Option<f32>,            // Average spindle speed during job
    pub machine_time: Option<std::time::Duration>, // Actual machine operation time
    pub idle_time: Option<std::time::Duration>,    // Time spent idle during job
    pub retry_count: usize,                        // Number of retries due to errors
    pub pause_count: usize,                        // Number of times job was paused
    pub resume_count: usize,                       // Number of times job was resumed
}

impl Job {
    pub fn new(name: String, job_type: JobType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            job_type,
            status: JobStatus::Pending,
            priority: 5,
            estimated_duration: None,
            actual_duration: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            gcode_path: None,
            gcode_content: String::new(),
            material: None,
            tool: None,
            notes: String::new(),
            progress: 0.0,
            error_message: None,
            last_completed_line: None,
            can_resume: true, // By default, jobs can be resumed
            interrupted_at: None,
            // Performance tracking fields
            total_lines: 0,
            lines_processed: 0,
            bytes_processed: 0,
            feed_rate_avg: None,
            spindle_speed_avg: None,
            machine_time: None,
            idle_time: None,
            retry_count: 0,
            pause_count: 0,
            resume_count: 0,
        }
    }

    pub fn with_gcode(mut self, path: PathBuf) -> Self {
        self.gcode_path = Some(path);
        self
    }

    pub fn with_material(mut self, material: String) -> Self {
        self.material = Some(material);
        self
    }

    pub fn with_tool(mut self, tool: String) -> Self {
        self.tool = Some(tool);
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority.clamp(1, 10);
        self
    }

    pub fn start(&mut self) {
        self.status = JobStatus::Running;
        self.started_at = Some(Utc::now());
        self.progress = 0.0;
    }

    pub fn pause(&mut self) {
        if self.status == JobStatus::Running {
            self.status = JobStatus::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.status == JobStatus::Paused {
            self.status = JobStatus::Running;
        }
    }

    pub fn complete(&mut self) {
        self.status = JobStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.progress = 1.0;
        if let Some(started) = self.started_at {
            self.actual_duration = Some(
                Utc::now()
                    .signed_duration_since(started)
                    .to_std()
                    .unwrap_or_default(),
            );
        }
    }

    pub fn fail(&mut self, error: String) {
        self.status = JobStatus::Failed;
        self.error_message = Some(error);
        self.completed_at = Some(Utc::now());
        if let Some(started) = self.started_at {
            self.actual_duration = Some(
                Utc::now()
                    .signed_duration_since(started)
                    .to_std()
                    .unwrap_or_default(),
            );
        }
    }

    pub fn cancel(&mut self) {
        self.status = JobStatus::Cancelled;
        self.completed_at = Some(Utc::now());
        if let Some(started) = self.started_at {
            self.actual_duration = Some(
                Utc::now()
                    .signed_duration_since(started)
                    .to_std()
                    .unwrap_or_default(),
            );
        }
    }

    pub fn update_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, JobStatus::Running | JobStatus::Paused)
    }

    pub fn is_finished(&self) -> bool {
        matches!(
            self.status,
            JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
        )
    }

    /// Mark the job as interrupted and record the last completed line
    pub fn interrupt(&mut self, last_line: usize) {
        self.status = JobStatus::Paused;
        self.last_completed_line = Some(last_line);
        self.interrupted_at = Some(Utc::now());
        self.error_message = Some("Job interrupted due to communication error".to_string());
    }

    /// Check if the job can be resumed
    pub fn can_resume_job(&self) -> bool {
        self.can_resume
            && self.last_completed_line.is_some()
            && matches!(self.status, JobStatus::Paused)
    }

    /// Get the line number to resume from
    pub fn get_resume_line(&self) -> Option<usize> {
        if self.can_resume_job() {
            self.last_completed_line
        } else {
            None
        }
    }

    /// Resume the job from the interrupted point
    pub fn resume_job(&mut self) -> Result<(), String> {
        if !self.can_resume_job() {
            return Err("Job cannot be resumed".to_string());
        }

        self.status = JobStatus::Running;
        self.error_message = None;
        self.interrupted_at = None;
        self.resume_count += 1;
        Ok(())
    }

    /// Update performance tracking data
    pub fn update_performance_data(&mut self, lines_processed: usize, bytes_processed: usize) {
        self.lines_processed = lines_processed;
        self.bytes_processed = bytes_processed;
        self.total_lines = self.gcode_content.lines().count();
    }

    /// Record a retry attempt
    pub fn record_retry(&mut self) {
        self.retry_count += 1;
    }

    /// Record a pause event
    pub fn record_pause(&mut self) {
        self.pause_count += 1;
    }

    /// Update feed rate average
    pub fn update_feed_rate(&mut self, feed_rate: f32) {
        self.feed_rate_avg = Some(match self.feed_rate_avg {
            Some(current_avg) => (current_avg + feed_rate) / 2.0,
            None => feed_rate,
        });
    }

    /// Update spindle speed average
    pub fn update_spindle_speed(&mut self, spindle_speed: f32) {
        self.spindle_speed_avg = Some(match self.spindle_speed_avg {
            Some(current_avg) => (current_avg + spindle_speed) / 2.0,
            None => spindle_speed,
        });
    }

    /// Calculate efficiency metrics
    pub fn efficiency(&self) -> f32 {
        if let (Some(total_duration), Some(machine_time)) = (self.actual_duration, self.machine_time) {
            if total_duration.as_secs_f32() > 0.0 {
                machine_time.as_secs_f32() / total_duration.as_secs_f32()
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Calculate success rate based on retries and errors
    pub fn success_rate(&self) -> f32 {
        if self.status == JobStatus::Completed {
            let total_attempts = 1 + self.retry_count;
            1.0 / total_attempts as f32
        } else if self.status == JobStatus::Failed {
            0.0
        } else {
            0.5 // Partial completion
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobQueue {
    pub jobs: VecDeque<Job>,
    pub max_concurrent_jobs: usize,
    pub active_jobs: Vec<String>, // Job IDs currently running
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobAnalytics {
    pub total_jobs: usize,
    pub completed_jobs: usize,
    pub failed_jobs: usize,
    pub cancelled_jobs: usize,
    pub average_completion_time: Option<std::time::Duration>,
    pub average_efficiency: f32,
    pub average_success_rate: f32,
    pub total_machine_time: std::time::Duration,
    pub total_idle_time: std::time::Duration,
    pub most_used_material: Option<String>,
    pub most_used_tool: Option<String>,
    pub jobs_by_type: std::collections::HashMap<JobType, usize>,
    pub jobs_by_day: std::collections::HashMap<String, usize>, // Date -> count
}

impl Default for JobAnalytics {
    fn default() -> Self {
        Self {
            total_jobs: 0,
            completed_jobs: 0,
            failed_jobs: 0,
            cancelled_jobs: 0,
            average_completion_time: None,
            average_efficiency: 0.0,
            average_success_rate: 0.0,
            total_machine_time: std::time::Duration::from_secs(0),
            total_idle_time: std::time::Duration::from_secs(0),
            most_used_material: None,
            most_used_tool: None,
            jobs_by_type: std::collections::HashMap::new(),
            jobs_by_day: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobHistory {
    pub completed_jobs: Vec<Job>,
    pub analytics: JobAnalytics,
    pub max_history_size: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RepeatInterval {
    None,         // Run once
    Minutes(u32), // Repeat every N minutes
    Hours(u32),   // Repeat every N hours
    Days(u32),    // Repeat every N days
    Weeks(u32),   // Repeat every N weeks
    Months(u32),  // Repeat every N months
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDependency {
    pub job_id: String,             // ID of the job this depends on
    pub required_status: JobStatus, // Status the dependency job must have (usually Completed)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    pub job: Job,                         // The base job
    pub schedule_id: String,              // Unique ID for this scheduled job
    pub start_time: DateTime<Utc>,        // When to start the job
    pub repeat_interval: RepeatInterval,  // How often to repeat
    pub dependencies: Vec<JobDependency>, // Jobs this depends on
    pub enabled: bool,                    // Whether this schedule is active
    pub last_run: Option<DateTime<Utc>>,  // When this was last executed
    pub next_run: Option<DateTime<Utc>>,  // When this will next run
    pub created_at: DateTime<Utc>,        // When this schedule was created
    pub max_runs: Option<u32>,            // Maximum number of times to run (None = unlimited)
    pub run_count: u32,                   // How many times this has been run
}

impl Default for JobHistory {
    fn default() -> Self {
        Self {
            completed_jobs: Vec::new(),
            analytics: JobAnalytics::default(),
            max_history_size: 1000, // Keep last 1000 jobs by default
        }
    }
}

impl ScheduledJob {
    pub fn new(job: Job, start_time: DateTime<Utc>) -> Self {
        let schedule_id = uuid::Uuid::new_v4().to_string();
        let next_run = Some(start_time);

        Self {
            job,
            schedule_id,
            start_time,
            repeat_interval: RepeatInterval::None,
            dependencies: Vec::new(),
            enabled: true,
            last_run: None,
            next_run,
            created_at: Utc::now(),
            max_runs: None,
            run_count: 0,
        }
    }

    pub fn with_repeat_interval(mut self, interval: RepeatInterval) -> Self {
        self.repeat_interval = interval;
        self.update_next_run();
        self
    }

    pub fn with_dependencies(mut self, dependencies: Vec<JobDependency>) -> Self {
        self.dependencies = dependencies;
        self
    }

    pub fn with_max_runs(mut self, max_runs: u32) -> Self {
        self.max_runs = Some(max_runs);
        self
    }

    pub fn enable(mut self) -> Self {
        self.enabled = true;
        self
    }

    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Check if this scheduled job should run now
    pub fn should_run(&self, current_time: DateTime<Utc>) -> bool {
        if !self.enabled {
            return false;
        }

        // Check if we've reached max runs
        if let Some(max) = self.max_runs {
            if self.run_count >= max {
                return false;
            }
        }

        // Check if it's time to run
        if let Some(next_run) = self.next_run {
            current_time >= next_run
        } else {
            false
        }
    }

    /// Check if all dependencies are satisfied
    pub fn dependencies_satisfied(&self, completed_job_ids: &[String]) -> bool {
        self.dependencies
            .iter()
            .all(|dep| completed_job_ids.contains(&dep.job_id))
    }

    /// Mark this job as executed and update next run time
    pub fn mark_executed(&mut self, execution_time: DateTime<Utc>) {
        self.last_run = Some(execution_time);
        self.run_count += 1;
        self.update_next_run();
    }

    /// Update the next run time based on repeat interval
    fn update_next_run(&mut self) {
        if let Some(last_run) = self.last_run {
            self.next_run = match self.repeat_interval {
                RepeatInterval::None => None,
                RepeatInterval::Minutes(mins) => {
                    Some(last_run + chrono::Duration::minutes(mins as i64))
                }
                RepeatInterval::Hours(hours) => {
                    Some(last_run + chrono::Duration::hours(hours as i64))
                }
                RepeatInterval::Days(days) => Some(last_run + chrono::Duration::days(days as i64)),
                RepeatInterval::Weeks(weeks) => {
                    Some(last_run + chrono::Duration::weeks(weeks as i64))
                }
                RepeatInterval::Months(months) => {
                    // Approximate months as 30 days
                    Some(last_run + chrono::Duration::days(months as i64 * 30))
                }
            };
        } else {
            // First run hasn't happened yet
            self.next_run = Some(self.start_time);
        }
    }

    /// Get the time until next run
    pub fn time_until_next_run(&self, current_time: DateTime<Utc>) -> Option<std::time::Duration> {
        self.next_run.and_then(|next| {
            if next > current_time {
                Some((next - current_time).to_std().unwrap_or_default())
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobScheduler {
    pub scheduled_jobs: Vec<ScheduledJob>,
    pub check_interval_seconds: u64, // How often to check for jobs to run
}

impl Default for JobScheduler {
    fn default() -> Self {
        Self {
            scheduled_jobs: Vec::new(),
            check_interval_seconds: 60, // Check every minute by default
        }
    }
}

impl JobScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_check_interval(mut self, seconds: u64) -> Self {
        self.check_interval_seconds = seconds;
        self
    }

    /// Add a scheduled job
    pub fn add_scheduled_job(&mut self, scheduled_job: ScheduledJob) {
        self.scheduled_jobs.push(scheduled_job);
    }

    /// Remove a scheduled job by schedule ID
    pub fn remove_scheduled_job(&mut self, schedule_id: &str) -> Option<ScheduledJob> {
        if let Some(pos) = self
            .scheduled_jobs
            .iter()
            .position(|sj| sj.schedule_id == schedule_id)
        {
            Some(self.scheduled_jobs.remove(pos))
        } else {
            None
        }
    }

    /// Get a scheduled job by schedule ID
    pub fn get_scheduled_job(&self, schedule_id: &str) -> Option<&ScheduledJob> {
        self.scheduled_jobs
            .iter()
            .find(|sj| sj.schedule_id == schedule_id)
    }

    /// Get a mutable scheduled job by schedule ID
    pub fn get_scheduled_job_mut(&mut self, schedule_id: &str) -> Option<&mut ScheduledJob> {
        self.scheduled_jobs
            .iter_mut()
            .find(|sj| sj.schedule_id == schedule_id)
    }

    /// Enable a scheduled job
    pub fn enable_schedule(&mut self, schedule_id: &str) -> Result<(), String> {
        if let Some(sj) = self.get_scheduled_job_mut(schedule_id) {
            sj.enabled = true;
            Ok(())
        } else {
            Err(format!("Scheduled job {} not found", schedule_id))
        }
    }

    /// Disable a scheduled job
    pub fn disable_schedule(&mut self, schedule_id: &str) -> Result<(), String> {
        if let Some(sj) = self.get_scheduled_job_mut(schedule_id) {
            sj.enabled = false;
            Ok(())
        } else {
            Err(format!("Scheduled job {} not found", schedule_id))
        }
    }

    /// Get all enabled scheduled jobs
    pub fn get_enabled_schedules(&self) -> Vec<&ScheduledJob> {
        self.scheduled_jobs.iter().filter(|sj| sj.enabled).collect()
    }

    /// Get jobs that should run now, considering dependencies
    pub fn get_jobs_to_run(
        &self,
        current_time: DateTime<Utc>,
        completed_job_ids: &[String],
    ) -> Vec<&ScheduledJob> {
        self.scheduled_jobs
            .iter()
            .filter(|sj| {
                sj.should_run(current_time) && sj.dependencies_satisfied(completed_job_ids)
            })
            .collect()
    }

    /// Mark a scheduled job as executed
    pub fn mark_job_executed(
        &mut self,
        schedule_id: &str,
        execution_time: DateTime<Utc>,
    ) -> Result<(), String> {
        if let Some(sj) = self.get_scheduled_job_mut(schedule_id) {
            sj.mark_executed(execution_time);
            Ok(())
        } else {
            Err(format!("Scheduled job {} not found", schedule_id))
        }
    }

    /// Get the next scheduled run time across all enabled jobs
    pub fn get_next_run_time(&self) -> Option<DateTime<Utc>> {
        self.get_enabled_schedules()
            .iter()
            .filter_map(|sj| sj.next_run)
            .min()
    }

    /// Get jobs scheduled to run within the next time window
    pub fn get_upcoming_jobs(&self, within_duration: std::time::Duration) -> Vec<&ScheduledJob> {
        let current_time = Utc::now();
        let cutoff_time = current_time
            + chrono::Duration::from_std(within_duration).unwrap_or(chrono::Duration::hours(1));

        self.get_enabled_schedules()
            .into_iter()
            .filter(|sj| sj.next_run.is_some_and(|next| next <= cutoff_time))
            .collect()
    }

    /// Save scheduler state to JSON file
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load scheduler state from JSON file
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let scheduler: JobScheduler = serde_json::from_str(&json)?;
        Ok(scheduler)
    }
}

impl JobHistory {
    pub fn new(max_history_size: usize) -> Self {
        Self {
            completed_jobs: Vec::new(),
            analytics: JobAnalytics::default(),
            max_history_size,
        }
    }

    /// Add a completed job to history and update analytics
    pub fn add_completed_job(&mut self, job: Job) {
        // Only add finished jobs
        if !job.is_finished() {
            return;
        }

        // Add to history
        self.completed_jobs.push(job.clone());

        // Maintain max history size
        if self.completed_jobs.len() > self.max_history_size {
            self.completed_jobs.remove(0); // Remove oldest
        }

        // Update analytics
        self.update_analytics(&job);
    }

    /// Update analytics with a new job
    fn update_analytics(&mut self, job: &Job) {
        self.analytics.total_jobs += 1;

        match job.status {
            JobStatus::Completed => self.analytics.completed_jobs += 1,
            JobStatus::Failed => self.analytics.failed_jobs += 1,
            JobStatus::Cancelled => self.analytics.cancelled_jobs += 1,
            _ => {}
        }

        // Update job type counts
        *self
            .analytics
            .jobs_by_type
            .entry(job.job_type.clone())
            .or_insert(0) += 1;

        // Update daily counts
        if let Some(completed_at) = job.completed_at {
            let date_key = completed_at.format("%Y-%m-%d").to_string();
            *self.analytics.jobs_by_day.entry(date_key).or_insert(0) += 1;
        }

        // Update material/tool usage
        if let Some(material) = &job.material {
            self.analytics.most_used_material = Some(material.clone());
        }
        if let Some(tool) = &job.tool {
            self.analytics.most_used_tool = Some(tool.clone());
        }

        // Update timing analytics
        if let Some(duration) = job.actual_duration {
            self.analytics.total_machine_time += duration;
        }
        if let Some(idle) = job.idle_time {
            self.analytics.total_idle_time += idle;
        }

        // Recalculate averages
        self.recalculate_averages();
    }

    /// Recalculate average metrics
    fn recalculate_averages(&mut self) {
        let completed_jobs: Vec<&Job> = self
            .completed_jobs
            .iter()
            .filter(|j| j.status == JobStatus::Completed)
            .collect();

        if completed_jobs.is_empty() {
            return;
        }

        // Average completion time
        let total_duration: std::time::Duration = completed_jobs
            .iter()
            .filter_map(|j| j.actual_duration)
            .sum();
        let avg_duration = total_duration / completed_jobs.len() as u32;
        self.analytics.average_completion_time = Some(avg_duration);

        // Average efficiency
        let total_efficiency: f32 = completed_jobs.iter().map(|j| j.efficiency()).sum();
        self.analytics.average_efficiency = total_efficiency / completed_jobs.len() as f32;

        // Average success rate
        let total_success_rate: f32 = completed_jobs.iter().map(|j| j.success_rate()).sum();
        self.analytics.average_success_rate = total_success_rate / completed_jobs.len() as f32;
    }

    /// Get jobs completed in the last N days
    pub fn get_recent_jobs(&self, days: i64) -> Vec<&Job> {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        self.completed_jobs
            .iter()
            .filter(|job| job.completed_at.is_some_and(|dt| dt > cutoff))
            .collect()
    }

    /// Get jobs by type
    pub fn get_jobs_by_type(&self, job_type: &JobType) -> Vec<&Job> {
        self.completed_jobs
            .iter()
            .filter(|job| job.job_type == *job_type)
            .collect()
    }

    /// Get performance summary for a date range
    pub fn get_performance_summary(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> JobAnalytics {
        let jobs_in_range: Vec<&Job> = self
            .completed_jobs
            .iter()
            .filter(|job| {
                job.completed_at
                    .is_some_and(|dt| dt >= start_date && dt <= end_date)
            })
            .collect();

        let mut summary = JobAnalytics::default();

        for job in jobs_in_range {
            summary.total_jobs += 1;
            match job.status {
                JobStatus::Completed => summary.completed_jobs += 1,
                JobStatus::Failed => summary.failed_jobs += 1,
                JobStatus::Cancelled => summary.cancelled_jobs += 1,
                _ => {}
            }

            if let Some(duration) = job.actual_duration {
                summary.total_machine_time += duration;
            }
            if let Some(idle) = job.idle_time {
                summary.total_idle_time += idle;
            }
        }

        summary
    }

    /// Clear all history
    pub fn clear_history(&mut self) {
        self.completed_jobs.clear();
        self.analytics = JobAnalytics::default();
    }

    /// Export history to JSON
    pub fn export_to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Import history from JSON
    pub fn import_from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl Default for JobQueue {
    fn default() -> Self {
        Self {
            jobs: VecDeque::new(),
            max_concurrent_jobs: 1, // Most CNC machines can only run one job at a time
            active_jobs: Vec::new(),
        }
    }
}

impl JobQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_job(&mut self, job: Job) {
        self.jobs.push_back(job);
    }

    pub fn remove_job(&mut self, job_id: &str) -> Option<Job> {
        if let Some(job) = self.get_job(job_id) {
            // Don't remove active jobs
            if !self.active_jobs.contains(&job.id) {
                if let Some(pos) = self.jobs.iter().position(|j| j.id == job_id) {
                    return self.jobs.remove(pos);
                }
            }
        }
        None
    }

    pub fn get_job(&self, job_id: &str) -> Option<&Job> {
        self.jobs.iter().find(|j| j.id == job_id)
    }

    pub fn get_job_mut(&mut self, job_id: &str) -> Option<&mut Job> {
        self.jobs.iter_mut().find(|j| j.id == job_id)
    }

    pub fn get_next_pending_job(&self) -> Option<&Job> {
        self.jobs
            .iter()
            .filter(|j| j.status == JobStatus::Pending)
            .max_by_key(|j| j.priority)
    }

    pub fn start_job(&mut self, job_id: &str) -> Result<(), String> {
        if self.active_jobs.len() >= self.max_concurrent_jobs {
            return Err("Maximum concurrent jobs reached".to_string());
        }

        if let Some(job) = self.get_job_mut(job_id) {
            if job.status == JobStatus::Pending {
                job.start();
                self.active_jobs.push(job_id.to_string());
                Ok(())
            } else {
                Err(format!("Job {} is not in pending state", job_id))
            }
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }

    pub fn pause_job(&mut self, job_id: &str) -> Result<(), String> {
        if let Some(job) = self.get_job_mut(job_id) {
            job.pause();
            Ok(())
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }

    pub fn resume_job(&mut self, job_id: &str) -> Result<(), String> {
        if let Some(job) = self.get_job_mut(job_id) {
            job.resume();
            Ok(())
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }

    pub fn complete_job(&mut self, job_id: &str) -> Result<(), String> {
        if let Some(job) = self.get_job_mut(job_id) {
            job.complete();
            self.active_jobs.retain(|id| id != job_id);
            Ok(())
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }

    pub fn fail_job(&mut self, job_id: &str, error: String) -> Result<(), String> {
        if let Some(job) = self.get_job_mut(job_id) {
            job.fail(error);
            self.active_jobs.retain(|id| id != job_id);
            Ok(())
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }

    pub fn cancel_job(&mut self, job_id: &str) -> Result<(), String> {
        if let Some(job) = self.get_job_mut(job_id) {
            job.cancel();
            self.active_jobs.retain(|id| id != job_id);
            Ok(())
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }

    pub fn update_job_progress(&mut self, job_id: &str, progress: f32) -> Result<(), String> {
        if let Some(job) = self.get_job_mut(job_id) {
            job.update_progress(progress);
            Ok(())
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }

    pub fn get_active_jobs(&self) -> Vec<&Job> {
        self.active_jobs
            .iter()
            .filter_map(|id| self.get_job(id))
            .collect()
    }

    pub fn get_pending_jobs(&self) -> Vec<&Job> {
        self.jobs
            .iter()
            .filter(|j| j.status == JobStatus::Pending)
            .collect()
    }

    pub fn get_completed_jobs(&self) -> Vec<&Job> {
        self.jobs
            .iter()
            .filter(|j| j.status == JobStatus::Completed)
            .collect()
    }

    pub fn get_failed_jobs(&self) -> Vec<&Job> {
        self.jobs
            .iter()
            .filter(|j| j.status == JobStatus::Failed)
            .collect()
    }

    pub fn clear_completed_jobs(&mut self) {
        self.jobs.retain(|j| !j.is_finished());
    }

    pub fn reorder_jobs(&mut self, job_ids: Vec<String>) -> Result<(), String> {
        if job_ids.len() != self.jobs.len() {
            return Err("Job ID count doesn't match queue size".to_string());
        }

        let mut new_queue = VecDeque::new();
        for id in job_ids {
            if let Some(job) = self.jobs.iter().find(|j| j.id == id).cloned() {
                new_queue.push_back(job);
            } else {
                return Err(format!("Job {} not found in queue", id));
            }
        }

        self.jobs = new_queue;
        Ok(())
    }

    /// Save the job queue to a JSON file
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load a job queue from a JSON file
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let mut queue: JobQueue = serde_json::from_str(&json)?;

        // Reset active jobs on load (they should be restarted manually)
        queue.active_jobs.clear();

        // Reset any running jobs to pending
        for job in &mut queue.jobs {
            if job.status == JobStatus::Running {
                job.status = JobStatus::Pending;
                job.started_at = None;
                job.progress = 0.0; // Reset progress for restarted jobs
            }
        }

        Ok(queue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation() {
        let job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        assert_eq!(job.name, "Test Job");
        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.priority, 5);
        assert!(job.id.len() > 0);
    }

    #[test]
    fn test_job_lifecycle() {
        let mut job = Job::new("Test Job".to_string(), JobType::GcodeFile);

        // Start job
        job.start();
        assert_eq!(job.status, JobStatus::Running);
        assert!(job.started_at.is_some());

        // Update progress
        job.update_progress(0.5);
        assert_eq!(job.progress, 0.5);

        // Complete job
        job.complete();
        assert_eq!(job.status, JobStatus::Completed);
        assert!(job.completed_at.is_some());
        assert!(job.actual_duration.is_some());
    }

    #[test]
    fn test_job_queue() {
        let mut queue = JobQueue::new();

        let job1 = Job::new("Job 1".to_string(), JobType::GcodeFile).with_priority(1);
        let job2 = Job::new("Job 2".to_string(), JobType::GcodeFile).with_priority(10);

        queue.add_job(job1);
        queue.add_job(job2);

        // Should get high priority job first
        let next = queue.get_next_pending_job().expect("expected next pending job");
        assert_eq!(next.name, "Job 2");
        assert_eq!(next.priority, 10);
    }

    #[test]
    fn test_job_resumption_integration() {
        let mut job_manager = manager::JobManager::default();

        // Create and start a job
        let job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        job_manager.job_queue.add_job(job);
        let job_id = job_manager.job_queue.jobs[0].id.clone();

        // Start the job
        assert!(job_manager.start_job(&job_id).is_ok());

        // Simulate sending some G-code lines successfully
        let gcode_content = "G1 X10\nG1 Y20\nG1 Z30\nG1 X40".to_string();
        let lines: Vec<String> = gcode_content.lines().map(|s| s.to_string()).collect();

        // Send first two lines successfully
        for i in 0..2 {
            if let Some(job) = job_manager.job_queue.get_job_mut(&job_id) {
                job.last_completed_line = Some(i);
                job.update_progress((i as f32 + 1.0) / lines.len() as f32);
            }
        }

        // Simulate an error on the third line
        if let Some(job) = job_manager.job_queue.get_job_mut(&job_id) {
            job.interrupt(2); // Interrupt at line 2 (0-indexed)
        }

        // Verify job is interrupted
        let job = job_manager.job_queue.get_job(&job_id).expect("expected job in queue");
        assert_eq!(job.status, JobStatus::Paused);
        assert_eq!(job.last_completed_line, Some(2));
        assert!(job.can_resume_job());

        // Test resume functionality
        let resume_line = job_manager.resume_job(&job_id).expect("expected resume line");
        assert_eq!(resume_line, 2);

        // Verify job is running again
        let job = job_manager.job_queue.get_job(&job_id).expect("expected job in queue");
        assert_eq!(job.status, JobStatus::Running);
        assert_eq!(job.last_completed_line, Some(2)); // Should still have the resume point
    }

    #[test]
    fn test_job_resumption_with_invalid_job() {
        let mut job_manager = super::manager::JobManager::default();

        // Try to resume non-existent job
        assert!(job_manager.resume_job("invalid-id").is_err());
    }

    #[test]
    fn test_job_queue_save_load() {
        let mut queue = JobQueue::new();

        // Add some test jobs
        let job1 = Job::new("Test Job 1".to_string(), JobType::GcodeFile);
        let job2 = Job::new("Test Job 2".to_string(), JobType::CAMOperation);
        queue.add_job(job1);
        queue.add_job(job2);

        // Start one job
        let job_id = queue.jobs[0].id.clone();
        queue.start_job(&job_id).expect("failed to start job");

        // Save to temporary file
        let temp_path = std::env::temp_dir().join("test_job_queue.json");
        queue
            .save_to_file(&temp_path)
            .expect("Failed to save job queue");

        // Load from file
        let loaded_queue = JobQueue::load_from_file(&temp_path).expect("Failed to load job queue");

        // Verify loaded queue
        assert_eq!(loaded_queue.jobs.len(), 2);
        assert_eq!(loaded_queue.jobs[0].name, "Test Job 1");
        assert_eq!(loaded_queue.jobs[1].name, "Test Job 2");

        // Active jobs should be cleared on load
        assert!(loaded_queue.active_jobs.is_empty());

        // Running jobs should become pending
        assert_eq!(loaded_queue.jobs[0].status, JobStatus::Pending);
        assert_eq!(loaded_queue.jobs[1].status, JobStatus::Pending);

        // Clean up
        std::fs::remove_file(&temp_path).ok();
    }

    #[test]
    fn test_job_queue_save_load_empty() {
        let queue = JobQueue::new();

        // Save empty queue
        let temp_path = std::env::temp_dir().join("test_empty_job_queue.json");
        queue
            .save_to_file(&temp_path)
            .expect("Failed to save empty job queue");

        // Load empty queue
        let loaded_queue =
            JobQueue::load_from_file(&temp_path).expect("Failed to load empty job queue");

        // Verify empty queue
        assert_eq!(loaded_queue.jobs.len(), 0);
        assert!(loaded_queue.active_jobs.is_empty());

        // Clean up
        std::fs::remove_file(&temp_path).ok();
    }
}
