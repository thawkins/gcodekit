use gcodekit::jobs::*;
use std::time::Duration;

#[cfg(test)]
mod job_history_tests {
    use super::*;

    #[test]
    fn test_job_history_creation() {
        let history = JobHistory::default();
        assert_eq!(history.max_history_size, 1000);
        assert!(history.completed_jobs.is_empty());
        assert_eq!(history.analytics.total_jobs, 0);
    }

    #[test]
    fn test_job_history_with_custom_size() {
        let history = JobHistory::new(500);
        assert_eq!(history.max_history_size, 500);
    }

    #[test]
    fn test_add_completed_job() {
        let mut history = JobHistory::default();
        let mut job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        job.start();
        job.complete();

        history.add_completed_job(job.clone());

        assert_eq!(history.completed_jobs.len(), 1);
        assert_eq!(history.analytics.total_jobs, 1);
        assert_eq!(history.analytics.completed_jobs, 1);
    }

    #[test]
    fn test_add_failed_job() {
        let mut history = JobHistory::default();
        let mut job = Job::new("Failed Job".to_string(), JobType::GcodeFile);
        job.start();
        job.fail("Test error".to_string());

        history.add_completed_job(job);

        assert_eq!(history.analytics.total_jobs, 1);
        assert_eq!(history.analytics.failed_jobs, 1);
        assert_eq!(history.analytics.completed_jobs, 0);
    }

    #[test]
    fn test_add_cancelled_job() {
        let mut history = JobHistory::default();
        let mut job = Job::new("Cancelled Job".to_string(), JobType::GcodeFile);
        job.start();
        job.cancel();

        history.add_completed_job(job);

        assert_eq!(history.analytics.total_jobs, 1);
        assert_eq!(history.analytics.cancelled_jobs, 1);
    }

    #[test]
    fn test_history_max_size_enforcement() {
        let mut history = JobHistory::new(5);

        // Add 10 jobs
        for i in 0..10 {
            let mut job = Job::new(format!("Job {}", i), JobType::GcodeFile);
            job.start();
            job.complete();
            history.add_completed_job(job);
        }

        // Should only keep last 5
        assert_eq!(history.completed_jobs.len(), 5);
        assert_eq!(history.completed_jobs[0].name, "Job 5");
        assert_eq!(history.completed_jobs[4].name, "Job 9");
    }

    #[test]
    fn test_get_recent_jobs() {
        let mut history = JobHistory::default();

        let mut job = Job::new("Recent Job".to_string(), JobType::GcodeFile);
        job.start();
        job.complete();
        history.add_completed_job(job);

        let recent = history.get_recent_jobs(7);
        assert_eq!(recent.len(), 1);
    }

    #[test]
    fn test_get_jobs_by_type() {
        let mut history = JobHistory::default();

        let mut job1 = Job::new("GCode Job".to_string(), JobType::GcodeFile);
        job1.complete();
        history.add_completed_job(job1);

        let mut job2 = Job::new("CAM Job".to_string(), JobType::CAMOperation);
        job2.complete();
        history.add_completed_job(job2);

        let gcode_jobs = history.get_jobs_by_type(&JobType::GcodeFile);
        assert_eq!(gcode_jobs.len(), 1);
        assert_eq!(gcode_jobs[0].name, "GCode Job");

        let cam_jobs = history.get_jobs_by_type(&JobType::CAMOperation);
        assert_eq!(cam_jobs.len(), 1);
        assert_eq!(cam_jobs[0].name, "CAM Job");
    }

    #[test]
    fn test_clear_history() {
        let mut history = JobHistory::default();

        let mut job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        job.complete();
        history.add_completed_job(job);

        assert_eq!(history.completed_jobs.len(), 1);

        history.clear_history();

        assert_eq!(history.completed_jobs.len(), 0);
        assert_eq!(history.analytics.total_jobs, 0);
    }

    #[test]
    fn test_export_import_json() {
        let mut history = JobHistory::default();
        let mut job = Job::new("Export Test".to_string(), JobType::GcodeFile);
        job.complete();
        history.add_completed_job(job);

        let json = history.export_to_json().expect("Failed to export");
        assert!(json.contains("Export Test"));

        let imported = JobHistory::import_from_json(&json).expect("Failed to import");
        assert_eq!(imported.completed_jobs.len(), 1);
        assert_eq!(imported.completed_jobs[0].name, "Export Test");
    }
}

#[cfg(test)]
mod job_analytics_tests {
    use super::*;

    #[test]
    fn test_job_analytics_default() {
        let analytics = JobAnalytics::default();
        assert_eq!(analytics.total_jobs, 0);
        assert_eq!(analytics.completed_jobs, 0);
        assert_eq!(analytics.failed_jobs, 0);
        assert_eq!(analytics.cancelled_jobs, 0);
        assert_eq!(analytics.average_efficiency, 0.0);
        assert_eq!(analytics.average_success_rate, 0.0);
    }

    #[test]
    fn test_analytics_track_job_types() {
        let mut history = JobHistory::default();

        let mut job1 = Job::new("Job 1".to_string(), JobType::GcodeFile);
        job1.complete();
        history.add_completed_job(job1);

        let mut job2 = Job::new("Job 2".to_string(), JobType::GcodeFile);
        job2.complete();
        history.add_completed_job(job2);

        let mut job3 = Job::new("Job 3".to_string(), JobType::CAMOperation);
        job3.complete();
        history.add_completed_job(job3);

        assert_eq!(*history.analytics.jobs_by_type.get(&JobType::GcodeFile).unwrap(), 2);
        assert_eq!(*history.analytics.jobs_by_type.get(&JobType::CAMOperation).unwrap(), 1);
    }

    #[test]
    fn test_analytics_track_materials() {
        let mut history = JobHistory::default();

        let mut job1 = Job::new("Job 1".to_string(), JobType::GcodeFile)
            .with_material("Aluminum".to_string());
        job1.complete();
        history.add_completed_job(job1);

        let mut job2 = Job::new("Job 2".to_string(), JobType::GcodeFile)
            .with_material("Wood".to_string());
        job2.complete();
        history.add_completed_job(job2);

        // Most recently used material should be tracked
        assert!(history.analytics.most_used_material.is_some());
    }

    #[test]
    fn test_analytics_track_tools() {
        let mut history = JobHistory::default();

        let mut job = Job::new("Job".to_string(), JobType::GcodeFile)
            .with_tool("End Mill 3mm".to_string());
        job.complete();
        history.add_completed_job(job);

        assert!(history.analytics.most_used_tool.is_some());
        assert_eq!(history.analytics.most_used_tool.as_ref().unwrap(), "End Mill 3mm");
    }
}

#[cfg(test)]
mod job_performance_tests {
    use super::*;

    #[test]
    fn test_job_efficiency() {
        let mut job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        job.machine_time = Some(Duration::from_secs(3600)); // 1 hour
        job.actual_duration = Some(Duration::from_secs(4000)); // 1h 6m 40s

        let efficiency = job.efficiency();
        assert!(efficiency > 0.89 && efficiency < 0.91); // ~90%
    }

    #[test]
    fn test_job_efficiency_no_data() {
        let job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        let efficiency = job.efficiency();
        assert_eq!(efficiency, 0.0);
    }

    #[test]
    fn test_job_success_rate_completed() {
        let mut job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        job.complete();

        let success_rate = job.success_rate();
        assert_eq!(success_rate, 1.0);
    }

    #[test]
    fn test_job_success_rate_with_retries() {
        let mut job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        job.retry_count = 2;
        job.complete();

        let success_rate = job.success_rate();
        assert!(success_rate < 1.0); // Lower due to retries
        assert!(success_rate > 0.0);
    }

    #[test]
    fn test_job_success_rate_failed() {
        let mut job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        job.fail("Test error".to_string());

        let success_rate = job.success_rate();
        assert_eq!(success_rate, 0.0);
    }

    #[test]
    fn test_update_performance_data() {
        let mut job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        job.gcode_content = "G1 X10\nG1 Y20\nG1 Z5".to_string();

        job.update_performance_data(2, 1000);

        assert_eq!(job.lines_processed, 2);
        assert_eq!(job.bytes_processed, 1000);
        assert_eq!(job.total_lines, 3);
    }

    #[test]
    fn test_record_retry() {
        let mut job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        assert_eq!(job.retry_count, 0);

        job.record_retry();
        assert_eq!(job.retry_count, 1);

        job.record_retry();
        assert_eq!(job.retry_count, 2);
    }

    #[test]
    fn test_record_pause() {
        let mut job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        assert_eq!(job.pause_count, 0);

        job.record_pause();
        assert_eq!(job.pause_count, 1);
    }

    #[test]
    fn test_update_feed_rate() {
        let mut job = Job::new("Test Job".to_string(), JobType::GcodeFile);

        job.update_feed_rate(100.0);
        assert_eq!(job.feed_rate_avg, Some(100.0));

        job.update_feed_rate(200.0);
        assert_eq!(job.feed_rate_avg, Some(150.0)); // Average
    }

    #[test]
    fn test_update_spindle_speed() {
        let mut job = Job::new("Test Job".to_string(), JobType::GcodeFile);

        job.update_spindle_speed(10000.0);
        assert_eq!(job.spindle_speed_avg, Some(10000.0));

        job.update_spindle_speed(12000.0);
        assert_eq!(job.spindle_speed_avg, Some(11000.0)); // Average
    }
}

#[cfg(test)]
mod job_queue_advanced_tests {
    use super::*;

    #[test]
    fn test_job_queue_multiple_active_jobs() {
        let mut queue = JobQueue::new();
        queue.max_concurrent_jobs = 2;

        let job1 = Job::new("Job 1".to_string(), JobType::GcodeFile);
        let job2 = Job::new("Job 2".to_string(), JobType::GcodeFile);
        let job3 = Job::new("Job 3".to_string(), JobType::GcodeFile);

        queue.add_job(job1);
        queue.add_job(job2);
        queue.add_job(job3);

        let id1 = queue.jobs[0].id.clone();
        let id2 = queue.jobs[1].id.clone();
        let id3 = queue.jobs[2].id.clone();

        // Start first two jobs
        assert!(queue.start_job(&id1).is_ok());
        assert!(queue.start_job(&id2).is_ok());

        // Third should fail - max concurrent reached
        assert!(queue.start_job(&id3).is_err());

        assert_eq!(queue.get_active_jobs().len(), 2);
    }

    #[test]
    fn test_reorder_jobs() {
        let mut queue = JobQueue::new();

        let job1 = Job::new("Job 1".to_string(), JobType::GcodeFile);
        let job2 = Job::new("Job 2".to_string(), JobType::GcodeFile);
        let job3 = Job::new("Job 3".to_string(), JobType::GcodeFile);

        queue.add_job(job1);
        queue.add_job(job2);
        queue.add_job(job3);

        let id1 = queue.jobs[0].id.clone();
        let id2 = queue.jobs[1].id.clone();
        let id3 = queue.jobs[2].id.clone();

        // Reorder: 3, 1, 2
        let new_order = vec![id3.clone(), id1.clone(), id2.clone()];
        let result = queue.reorder_jobs(new_order);
        assert!(result.is_ok());

        assert_eq!(queue.jobs[0].id, id3);
        assert_eq!(queue.jobs[1].id, id1);
        assert_eq!(queue.jobs[2].id, id2);
    }

    #[test]
    fn test_reorder_jobs_invalid() {
        let mut queue = JobQueue::new();

        let job1 = Job::new("Job 1".to_string(), JobType::GcodeFile);
        queue.add_job(job1);

        // Try to reorder with wrong number of IDs
        let result = queue.reorder_jobs(vec!["invalid-id".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_clear_completed_jobs() {
        let mut queue = JobQueue::new();

        let mut job1 = Job::new("Job 1".to_string(), JobType::GcodeFile);
        job1.complete();
        queue.add_job(job1);

        let mut job2 = Job::new("Job 2".to_string(), JobType::GcodeFile);
        job2.fail("Error".to_string());
        queue.add_job(job2);

        let job3 = Job::new("Job 3".to_string(), JobType::GcodeFile);
        queue.add_job(job3);

        assert_eq!(queue.jobs.len(), 3);

        queue.clear_completed_jobs();

        // Only pending job should remain
        assert_eq!(queue.jobs.len(), 1);
        assert_eq!(queue.jobs[0].name, "Job 3");
    }

    #[test]
    fn test_get_pending_jobs() {
        let mut queue = JobQueue::new();

        let job1 = Job::new("Pending 1".to_string(), JobType::GcodeFile);
        queue.add_job(job1);

        let mut job2 = Job::new("Running".to_string(), JobType::GcodeFile);
        job2.start();
        queue.add_job(job2);

        let job3 = Job::new("Pending 2".to_string(), JobType::GcodeFile);
        queue.add_job(job3);

        let pending = queue.get_pending_jobs();
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].name, "Pending 1");
        assert_eq!(pending[1].name, "Pending 2");
    }

    #[test]
    fn test_get_completed_jobs() {
        let mut queue = JobQueue::new();

        let mut job1 = Job::new("Completed 1".to_string(), JobType::GcodeFile);
        job1.complete();
        queue.add_job(job1);

        let job2 = Job::new("Pending".to_string(), JobType::GcodeFile);
        queue.add_job(job2);

        let mut job3 = Job::new("Completed 2".to_string(), JobType::GcodeFile);
        job3.complete();
        queue.add_job(job3);

        let completed = queue.get_completed_jobs();
        assert_eq!(completed.len(), 2);
    }

    #[test]
    fn test_get_failed_jobs() {
        let mut queue = JobQueue::new();

        let mut job1 = Job::new("Failed 1".to_string(), JobType::GcodeFile);
        job1.fail("Error 1".to_string());
        queue.add_job(job1);

        let job2 = Job::new("Pending".to_string(), JobType::GcodeFile);
        queue.add_job(job2);

        let mut job3 = Job::new("Failed 2".to_string(), JobType::GcodeFile);
        job3.fail("Error 2".to_string());
        queue.add_job(job3);

        let failed = queue.get_failed_jobs();
        assert_eq!(failed.len(), 2);
    }
}
