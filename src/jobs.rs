use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub material: Option<String>,
    pub tool: Option<String>,
    pub notes: String,
    pub progress: f32, // 0.0 to 1.0
    pub error_message: Option<String>,
    pub last_completed_line: Option<usize>, // For job resumption
    pub can_resume: bool,                   // Whether this job can be resumed after interruption
    pub interrupted_at: Option<DateTime<Utc>>, // When the job was interrupted
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
            material: None,
            tool: None,
            notes: String::new(),
            progress: 0.0,
            error_message: None,
            last_completed_line: None,
            can_resume: true, // By default, jobs can be resumed
            interrupted_at: None,
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
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobQueue {
    pub jobs: VecDeque<Job>,
    pub max_concurrent_jobs: usize,
    pub active_jobs: Vec<String>, // Job IDs currently running
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
            if !self.active_jobs.contains(&job.id)
                && let Some(pos) = self.jobs.iter().position(|j| j.id == job_id)
            {
                return self.jobs.remove(pos);
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
        let next = queue.get_next_pending_job().unwrap();
        assert_eq!(next.name, "Job 2");
        assert_eq!(next.priority, 10);
    }
}
