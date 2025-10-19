mod scheduling;

mod tests {
    use gcodekit::jobs::*;
    use gcodekit::jobs::manager;
    use std::env;
    use std::fs;

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
        let job = job_manager.job_queue.get_job(&job_id).unwrap();
        assert_eq!(job.status, JobStatus::Paused);
        assert_eq!(job.last_completed_line, Some(2));
        assert!(job.can_resume_job());

        // Test resume functionality
        let resume_line = job_manager.resume_job(&job_id).unwrap();
        assert_eq!(resume_line, 2);

        // Verify job is running again
        let job = job_manager.job_queue.get_job(&job_id).unwrap();
        assert_eq!(job.status, JobStatus::Running);
        assert_eq!(job.last_completed_line, Some(2)); // Should still have the resume point
    }

    #[test]
    fn test_job_resumption_with_invalid_job() {
        let mut job_manager = manager::JobManager::default();

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
        queue.start_job(&job_id).unwrap();

        // Save to temporary file
        let temp_path = env::temp_dir().join("test_job_queue.json");
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
        fs::remove_file(&temp_path).ok();
    }

    #[test]
    fn test_job_queue_save_load_empty() {
        let queue = JobQueue::new();

        // Save empty queue
        let temp_path = env::temp_dir().join("test_empty_job_queue.json");
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
        fs::remove_file(&temp_path).ok();
    }
}
