use egui;
use rfd;

use crate::GcodeKitApp;

/// Renders the job manager tab UI showing job queue, controls, and status.
/// Provides interface for creating, starting, pausing, and managing jobs.
pub fn show_job_manager_tab(app: &mut GcodeKitApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.heading("Job Manager");
        ui.separator();

        // Job queue controls
        ui.horizontal(|ui| {
            if ui.button("➕ Add Job").clicked() {
                app.ui.show_job_creation_dialog = true;
                app.ui.new_job_name = "New Job".to_string();
                app.ui.new_job_type = crate::jobs::JobType::GcodeFile;
                app.ui.new_job_file_path.clear();
                app.ui.selected_material = None;
            }
            if ui.button("🗑️ Clear Completed").clicked() {
                app.job.job_queue.clear_completed_jobs();
            }
            ui.label(format!("Jobs: {}", app.job.job_queue.jobs.len()));
        });

        // Job creation dialog
        if app.ui.show_job_creation_dialog {
            egui::Window::new("Create New Job")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.vertical(|ui| {
                        ui.label("Job Name:");
                        ui.text_edit_singleline(&mut app.ui.new_job_name);

                        ui.label("Job Type:");
                        egui::ComboBox::from_id_salt("job_type_combobox")
                            .selected_text(format!("{:?}", app.ui.new_job_type))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut app.ui.new_job_type,
                                    crate::jobs::JobType::GcodeFile,
                                    "G-code File",
                                );
                                ui.selectable_value(
                                    &mut app.ui.new_job_type,
                                    crate::jobs::JobType::CAMOperation,
                                    "CAM Operation",
                                );
                                ui.selectable_value(
                                    &mut app.ui.new_job_type,
                                    crate::jobs::JobType::Probing,
                                    "Probing",
                                );
                                ui.selectable_value(
                                    &mut app.ui.new_job_type,
                                    crate::jobs::JobType::Calibration,
                                    "Calibration",
                                );
                                ui.selectable_value(
                                    &mut app.ui.new_job_type,
                                    crate::jobs::JobType::Maintenance,
                                    "Maintenance",
                                );
                            });

                        if app.ui.new_job_type == crate::jobs::JobType::GcodeFile {
                            ui.label("File Path:");
                            ui.horizontal(|ui| {
                                ui.text_edit_singleline(&mut app.ui.new_job_file_path);
                                if ui.button("Browse").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                                        app.ui.new_job_file_path = path.display().to_string();
                                    }
                                }
                            });
                        }

                        ui.label("Material:");
                        let mut material_names: Vec<String> = app
                            .material_database
                            .get_all_materials()
                            .iter()
                            .map(|m| m.name.clone())
                            .collect();
                        material_names.insert(0, "None".to_string());

                        let current_selection = app
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
                                        == app.ui.selected_material
                                        || (material_name == "None"
                                            && app.ui.selected_material.is_none());
                                    if ui.selectable_label(is_selected, material_name).clicked() {
                                        if material_name == "None" {
                                            app.ui.selected_material = None;
                                        } else {
                                            app.ui.selected_material = Some(material_name.clone());
                                        }
                                    }
                                }
                            });

                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("Create").clicked() {
                                let job_name = app.ui.new_job_name.clone();
                                let job_type = app.ui.new_job_type.clone();
                                let selected_material_clone = app.ui.selected_material.clone();
                                let mut job = crate::jobs::Job::new(job_name, job_type.clone());
                                if let Some(material) = &selected_material_clone {
                                    job = job.with_material(material.clone());
                                }
                                if job_type == crate::jobs::JobType::GcodeFile {
                                    job.gcode_path = Some(std::path::PathBuf::from(
                                        app.ui.new_job_file_path.clone(),
                                    ));
                                }
                                app.job.job_queue.add_job(job);
                                app.ui.show_job_creation_dialog = false;
                                app.ui.new_job_file_path.clear();
                            }
                            if ui.button("Cancel").clicked() {
                                app.ui.show_job_creation_dialog = false;
                                app.ui.new_job_file_path.clear();
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
            let jobs_data: Vec<_> = app.job.job_queue.jobs.iter().cloned().collect();

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

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(format!("{:.1}%", job.progress * 100.0));

                            // Control buttons - collect actions instead of executing immediately
                            match job.status {
                                crate::jobs::JobStatus::Pending => {
                                    if ui.button(format!("▶️ Start##{}", job.id)).clicked() {
                                        jobs_to_start.push(job.id.clone());
                                    }
                                }
                                crate::jobs::JobStatus::Running => {
                                    if ui.button(format!("⏸️ Pause##{}", job.id)).clicked() {
                                        jobs_to_pause.push(job.id.clone());
                                    }
                                    if ui.button(format!("⏹️ Stop##{}", job.id)).clicked() {
                                        jobs_to_cancel.push(job.id.clone());
                                    }
                                }
                                crate::jobs::JobStatus::Paused => {
                                    if job.can_resume_job() {
                                        if ui.button(format!("🔄 Resume##{}", job.id)).clicked() {
                                            jobs_to_resume_interrupted.push(job.id.clone());
                                        }
                                    } else if ui.button(format!("▶️ Resume##{}", job.id)).clicked()
                                    {
                                        jobs_to_resume.push(job.id.clone());
                                    }
                                }
                                _ => {
                                    if ui.button(format!("🗑️ Remove##{}", job.id)).clicked() {
                                        jobs_to_remove.push(job.id.clone());
                                    }
                                }
                            }
                        });
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
                let _ = app.start_job(&job_id);
            }
            for job_id in jobs_to_pause {
                let _ = app.job.job_queue.pause_job(&job_id);
            }
            for job_id in jobs_to_resume {
                let _ = app.job.job_queue.resume_job(&job_id);
            }
            for job_id in jobs_to_resume_interrupted {
                let _ = app.resume_job(&job_id);
            }
            for job_id in jobs_to_cancel {
                let _ = app.job.job_queue.cancel_job(&job_id);
            }
            for job_id in jobs_to_remove {
                app.job.job_queue.remove_job(&job_id);
            }
        });
    });
}
