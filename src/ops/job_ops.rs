use crate::GcodeKitApp;

impl GcodeKitApp {
    /// Creates a new job from the currently generated G-code content.
    /// Adds the job to the job queue with optional material assignment.
    ///
    /// # Arguments
    /// * `name` - The name for the new job
    /// * `job_type` - The type of job being created
    pub fn create_job_from_generated_gcode(&mut self, name: &str, job_type: crate::jobs::JobType) {
        if !self.gcode.gcode_content.is_empty() {
            let mut job = crate::jobs::Job::new(name.to_string(), job_type);
            if let Some(material) = &self.ui.selected_material {
                job = job.with_material(material.clone());
            }
            // For generated G-code, we don't have a file path, so we'll store it as content
            // The job system would need to be extended to handle in-memory G-code
            self.job.job_queue.add_job(job);
        }
    }

    /// Starts execution of a queued job.
    /// Changes the job status to running and sets it as the current active job.
    ///
    /// # Arguments
    /// * `job_id` - The ID of the job to start
    ///
    /// # Returns
    /// `Ok(())` on success, `Err(String)` with error message on failure
    pub fn start_job(&mut self, job_id: &str) -> Result<(), String> {
        self.job.job_queue.start_job(job_id)?;
        self.job.current_job_id = Some(job_id.to_string());
        // TODO: log_console
        // self.log_console(&format!("Started job: {}", job_id));
        Ok(())
    }

    /// Resumes execution of a paused or interrupted job from its last completed line.
    /// Only works for jobs that support resumption.
    ///
    /// # Arguments
    /// * `job_id` - The ID of the job to resume
    ///
    /// # Returns
    /// `Ok(())` on success, `Err(String)` with error message on failure
    pub fn resume_job(&mut self, job_id: &str) -> Result<(), String> {
        // Get the resume line
        let resume_line = {
            let job = self.job.job_queue.get_job(job_id).ok_or("Job not found")?;
            job.get_resume_line().ok_or("Job cannot be resumed")?
        };

        // Resume the job
        self.job.job_queue.resume_job(job_id)?;
        self.job.current_job_id = Some(job_id.to_string());

        // Start sending from resume line
        self.send_gcode_from_line(resume_line);
        // TODO: log_console
        // self.log_console(&format!(
        //     "Resumed job {} from line {}",
        //     job_id,
        //     resume_line + 1
        // ));
        Ok(())
    }
}
