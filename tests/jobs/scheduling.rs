use gcodekit::jobs::*;
use chrono::{Duration, Utc};

#[cfg(test)]
mod scheduled_job_tests {
    use super::*;

    #[test]
    fn test_scheduled_job_creation() {
        let job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        let start_time = Utc::now() + Duration::hours(1);
        let scheduled = ScheduledJob::new(job.clone(), start_time);

        assert_eq!(scheduled.job.name, "Test Job");
        assert_eq!(scheduled.start_time, start_time);
        assert_eq!(scheduled.repeat_interval, RepeatInterval::None);
        assert!(scheduled.enabled);
        assert_eq!(scheduled.run_count, 0);
        assert!(scheduled.dependencies.is_empty());
    }

    #[test]
    fn test_scheduled_job_with_repeat() {
        let job = Job::new("Recurring Job".to_string(), JobType::Maintenance);
        let start_time = Utc::now();
        let scheduled = ScheduledJob::new(job, start_time)
            .with_repeat_interval(RepeatInterval::Hours(2));

        assert_eq!(scheduled.repeat_interval, RepeatInterval::Hours(2));
    }

    #[test]
    fn test_scheduled_job_with_dependencies() {
        let job = Job::new("Dependent Job".to_string(), JobType::GcodeFile);
        let start_time = Utc::now();

        let dep = JobDependency {
            job_id: "job-123".to_string(),
            required_status: JobStatus::Completed,
        };

        let scheduled = ScheduledJob::new(job, start_time)
            .with_dependencies(vec![dep]);

        assert_eq!(scheduled.dependencies.len(), 1);
        assert_eq!(scheduled.dependencies[0].job_id, "job-123");
    }

    #[test]
    fn test_scheduled_job_with_max_runs() {
        let job = Job::new("Limited Job".to_string(), JobType::GcodeFile);
        let start_time = Utc::now();
        let scheduled = ScheduledJob::new(job, start_time)
            .with_max_runs(5);

        assert_eq!(scheduled.max_runs, Some(5));
    }

    #[test]
    fn test_should_run_enabled() {
        let job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        let start_time = Utc::now() - Duration::minutes(1); // Past time
        let scheduled = ScheduledJob::new(job, start_time);

        assert!(scheduled.should_run(Utc::now()));
    }

    #[test]
    fn test_should_run_disabled() {
        let job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        let start_time = Utc::now() - Duration::minutes(1);
        let scheduled = ScheduledJob::new(job, start_time).disable();

        assert!(!scheduled.should_run(Utc::now()));
    }

    #[test]
    fn test_should_run_future() {
        let job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        let start_time = Utc::now() + Duration::hours(1); // Future time
        let scheduled = ScheduledJob::new(job, start_time);

        assert!(!scheduled.should_run(Utc::now()));
    }

    #[test]
    fn test_should_run_max_runs_reached() {
        let job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        let start_time = Utc::now() - Duration::minutes(1);
        let mut scheduled = ScheduledJob::new(job, start_time)
            .with_max_runs(2);

        scheduled.run_count = 2;

        assert!(!scheduled.should_run(Utc::now()));
    }

    #[test]
    fn test_dependencies_satisfied() {
        let job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        let start_time = Utc::now();

        let dep1 = JobDependency {
            job_id: "job-1".to_string(),
            required_status: JobStatus::Completed,
        };
        let dep2 = JobDependency {
            job_id: "job-2".to_string(),
            required_status: JobStatus::Completed,
        };

        let scheduled = ScheduledJob::new(job, start_time)
            .with_dependencies(vec![dep1, dep2]);

        // All dependencies satisfied
        let completed_jobs = vec!["job-1".to_string(), "job-2".to_string()];
        assert!(scheduled.dependencies_satisfied(&completed_jobs));

        // Only one dependency satisfied
        let completed_jobs = vec!["job-1".to_string()];
        assert!(!scheduled.dependencies_satisfied(&completed_jobs));

        // No dependencies satisfied
        let completed_jobs = vec![];
        assert!(!scheduled.dependencies_satisfied(&completed_jobs));
    }

    #[test]
    fn test_mark_executed() {
        let job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        let start_time = Utc::now();
        let mut scheduled = ScheduledJob::new(job, start_time)
            .with_repeat_interval(RepeatInterval::Hours(1));

        let execution_time = Utc::now();
        scheduled.mark_executed(execution_time);

        assert_eq!(scheduled.run_count, 1);
        assert!(scheduled.last_run.is_some());
        assert_eq!(scheduled.last_run.unwrap(), execution_time);
    }

    #[test]
    fn test_time_until_next_run() {
        let job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        let start_time = Utc::now() + Duration::hours(2);
        let scheduled = ScheduledJob::new(job, start_time);

        let time_until = scheduled.time_until_next_run(Utc::now());
        assert!(time_until.is_some());
        let duration = time_until.unwrap();
        assert!(duration.as_secs() > 7000); // ~2 hours
    }

    #[test]
    fn test_enable_disable() {
        let job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        let start_time = Utc::now();
        let scheduled = ScheduledJob::new(job, start_time);

        assert!(scheduled.enabled);

        let scheduled = scheduled.disable();
        assert!(!scheduled.enabled);

        let scheduled = scheduled.enable();
        assert!(scheduled.enabled);
    }
}

#[cfg(test)]
mod repeat_interval_tests {
    use super::*;

    #[test]
    fn test_repeat_interval_variants() {
        let intervals = vec![
            RepeatInterval::None,
            RepeatInterval::Minutes(30),
            RepeatInterval::Hours(2),
            RepeatInterval::Days(7),
            RepeatInterval::Weeks(2),
            RepeatInterval::Months(1),
        ];

        for interval in intervals {
            let cloned = interval.clone();
            assert_eq!(interval, cloned);
        }
    }

    #[test]
    fn test_repeat_interval_equality() {
        assert_eq!(RepeatInterval::None, RepeatInterval::None);
        assert_eq!(RepeatInterval::Hours(1), RepeatInterval::Hours(1));
        assert_ne!(RepeatInterval::Hours(1), RepeatInterval::Hours(2));
        assert_ne!(RepeatInterval::Hours(24), RepeatInterval::Days(1));
    }
}

#[cfg(test)]
mod job_scheduler_tests {
    use super::*;

    #[test]
    fn test_job_scheduler_creation() {
        let scheduler = JobScheduler::new();
        assert!(scheduler.scheduled_jobs.is_empty());
        assert_eq!(scheduler.check_interval_seconds, 60);
    }

    #[test]
    fn test_job_scheduler_with_check_interval() {
        let scheduler = JobScheduler::new().with_check_interval(30);
        assert_eq!(scheduler.check_interval_seconds, 30);
    }

    #[test]
    fn test_add_scheduled_job() {
        let mut scheduler = JobScheduler::new();
        let job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        let start_time = Utc::now() + Duration::hours(1);
        let scheduled = ScheduledJob::new(job, start_time);

        scheduler.add_scheduled_job(scheduled);
        assert_eq!(scheduler.scheduled_jobs.len(), 1);
    }

    #[test]
    fn test_remove_scheduled_job() {
        let mut scheduler = JobScheduler::new();
        let job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        let start_time = Utc::now();
        let scheduled = ScheduledJob::new(job, start_time);
        let schedule_id = scheduled.schedule_id.clone();

        scheduler.add_scheduled_job(scheduled);
        assert_eq!(scheduler.scheduled_jobs.len(), 1);

        let removed = scheduler.remove_scheduled_job(&schedule_id);
        assert!(removed.is_some());
        assert_eq!(scheduler.scheduled_jobs.len(), 0);
    }

    #[test]
    fn test_get_scheduled_job() {
        let mut scheduler = JobScheduler::new();
        let job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        let start_time = Utc::now();
        let scheduled = ScheduledJob::new(job, start_time);
        let schedule_id = scheduled.schedule_id.clone();

        scheduler.add_scheduled_job(scheduled);

        let retrieved = scheduler.get_scheduled_job(&schedule_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().schedule_id, schedule_id);
    }

    #[test]
    fn test_enable_disable_schedule() {
        let mut scheduler = JobScheduler::new();
        let job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        let start_time = Utc::now();
        let scheduled = ScheduledJob::new(job, start_time);
        let schedule_id = scheduled.schedule_id.clone();

        scheduler.add_scheduled_job(scheduled);

        // Disable
        let result = scheduler.disable_schedule(&schedule_id);
        assert!(result.is_ok());
        let scheduled = scheduler.get_scheduled_job(&schedule_id).unwrap();
        assert!(!scheduled.enabled);

        // Enable
        let result = scheduler.enable_schedule(&schedule_id);
        assert!(result.is_ok());
        let scheduled = scheduler.get_scheduled_job(&schedule_id).unwrap();
        assert!(scheduled.enabled);
    }

    #[test]
    fn test_get_enabled_schedules() {
        let mut scheduler = JobScheduler::new();

        let job1 = Job::new("Job 1".to_string(), JobType::GcodeFile);
        let scheduled1 = ScheduledJob::new(job1, Utc::now());
        scheduler.add_scheduled_job(scheduled1);

        let job2 = Job::new("Job 2".to_string(), JobType::GcodeFile);
        let scheduled2 = ScheduledJob::new(job2, Utc::now()).disable();
        scheduler.add_scheduled_job(scheduled2);

        let enabled = scheduler.get_enabled_schedules();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].job.name, "Job 1");
    }

    #[test]
    fn test_get_jobs_to_run() {
        let mut scheduler = JobScheduler::new();

        // Job that should run (past time)
        let job1 = Job::new("Job 1".to_string(), JobType::GcodeFile);
        let scheduled1 = ScheduledJob::new(job1, Utc::now() - Duration::minutes(5));
        scheduler.add_scheduled_job(scheduled1);

        // Job that shouldn't run (future time)
        let job2 = Job::new("Job 2".to_string(), JobType::GcodeFile);
        let scheduled2 = ScheduledJob::new(job2, Utc::now() + Duration::hours(1));
        scheduler.add_scheduled_job(scheduled2);

        let jobs_to_run = scheduler.get_jobs_to_run(Utc::now(), &[]);
        assert_eq!(jobs_to_run.len(), 1);
        assert_eq!(jobs_to_run[0].job.name, "Job 1");
    }

    #[test]
    fn test_get_jobs_to_run_with_dependencies() {
        let mut scheduler = JobScheduler::new();

        let dep = JobDependency {
            job_id: "dependency-job".to_string(),
            required_status: JobStatus::Completed,
        };

        let job = Job::new("Dependent Job".to_string(), JobType::GcodeFile);
        let scheduled = ScheduledJob::new(job, Utc::now() - Duration::minutes(5))
            .with_dependencies(vec![dep]);
        scheduler.add_scheduled_job(scheduled);

        // Without dependency satisfied
        let jobs_to_run = scheduler.get_jobs_to_run(Utc::now(), &[]);
        assert_eq!(jobs_to_run.len(), 0);

        // With dependency satisfied
        let completed_jobs = vec!["dependency-job".to_string()];
        let jobs_to_run = scheduler.get_jobs_to_run(Utc::now(), &completed_jobs);
        assert_eq!(jobs_to_run.len(), 1);
    }

    #[test]
    fn test_mark_job_executed() {
        let mut scheduler = JobScheduler::new();
        let job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        let start_time = Utc::now();
        let scheduled = ScheduledJob::new(job, start_time);
        let schedule_id = scheduled.schedule_id.clone();

        scheduler.add_scheduled_job(scheduled);

        let execution_time = Utc::now();
        let result = scheduler.mark_job_executed(&schedule_id, execution_time);
        assert!(result.is_ok());

        let scheduled = scheduler.get_scheduled_job(&schedule_id).unwrap();
        assert_eq!(scheduled.run_count, 1);
        assert!(scheduled.last_run.is_some());
    }

    #[test]
    fn test_get_next_run_time() {
        let mut scheduler = JobScheduler::new();

        let job1 = Job::new("Job 1".to_string(), JobType::GcodeFile);
        let time1 = Utc::now() + Duration::hours(2);
        let scheduled1 = ScheduledJob::new(job1, time1);
        scheduler.add_scheduled_job(scheduled1);

        let job2 = Job::new("Job 2".to_string(), JobType::GcodeFile);
        let time2 = Utc::now() + Duration::hours(1); // Earlier
        let scheduled2 = ScheduledJob::new(job2, time2);
        scheduler.add_scheduled_job(scheduled2);

        let next_run = scheduler.get_next_run_time();
        assert!(next_run.is_some());
        // Should be the earlier time (time2)
        assert!((next_run.unwrap() - time2).num_seconds().abs() < 2);
    }

    #[test]
    fn test_get_upcoming_jobs() {
        let mut scheduler = JobScheduler::new();

        let job1 = Job::new("Job 1".to_string(), JobType::GcodeFile);
        let scheduled1 = ScheduledJob::new(job1, Utc::now() + Duration::minutes(30));
        scheduler.add_scheduled_job(scheduled1);

        let job2 = Job::new("Job 2".to_string(), JobType::GcodeFile);
        let scheduled2 = ScheduledJob::new(job2, Utc::now() + Duration::hours(2));
        scheduler.add_scheduled_job(scheduled2);

        // Get jobs within next hour
        let upcoming = scheduler.get_upcoming_jobs(std::time::Duration::from_secs(3600));
        assert_eq!(upcoming.len(), 1);
        assert_eq!(upcoming[0].job.name, "Job 1");
    }

    #[test]
    fn test_scheduler_save_load() {
        let mut scheduler = JobScheduler::new();

        let job = Job::new("Test Job".to_string(), JobType::GcodeFile);
        let scheduled = ScheduledJob::new(job, Utc::now());
        scheduler.add_scheduled_job(scheduled);

        let temp_path = std::env::temp_dir().join("test_scheduler.json");
        scheduler.save_to_file(&temp_path).expect("Failed to save scheduler");

        let loaded = JobScheduler::load_from_file(&temp_path).expect("Failed to load scheduler");
        assert_eq!(loaded.scheduled_jobs.len(), 1);
        assert_eq!(loaded.scheduled_jobs[0].job.name, "Test Job");

        std::fs::remove_file(&temp_path).ok();
    }
}

#[cfg(test)]
mod job_dependency_tests {
    use super::*;

    #[test]
    fn test_job_dependency_creation() {
        let dep = JobDependency {
            job_id: "job-123".to_string(),
            required_status: JobStatus::Completed,
        };

        assert_eq!(dep.job_id, "job-123");
        assert_eq!(dep.required_status, JobStatus::Completed);
    }

    #[test]
    fn test_job_dependency_clone() {
        let dep1 = JobDependency {
            job_id: "job-456".to_string(),
            required_status: JobStatus::Completed,
        };

        let dep2 = dep1.clone();
        assert_eq!(dep1.job_id, dep2.job_id);
        assert_eq!(dep1.required_status, dep2.required_status);
    }
}
