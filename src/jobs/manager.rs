use super::{
    Job, JobDependency, JobHistory, JobQueue, JobScheduler, JobStatus, JobType, RepeatInterval,
    ScheduledJob,
};
use chrono::{DateTime, Utc};
use uuid;

pub struct JobManager {
    pub job_queue: JobQueue,
    pub job_history: JobHistory,
    pub scheduler: JobScheduler,
}

impl JobManager {
    pub fn new() -> Self {
        Self {
            job_queue: JobQueue::new(),
            job_history: JobHistory::default(),
            scheduler: JobScheduler::new(),
        }
    }

    /// Create a job from generated G-code content
    pub fn create_job_from_generated_gcode(
        &mut self,
        name: &str,
        job_type: JobType,
        gcode_content: &str,
        selected_material: Option<&str>,
    ) {
        if !gcode_content.is_empty() {
            let mut job = Job::new(name.to_string(), job_type);
            if let Some(material) = selected_material {
                job = job.with_material(material.to_string());
            }
            // For generated G-code, we don't have a file path, so we'll store it as content
            // The job system would need to be extended to handle in-memory G-code
            job.gcode_content = gcode_content.to_string();
            self.job_queue.add_job(job);
        }
    }

    /// Start a job by ID
    pub fn start_job(&mut self, job_id: &str) -> Result<(), String> {
        self.job_queue.start_job(job_id)
    }

    /// Resume a job by ID
    pub fn resume_job(&mut self, job_id: &str) -> Result<usize, String> {
        // Get the resume line
        let resume_line = {
            let job = self.job_queue.get_job(job_id).ok_or("Job not found")?;
            job.get_resume_line().ok_or("Job cannot be resumed")?
        };

        // Resume the job
        self.job_queue.resume_job(job_id)?;

        Ok(resume_line)
    }

    /// Complete a job by ID
    pub fn complete_job(&mut self, job_id: &str) -> Result<(), String> {
        // Get the job before completing it
        let job = self.job_queue.get_job(job_id).cloned();

        // Complete the job
        self.job_queue.complete_job(job_id)?;

        // Add to history if we have the job
        if let Some(completed_job) = job {
            self.job_history.add_completed_job(completed_job);
        }

        Ok(())
    }

    /// Fail a job by ID with an error message
    pub fn fail_job(&mut self, job_id: &str, error: String) -> Result<(), String> {
        // Get the job before failing it
        let job = self.job_queue.get_job(job_id).cloned();

        // Fail the job
        self.job_queue.fail_job(job_id, error)?;

        // Add to history if we have the job
        if let Some(failed_job) = job {
            self.job_history.add_completed_job(failed_job);
        }

        Ok(())
    }

    /// Update job progress
    pub fn update_job_progress(&mut self, job_id: &str, progress: f32) -> Result<(), String> {
        self.job_queue.update_job_progress(job_id, progress)
    }

    /// Get the current active job
    pub fn get_current_job(&self) -> Option<&Job> {
        self.job_queue.get_active_jobs().first().copied()
    }

    /// Get the current job ID
    pub fn get_current_job_id(&self) -> Option<String> {
        self.get_current_job().map(|job| job.id.clone())
    }

    /// Save the job queue to a file
    pub fn save_jobs_to_file(
        &self,
        path: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.job_queue.save_to_file(path)
    }

    /// Load jobs from a file
    pub fn load_jobs_from_file(
        &self,
        path: &std::path::Path,
    ) -> Result<super::JobQueue, Box<dyn std::error::Error>> {
        super::JobQueue::load_from_file(path)
    }

    /// Replace the current job queue with a loaded one
    pub fn replace_job_queue(&mut self, new_queue: super::JobQueue) {
        self.job_queue = new_queue;
    }

    /// Get job history analytics
    pub fn get_job_analytics(&self) -> &super::JobAnalytics {
        &self.job_history.analytics
    }

    /// Get recent completed jobs
    pub fn get_recent_jobs(&self, days: i64) -> Vec<&super::Job> {
        self.job_history.get_recent_jobs(days)
    }

    /// Get jobs by type
    pub fn get_jobs_by_type(&self, job_type: &super::JobType) -> Vec<&super::Job> {
        self.job_history.get_jobs_by_type(job_type)
    }

    /// Get performance summary for date range
    pub fn get_performance_summary(
        &self,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> super::JobAnalytics {
        self.job_history
            .get_performance_summary(start_date, end_date)
    }

    /// Clear job history
    pub fn clear_job_history(&mut self) {
        self.job_history.clear_history();
    }

    /// Export job history to JSON
    pub fn export_job_history(&self) -> Result<String, serde_json::Error> {
        self.job_history.export_to_json()
    }

    /// Import job history from JSON
    pub fn import_job_history(&mut self, json: &str) -> Result<(), serde_json::Error> {
        let history = super::JobHistory::import_from_json(json)?;
        self.job_history = history;
        Ok(())
    }

    /// Pause a job by ID
    pub fn pause_job(&mut self, job_id: &str) -> Result<(), String> {
        self.job_queue.pause_job(job_id)
    }

    /// Cancel a job by ID
    pub fn cancel_job(&mut self, job_id: &str) -> Result<(), String> {
        // Get the job before cancelling it
        let job = self.job_queue.get_job(job_id).cloned();

        // Cancel the job
        self.job_queue.cancel_job(job_id)?;

        // Add to history if we have the job
        if let Some(cancelled_job) = job {
            self.job_history.add_completed_job(cancelled_job);
        }

        Ok(())
    }
}

impl Default for JobManager {
    fn default() -> Self {
        Self::new()
    }
}

impl JobManager {
    /// Schedule a job to run at a specific time
    pub fn schedule_job(&mut self, job: Job, start_time: DateTime<Utc>) -> String {
        let scheduled_job = ScheduledJob::new(job, start_time);
        let schedule_id = scheduled_job.schedule_id.clone();
        self.scheduler.add_scheduled_job(scheduled_job);
        schedule_id
    }

    /// Schedule a job with repeat interval
    pub fn schedule_recurring_job(
        &mut self,
        job: Job,
        start_time: DateTime<Utc>,
        repeat_interval: RepeatInterval,
    ) -> String {
        let scheduled_job =
            ScheduledJob::new(job, start_time).with_repeat_interval(repeat_interval);
        let schedule_id = scheduled_job.schedule_id.clone();
        self.scheduler.add_scheduled_job(scheduled_job);
        schedule_id
    }

    /// Schedule a job with dependencies
    pub fn schedule_job_with_dependencies(
        &mut self,
        job: Job,
        start_time: DateTime<Utc>,
        dependencies: Vec<JobDependency>,
    ) -> String {
        let scheduled_job = ScheduledJob::new(job, start_time).with_dependencies(dependencies);
        let schedule_id = scheduled_job.schedule_id.clone();
        self.scheduler.add_scheduled_job(scheduled_job);
        schedule_id
    }

    /// Cancel a scheduled job
    pub fn cancel_schedule(&mut self, schedule_id: &str) -> Result<(), String> {
        self.scheduler.remove_scheduled_job(schedule_id);
        Ok(())
    }

    /// Enable a scheduled job
    pub fn enable_schedule(&mut self, schedule_id: &str) -> Result<(), String> {
        self.scheduler.enable_schedule(schedule_id)
    }

    /// Disable a scheduled job
    pub fn disable_schedule(&mut self, schedule_id: &str) -> Result<(), String> {
        self.scheduler.disable_schedule(schedule_id)
    }

    /// Get all scheduled jobs
    pub fn get_scheduled_jobs(&self) -> &Vec<ScheduledJob> {
        &self.scheduler.scheduled_jobs
    }

    /// Get a specific scheduled job
    pub fn get_scheduled_job(&self, schedule_id: &str) -> Option<&ScheduledJob> {
        self.scheduler.get_scheduled_job(schedule_id)
    }

    /// Check for and execute scheduled jobs that are ready
    pub fn process_scheduled_jobs(&mut self) -> Result<Vec<String>, String> {
        let current_time = Utc::now();

        // Get completed job IDs for dependency checking
        let completed_job_ids: Vec<String> = self
            .job_history
            .completed_jobs
            .iter()
            .filter(|j| j.status == JobStatus::Completed)
            .map(|j| j.id.clone())
            .collect();

        // Get schedule IDs of jobs that should run now
        let schedule_ids_to_run: Vec<String> = self
            .scheduler
            .scheduled_jobs
            .iter()
            .filter(|sj| {
                sj.should_run(current_time) && sj.dependencies_satisfied(&completed_job_ids)
            })
            .map(|sj| sj.schedule_id.clone())
            .collect();

        let mut executed_schedule_ids = Vec::new();

        for schedule_id in schedule_ids_to_run {
            // Get the scheduled job
            if let Some(scheduled_job) = self.scheduler.get_scheduled_job(&schedule_id) {
                // Create a copy of the job for the queue
                let mut job_copy = scheduled_job.job.clone();
                job_copy.id = uuid::Uuid::new_v4().to_string(); // New ID for the queue job

                // Add to job queue
                self.job_queue.add_job(job_copy);

                // Mark as executed in scheduler
                if let Err(e) = self.scheduler.mark_job_executed(&schedule_id, current_time) {
                    eprintln!("Warning: Failed to mark job as executed: {}", e);
                }

                executed_schedule_ids.push(schedule_id);
            }
        }

        Ok(executed_schedule_ids)
    }

    /// Get upcoming scheduled jobs within a time window
    pub fn get_upcoming_scheduled_jobs(
        &self,
        within_duration: std::time::Duration,
    ) -> Vec<&ScheduledJob> {
        self.scheduler.get_upcoming_jobs(within_duration)
    }

    /// Get the next scheduled run time
    pub fn get_next_scheduled_run(&self) -> Option<DateTime<Utc>> {
        self.scheduler.get_next_run_time()
    }

    /// Save scheduler state to file
    pub fn save_scheduler_to_file(
        &self,
        path: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.scheduler.save_to_file(path)
    }

    /// Load scheduler state from file
    pub fn load_scheduler_from_file(
        &mut self,
        path: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let scheduler = JobScheduler::load_from_file(path)?;
        self.scheduler = scheduler;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_manager_new() {
        let manager = JobManager::new();
        assert_eq!(manager.job_queue.jobs.len(), 0);
    }

    #[test]
    fn test_create_job_from_generated_gcode() {
        let mut manager = JobManager::new();

        manager.create_job_from_generated_gcode(
            "Generated Toolpath",
            JobType::CAMOperation,
            "G1 X10 Y20\nG1 X20 Y30",
            Some("Oak"),
        );

        assert_eq!(manager.job_queue.jobs.len(), 1);
        let job = &manager.job_queue.jobs[0];
        assert_eq!(job.name, "Generated Toolpath");
        assert_eq!(job.job_type, JobType::CAMOperation);
        assert_eq!(job.material, Some("Oak".to_string()));
        assert_eq!(job.gcode_content, "G1 X10 Y20\nG1 X20 Y30");
    }

    #[test]
    fn test_create_job_from_generated_gcode_empty() {
        let mut manager = JobManager::new();

        // Empty G-code should not create a job
        manager.create_job_from_generated_gcode("Empty Job", JobType::GcodeFile, "", None);

        assert_eq!(manager.job_queue.jobs.len(), 0);
    }

    #[test]
    fn test_create_job_from_generated_gcode_no_material() {
        let mut manager = JobManager::new();

        manager.create_job_from_generated_gcode(
            "No Material Job",
            JobType::GcodeFile,
            "G0 X0 Y0",
            None,
        );

        assert_eq!(manager.job_queue.jobs.len(), 1);
        let job = &manager.job_queue.jobs[0];
        assert_eq!(job.material, None);
    }

    #[test]
    fn test_start_job() {
        let mut manager = JobManager::new();
        manager.create_job_from_generated_gcode("Test Job", JobType::GcodeFile, "G1 X10", None);

        let job_id = manager.job_queue.jobs[0].id.clone();
        let result = manager.start_job(&job_id);

        assert!(result.is_ok());
        assert_eq!(manager.job_queue.jobs[0].status, JobStatus::Running);
    }

    #[test]
    fn test_start_job_not_found() {
        let mut manager = JobManager::new();
        let result = manager.start_job("nonexistent");

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Job nonexistent not found");
    }

    #[test]
    fn test_pause_job() {
        let mut manager = JobManager::new();
        manager.create_job_from_generated_gcode("Test Job", JobType::GcodeFile, "G1 X10", None);

        let job_id = manager.job_queue.jobs[0].id.clone();
        manager.start_job(&job_id).unwrap();

        let result = manager.pause_job(&job_id);
        assert!(result.is_ok());
        assert_eq!(manager.job_queue.jobs[0].status, JobStatus::Paused);
    }

    #[test]
    fn test_cancel_job() {
        let mut manager = JobManager::new();
        manager.create_job_from_generated_gcode("Test Job", JobType::GcodeFile, "G1 X10", None);

        let job_id = manager.job_queue.jobs[0].id.clone();
        manager.start_job(&job_id).unwrap();

        let result = manager.cancel_job(&job_id);
        assert!(result.is_ok());
        assert_eq!(manager.job_queue.jobs[0].status, JobStatus::Cancelled);
    }

    #[test]
    fn test_complete_job() {
        let mut manager = JobManager::new();
        manager.create_job_from_generated_gcode("Test Job", JobType::GcodeFile, "G1 X10", None);

        let job_id = manager.job_queue.jobs[0].id.clone();
        manager.start_job(&job_id).unwrap();

        let result = manager.complete_job(&job_id);
        assert!(result.is_ok());
        assert_eq!(manager.job_queue.jobs[0].status, JobStatus::Completed);
    }

    #[test]
    fn test_fail_job() {
        let mut manager = JobManager::new();
        manager.create_job_from_generated_gcode("Test Job", JobType::GcodeFile, "G1 X10", None);

        let job_id = manager.job_queue.jobs[0].id.clone();
        manager.start_job(&job_id).unwrap();

        let result = manager.fail_job(&job_id, "Test error".to_string());
        assert!(result.is_ok());
        assert_eq!(manager.job_queue.jobs[0].status, JobStatus::Failed);
        assert_eq!(
            manager.job_queue.jobs[0].error_message,
            Some("Test error".to_string())
        );
    }

    #[test]
    fn test_update_job_progress() {
        let mut manager = JobManager::new();
        manager.create_job_from_generated_gcode("Test Job", JobType::GcodeFile, "G1 X10", None);

        let job_id = manager.job_queue.jobs[0].id.clone();
        manager.start_job(&job_id).unwrap();

        let result = manager.update_job_progress(&job_id, 0.75);
        assert!(result.is_ok());
        assert_eq!(manager.job_queue.jobs[0].progress, 0.75);
    }

    #[test]
    fn test_get_current_job() {
        let mut manager = JobManager::new();
        manager.create_job_from_generated_gcode("Test Job", JobType::GcodeFile, "G1 X10", None);

        let job_id = manager.job_queue.jobs[0].id.clone();
        manager.start_job(&job_id).unwrap();

        let current_job = manager.get_current_job();
        assert!(current_job.is_some());
        assert_eq!(current_job.unwrap().name, "Test Job");
    }

    #[test]
    fn test_get_current_job_id() {
        let mut manager = JobManager::new();
        manager.create_job_from_generated_gcode("Test Job", JobType::GcodeFile, "G1 X10", None);

        let job_id = manager.job_queue.jobs[0].id.clone();
        manager.start_job(&job_id).unwrap();

        let current_job_id = manager.get_current_job_id();
        assert!(current_job_id.is_some());
        assert_eq!(current_job_id.unwrap(), job_id);
    }

    #[test]
    fn test_get_current_job_no_active() {
        let manager = JobManager::new();

        let current_job = manager.get_current_job();
        assert!(current_job.is_none());

        let current_job_id = manager.get_current_job_id();
        assert!(current_job_id.is_none());
    }

    #[test]
    fn test_resume_job() {
        let mut manager = JobManager::new();
        manager.create_job_from_generated_gcode(
            "Test Job",
            JobType::GcodeFile,
            "G1 X10\nG1 X20",
            None,
        );

        let job_id = manager.job_queue.jobs[0].id.clone();
        manager.start_job(&job_id).unwrap();

        // Simulate pausing and setting resume line
        manager.pause_job(&job_id).unwrap();
        manager.job_queue.jobs[0].last_completed_line = Some(1);

        let result = manager.resume_job(&job_id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Should return resume line
        assert_eq!(manager.job_queue.jobs[0].status, JobStatus::Running);
    }

    #[test]
    fn test_resume_job_not_found() {
        let mut manager = JobManager::new();
        let result = manager.resume_job("nonexistent");

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Job not found");
    }

    #[test]
    fn test_resume_job_cannot_resume() {
        let mut manager = JobManager::new();
        manager.create_job_from_generated_gcode("Test Job", JobType::GcodeFile, "G1 X10", None);

        let job_id = manager.job_queue.jobs[0].id.clone();
        manager.start_job(&job_id).unwrap();

        let result = manager.resume_job(&job_id);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Job cannot be resumed");
    }

    #[test]
    fn test_save_jobs_to_file() {
        let mut manager = JobManager::new();

        // Add some test jobs
        manager.create_job_from_generated_gcode(
            "Test Job 1",
            JobType::GcodeFile,
            "G1 X10 Y20",
            Some("Oak"),
        );
        manager.create_job_from_generated_gcode(
            "Test Job 2",
            JobType::CAMOperation,
            "G1 X20 Y30",
            None,
        );

        // Start one job
        let job_id = manager.job_queue.jobs[0].id.clone();
        manager.start_job(&job_id).unwrap();

        // Save to temporary file
        let temp_path = std::env::temp_dir().join("test_manager_save.json");
        let result = manager.save_jobs_to_file(&temp_path);
        assert!(result.is_ok());

        // Verify file exists and has content
        assert!(temp_path.exists());
        let content = std::fs::read_to_string(&temp_path).unwrap();
        assert!(content.contains("Test Job 1"));
        assert!(content.contains("Test Job 2"));

        // Clean up
        std::fs::remove_file(&temp_path).ok();
    }

    #[test]
    fn test_load_jobs_from_file() {
        let manager = JobManager::new();

        // Create a test queue to save first
        let mut test_queue = JobQueue::new();
        let job1 = Job::new("Loaded Job 1".to_string(), JobType::GcodeFile)
            .with_material("Pine".to_string());
        let job2 = Job::new("Loaded Job 2".to_string(), JobType::CAMOperation);
        test_queue.add_job(job1);
        test_queue.add_job(job2);

        // Save test queue
        let temp_path = std::env::temp_dir().join("test_manager_load.json");
        test_queue.save_to_file(&temp_path).unwrap();

        // Load using manager method
        let loaded_queue = manager.load_jobs_from_file(&temp_path).unwrap();

        // Verify loaded queue
        assert_eq!(loaded_queue.jobs.len(), 2);
        assert_eq!(loaded_queue.jobs[0].name, "Loaded Job 1");
        assert_eq!(loaded_queue.jobs[1].name, "Loaded Job 2");
        assert_eq!(loaded_queue.jobs[0].material, Some("Pine".to_string()));
        assert_eq!(loaded_queue.jobs[1].material, None);

        // Active jobs should be cleared
        assert!(loaded_queue.active_jobs.is_empty());

        // Clean up
        std::fs::remove_file(&temp_path).ok();
    }

    #[test]
    fn test_replace_job_queue() {
        let mut manager = JobManager::new();

        // Add initial job
        manager.create_job_from_generated_gcode("Original Job", JobType::GcodeFile, "G1 X10", None);
        assert_eq!(manager.job_queue.jobs.len(), 1);
        assert_eq!(manager.job_queue.jobs[0].name, "Original Job");

        // Create replacement queue
        let mut new_queue = JobQueue::new();
        let job = Job::new("Replacement Job".to_string(), JobType::CAMOperation)
            .with_material("Maple".to_string());
        new_queue.add_job(job);

        // Replace queue
        manager.replace_job_queue(new_queue);

        // Verify replacement
        assert_eq!(manager.job_queue.jobs.len(), 1);
        assert_eq!(manager.job_queue.jobs[0].name, "Replacement Job");
        assert_eq!(manager.job_queue.jobs[0].job_type, JobType::CAMOperation);
        assert_eq!(
            manager.job_queue.jobs[0].material,
            Some("Maple".to_string())
        );
    }

    #[test]
    fn test_save_load_integration() {
        let mut manager = JobManager::new();

        // Create jobs with various states
        manager.create_job_from_generated_gcode(
            "Pending Job",
            JobType::GcodeFile,
            "G1 X10",
            Some("Oak"),
        );
        manager.create_job_from_generated_gcode(
            "Running Job",
            JobType::CAMOperation,
            "G1 Y20",
            None,
        );

        // Start the second job
        let running_job_id = manager.job_queue.jobs[1].id.clone();
        manager.start_job(&running_job_id).unwrap();
        manager.update_job_progress(&running_job_id, 0.5).unwrap();

        // Save jobs
        let temp_path = std::env::temp_dir().join("test_integration_save_load.json");
        manager.save_jobs_to_file(&temp_path).unwrap();

        // Load jobs into new manager
        let new_manager = JobManager::new();
        let loaded_queue = new_manager.load_jobs_from_file(&temp_path).unwrap();
        let mut new_manager = JobManager::new();
        new_manager.replace_job_queue(loaded_queue);

        // Verify loaded state
        assert_eq!(new_manager.job_queue.jobs.len(), 2);

        // First job should be pending
        assert_eq!(new_manager.job_queue.jobs[0].name, "Pending Job");
        assert_eq!(new_manager.job_queue.jobs[0].status, JobStatus::Pending);
        assert_eq!(
            new_manager.job_queue.jobs[0].material,
            Some("Oak".to_string())
        );

        // Second job should be reset to pending (running jobs become pending on load)
        assert_eq!(new_manager.job_queue.jobs[1].name, "Running Job");
        assert_eq!(new_manager.job_queue.jobs[1].status, JobStatus::Pending);
        assert_eq!(new_manager.job_queue.jobs[1].progress, 0.0); // Progress reset
        assert!(new_manager.job_queue.active_jobs.is_empty()); // No active jobs

        // Clean up
        std::fs::remove_file(&temp_path).ok();
    }
}
