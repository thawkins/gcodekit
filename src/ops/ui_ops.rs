use crate::{
    input::{handle_keyboard_shortcuts, Action},
    materials, GcodeKitApp,
};

impl GcodeKitApp {
    /// Processes keyboard shortcuts and executes corresponding actions.
    /// Handles file operations, machine control, jogging, and other shortcuts
    /// based on the configured key bindings.
    ///
    /// # Arguments
    /// * `ctx` - The egui context for input handling
    pub fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        // Clone keybindings to avoid borrowing issues
        let keybindings = self.keybindings.clone();

        let action_handler = |action: &Action| match action {
            Action::OpenFile => {
                self.load_gcode_file();
            }
            Action::SaveFile => {
                self.save_gcode_file();
            }
            Action::ExportGcode => {
                self.export_design_to_gcode();
            }
            Action::ImportVector => {
                self.import_vector_file();
            }
            Action::Undo => {
                self.designer.undo();
            }
            Action::Redo => {
                self.designer.redo();
            }
            Action::ZoomIn => {
                // TODO: Implement zoom
            }
            Action::ZoomOut => {
                // TODO: Implement zoom
            }
            Action::Home => {
                self.send_gcode("G28");
            }
            Action::JogXPlus => {
                self.send_gcode("G91 G0 X10 F1000");
            }
            Action::JogXMinus => {
                self.send_gcode("G91 G0 X-10 F1000");
            }
            Action::JogYPlus => {
                self.send_gcode("G91 G0 Y10 F1000");
            }
            Action::JogYMinus => {
                self.send_gcode("G91 G0 Y-10 F1000");
            }
            Action::JogZPlus => {
                self.send_gcode("G91 G0 Z10 F1000");
            }
            Action::JogZMinus => {
                self.send_gcode("G91 G0 Z-10 F1000");
            }
            Action::ProbeZ => {
                self.send_gcode("G38.2 Z-10 F50");
            }
            Action::FeedHold => {
                self.send_gcode("!");
            }
            Action::Resume => {
                self.send_gcode("~");
            }
            Action::Reset => {
                self.send_gcode("\x18");
            }
        };

        handle_keyboard_shortcuts(ctx, &keybindings, action_handler);
    }

    /// Resets all fields in the add material dialog to their default values.
    /// Clears name, type, properties, and notes fields.
    pub fn reset_add_material_dialog(&mut self) {
        self.ui.new_material_name.clear();
        self.ui.new_material_type = materials::MaterialType::Wood;
        self.ui.new_material_density = 0.0;
        self.ui.new_material_hardness = 0.0;
        self.ui.new_material_cutting_speed = 0.0;
        self.ui.new_material_feed_rate = 0.0;
        self.ui.new_material_spindle_speed = 0.0;
        self.ui.new_material_tool_material.clear();
        self.ui.new_material_tool_coating.clear();
        self.ui.new_material_chip_load_min = 0.0;
        self.ui.new_material_chip_load_max = 0.0;
        self.ui.new_material_notes.clear();
    }

    /// Renders the job manager tab UI showing job queue, controls, and status.
    /// Provides interface for creating, starting, pausing, and managing jobs.
    ///
    /// # Arguments
    /// * `ui` - The egui UI context for rendering
    pub fn show_job_manager_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Job Manager");
            ui.separator();

            // Job queue controls
            ui.horizontal(|ui| {
                if ui.button("➕ Add Job").clicked() {
                    self.ui.show_job_creation_dialog = true;
                    self.ui.new_job_name = "New Job".to_string();
                    self.ui.new_job_type = crate::jobs::JobType::GcodeFile;
                    self.ui.selected_material = None;
                }
                if ui.button("🗑️ Clear Completed").clicked() {
                    self.job.job_queue.clear_completed_jobs();
                }
                ui.label(format!("Jobs: {}", self.job.job_queue.jobs.len()));
            });

            // Job creation dialog
            if self.ui.show_job_creation_dialog {
                egui::Window::new("Create New Job")
                    .collapsible(false)
                    .resizable(false)
                    .show(ui.ctx(), |ui| {
                        ui.vertical(|ui| {
                            ui.label("Job Name:");
                            ui.text_edit_singleline(&mut self.ui.new_job_name);

                            ui.label("Job Type:");
                            egui::ComboBox::from_id_salt("job_type_combobox")
                                .selected_text(format!("{:?}", self.ui.new_job_type))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.ui.new_job_type,
                                        crate::jobs::JobType::GcodeFile,
                                        "G-code File",
                                    );
                                    ui.selectable_value(
                                        &mut self.ui.new_job_type,
                                        crate::jobs::JobType::CAMOperation,
                                        "CAM Operation",
                                    );
                                    ui.selectable_value(
                                        &mut self.ui.new_job_type,
                                        crate::jobs::JobType::Probing,
                                        "Probing",
                                    );
                                    ui.selectable_value(
                                        &mut self.ui.new_job_type,
                                        crate::jobs::JobType::Calibration,
                                        "Calibration",
                                    );
                                    ui.selectable_value(
                                        &mut self.ui.new_job_type,
                                        crate::jobs::JobType::Maintenance,
                                        "Maintenance",
                                    );
                                });

                            ui.label("Material:");
                            let mut material_names: Vec<String> = self
                                .material_database
                                .get_all_materials()
                                .iter()
                                .map(|m| m.name.clone())
                                .collect();
                            material_names.insert(0, "None".to_string());

                            let current_selection = self
                                .ui
                                .selected_material
                                .as_ref()
                                .unwrap_or(&"None".to_string())
                                .clone();

                            egui::ComboBox::from_id_salt("material_combobox")
                                .selected_text(&current_selection)
                                .show_ui(ui, |ui| {
                                    for material_name in &material_names {
                                        let is_selected = Some(material_name.clone())
                                            == self.ui.selected_material
                                            || (material_name == "None"
                                                && self.ui.selected_material.is_none());
                                        if ui.selectable_label(is_selected, material_name).clicked()
                                        {
                                            if material_name == "None" {
                                                self.ui.selected_material = None;
                                            } else {
                                                self.ui.selected_material =
                                                    Some(material_name.clone());
                                            }
                                        }
                                    }
                                });

                            ui.separator();
                            ui.horizontal(|ui| {
                                if ui.button("Create").clicked() {
                                    let job_name = self.ui.new_job_name.clone();
                                    let job_type = self.ui.new_job_type.clone();
                                    let selected_material = self.ui.selected_material.clone();
                                    let mut job = crate::jobs::Job::new(job_name, job_type);
                                    if let Some(material) = &selected_material {
                                        job = job.with_material(material.clone());
                                    }
                                    self.job.job_queue.add_job(job);
                                    self.ui.show_job_creation_dialog = false;
                                }
                                if ui.button("Cancel").clicked() {
                                    self.ui.show_job_creation_dialog = false;
                                }
                            });
                        });
                    });
            }

            ui.separator();

            // Job list
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut jobs_to_start = Vec::new();
                let mut jobs_to_pause = Vec::new();
                let mut jobs_to_resume = Vec::new();
                let mut jobs_to_resume_interrupted = Vec::new(); // For resuming interrupted jobs
                let mut jobs_to_cancel = Vec::new();
                let mut jobs_to_remove = Vec::new();

                // Clone job data for display to avoid borrow issues
                let jobs_data: Vec<_> = self.job.job_queue.jobs.iter().cloned().collect();

                for job in &jobs_data {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            // Job status indicator
                            let status_color = match job.status {
                                crate::jobs::JobStatus::Pending => egui::Color32::GRAY,
                                crate::jobs::JobStatus::Running => egui::Color32::GREEN,
                                crate::jobs::JobStatus::Paused => egui::Color32::YELLOW,
                                crate::jobs::JobStatus::Completed => egui::Color32::BLUE,
                                crate::jobs::JobStatus::Failed => egui::Color32::RED,
                                crate::jobs::JobStatus::Cancelled => egui::Color32::ORANGE,
                            };
                            ui.colored_label(status_color, "●");

                            ui.label(&job.name);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(format!("{:.1}%", job.progress * 100.0));

                                    // Control buttons - collect actions instead of executing immediately
                                    match job.status {
                                        crate::jobs::JobStatus::Pending => {
                                            if ui.button(format!("▶️ Start##{}", job.id)).clicked()
                                            {
                                                jobs_to_start.push(job.id.clone());
                                            }
                                        }
                                        crate::jobs::JobStatus::Running => {
                                            if ui.button(format!("⏸️ Pause##{}", job.id)).clicked()
                                            {
                                                jobs_to_pause.push(job.id.clone());
                                            }
                                            if ui.button(format!("⏹️ Stop##{}", job.id)).clicked()
                                            {
                                                jobs_to_cancel.push(job.id.clone());
                                            }
                                        }
                                        crate::jobs::JobStatus::Paused => {
                                            if job.can_resume_job() {
                                                if ui
                                                    .button(format!("🔄 Resume##{}", job.id))
                                                    .clicked()
                                                {
                                                    jobs_to_resume_interrupted.push(job.id.clone());
                                                }
                                            } else if ui
                                                .button(format!("▶️ Resume##{}", job.id))
                                                .clicked()
                                            {
                                                jobs_to_resume.push(job.id.clone());
                                            }
                                        }
                                        _ => {
                                            if ui.button(format!("🗑️ Remove##{}", job.id)).clicked()
                                            {
                                                jobs_to_remove.push(job.id.clone());
                                            }
                                        }
                                    }
                                },
                            );
                        });

                        ui.label(format!(
                            "Type: {:?} | Priority: {}",
                            job.job_type, job.priority
                        ));
                        if let Some(material) = &job.material {
                            ui.label(format!("Material: {}", material));
                        }
                        if let Some(tool) = &job.tool {
                            ui.label(format!("Tool: {}", tool));
                        }

                        // Progress bar
                        let progress_bar = egui::ProgressBar::new(job.progress)
                            .show_percentage()
                            .animate(true);
                        ui.add(progress_bar);

                        // Show timing info
                        if let Some(started) = job.started_at {
                            let duration = if let Some(completed) = job.completed_at {
                                completed.signed_duration_since(started)
                            } else {
                                chrono::Utc::now().signed_duration_since(started)
                            };
                            ui.label(format!("Duration: {:.1}s", duration.num_seconds() as f32));
                        }

                        if let Some(error) = &job.error_message {
                            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                        }
                    });
                    ui.separator();
                }

                // Execute collected actions
                for job_id in jobs_to_start {
                    let _ = self.start_job(&job_id);
                }
                for job_id in jobs_to_pause {
                    let _ = self.job.job_queue.pause_job(&job_id);
                }
                for job_id in jobs_to_resume {
                    let _ = self.job.job_queue.resume_job(&job_id);
                }
                for job_id in jobs_to_resume_interrupted {
                    let _ = self.resume_job(&job_id);
                }
                for job_id in jobs_to_cancel {
                    let _ = self.job.job_queue.cancel_job(&job_id);
                }
                for job_id in jobs_to_remove {
                    self.job.job_queue.remove_job(&job_id);
                }
            });
        });
    }
}
