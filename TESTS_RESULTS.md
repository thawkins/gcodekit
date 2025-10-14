# Test Results

```
running 190 tests
test communication::fluidnc::tests::test_jog_command_formatting ... ok
test communication::fluidnc::tests::test_gcode_line_sending ... ok
test communication::fluidnc::tests::test_parse_grbl_status_invalid ... ok
test communication::fluidnc::tests::test_fluidnc_communication_new ... ok
test communication::fluidnc::tests::test_parse_grbl_status_complex ... ok
test communication::fluidnc::tests::test_parse_grbl_status_minimal ... ok
test communication::fluidnc::tests::test_disconnected_operations ... ok
test communication::fluidnc::tests::test_parse_position ... ok
test communication::fluidnc::tests::test_recovery_command_max_retries_exceeded ... ok
test communication::fluidnc::tests::test_recovery_connection_error ... ok
test communication::fluidnc::tests::test_recovery_command_error ... ok
test communication::fluidnc::tests::test_home_command ... ok
test communication::fluidnc::tests::test_recovery_disabled ... ok
test communication::fluidnc::tests::test_recovery_is_recovering ... ok
test communication::fluidnc::tests::test_connection_state_management ... ok
test communication::fluidnc::tests::test_recovery_critical_error ... ok
test communication::fluidnc::tests::test_error_recovery_config ... ok
test communication::fluidnc::tests::test_parse_grbl_status ... ok
test communication::fluidnc::tests::test_override_commands ... ok
test communication::fluidnc::tests::test_override_commands_edge_cases ... ok
test communication::fluidnc::tests::test_recovery_max_attempts_exceeded ... ok
test communication::fluidnc::tests::test_recovery_state_initialization ... ok
test communication::fluidnc::tests::test_recovery_state_reset ... ok
test communication::g2core::tests::test_connection_state_management ... ok
test communication::g2core::tests::test_disconnected_operations ... ok
test communication::g2core::tests::test_error_recovery_config ... ok
test communication::g2core::tests::test_attempt_recovery ... FAILED
test communication::g2core::tests::test_g2core_communication_new ... ok
test communication::g2core::tests::test_g2core_status_default ... ok
test communication::g2core::tests::test_gcode_line_sending ... ok
test communication::g2core::tests::test_home_command ... ok
test communication::g2core::tests::test_is_recovering ... ok
test communication::g2core::tests::test_jog_command_formatting ... ok
test communication::g2core::tests::test_override_commands ... ok
test communication::g2core::tests::test_recovery_state_initialization ... ok
test communication::g2core::tests::test_parse_g2core_status ... ok
test communication::grbl::tests::test_connection_state_management ... ok
test communication::g2core::tests::test_recovery_state_reset ... FAILED
test communication::grbl::tests::test_control_commands ... ok
test communication::grbl::tests::test_disconnected_operations ... ok
test communication::grbl::tests::test_error_recovery_config ... ok
test communication::grbl::tests::test_gcode_line_sending ... ok
test communication::grbl::tests::test_grbl_communication_new ... ok
test communication::grbl::tests::test_home_command ... ok
test communication::grbl::tests::test_jog_command_edge_cases ... ok
test communication::grbl::tests::test_jog_command_formatting ... ok
test communication::grbl::tests::test_machine_state_from_string ... ok
test communication::grbl::tests::test_machine_state_parsing ... ok
test communication::grbl::tests::test_override_commands ... ok
test communication::grbl::tests::test_override_commands_edge_cases ... ok
test communication::grbl::tests::test_parse_grbl_response_edge_cases ... ok
test communication::grbl::tests::test_parse_grbl_response_types ... ok
test communication::grbl::tests::test_parse_grbl_status ... ok
test communication::grbl::tests::test_parse_grbl_status_complex ... ok
test communication::grbl::tests::test_parse_grbl_status_invalid ... ok
test communication::grbl::tests::test_parse_grbl_status_minimal ... ok
test communication::grbl::tests::test_realtime_override_commands ... ok
test communication::grbl::tests::test_parse_grbl_version ... ok
test communication::grbl::tests::test_recovery_command_error ... ok
test communication::grbl::tests::test_recovery_connection_error ... ok
test communication::grbl::tests::test_recovery_critical_error ... ok
test communication::grbl::tests::test_recovery_disabled ... ok
test communication::grbl::tests::test_recovery_is_recovering ... ok
test communication::grbl::tests::test_recovery_command_max_retries_exceeded ... ok
test communication::grbl::tests::test_recovery_state_initialization ... ok
test communication::grbl::tests::test_recovery_max_attempts_exceeded ... ok
test communication::grbl::tests::test_recovery_state_reset ... ok
test communication::smoothieware::tests::test_connection_state_management ... ok
test communication::smoothieware::tests::test_disconnected_operations ... ok
test communication::smoothieware::tests::test_error_recovery_command_error ... ok
test communication::smoothieware::tests::test_error_recovery_connection_error ... ok
test communication::smoothieware::tests::test_error_recovery_critical_error ... ok
test communication::smoothieware::tests::test_error_recovery_disabled ... ok
test communication::smoothieware::tests::test_error_recovery_max_attempts ... ok
test communication::smoothieware::tests::test_gcode_line_sending ... ok
test communication::smoothieware::tests::test_home_command ... ok
test communication::smoothieware::tests::test_is_recovering ... ok
test communication::smoothieware::tests::test_jog_command_edge_cases ... ok
test communication::smoothieware::tests::test_jog_command_formatting ... ok
test communication::smoothieware::tests::test_machine_state_from_string ... ok
test communication::smoothieware::tests::test_override_commands ... ok
test communication::smoothieware::tests::test_override_commands_edge_cases ... ok
test communication::smoothieware::tests::test_parse_smoothieware_response_edge_cases ... ok
test communication::smoothieware::tests::test_parse_smoothieware_response_error ... ok
test communication::smoothieware::tests::test_parse_smoothieware_response_feedback ... ok
test communication::smoothieware::tests::test_parse_smoothieware_response_ok ... ok
test communication::smoothieware::tests::test_parse_smoothieware_response_other ... ok
test communication::smoothieware::tests::test_parse_smoothieware_response_status ... ok
test communication::smoothieware::tests::test_parse_smoothieware_response_version ... ok
test communication::smoothieware::tests::test_parse_smoothieware_status_complex ... ok
test communication::smoothieware::tests::test_parse_smoothieware_status_invalid ... ok
test communication::smoothieware::tests::test_parse_smoothieware_status_minimal ... ok
test communication::smoothieware::tests::test_reset_recovery_state ... ok
test communication::smoothieware::tests::test_smoothieware_communication_new ... ok
test communication::tinyg::tests::test_connection_state_equality ... ok
test communication::tinyg::tests::test_connection_state_management ... ok
test communication::tinyg::tests::test_feed_hold ... ok
test communication::tinyg::tests::test_get_tinyg_settings ... ok
test communication::tinyg::tests::test_parse_tinyg_response ... ok
test communication::tinyg::tests::test_query_realtime_status ... ok
test communication::tinyg::tests::test_read_response_no_data ... ok
test communication::tinyg::tests::test_reset_tinyg ... ok
test communication::tinyg::tests::test_resume ... ok
test communication::tinyg::tests::test_send_gcode_line_disconnected ... ok
test communication::tinyg::tests::test_set_tinyg_setting ... ok
test communication::tinyg::tests::test_tinyg_communication_new ... ok
test communication::tinyg::tests::test_tinyg_response_equality ... ok
test communication::tinyg::tests::test_wcs_coordinate ... ok
test designer::tests::test_add_shape_command ... ok
test designer::tests::test_align_shapes ... ok
test designer::tests::test_delete_shape_command ... ok
test designer::tests::test_export_empty_designer_to_gcode ... ok
test designer::tests::test_export_circle_to_gcode ... ok
test designer::tests::test_export_line_to_gcode ... ok
test designer::tests::test_export_to_obj_empty ... ok
test designer::tests::test_export_rectangle_to_gcode ... ok
test designer::tests::test_export_to_stl_empty ... ok
test designer::tests::test_get_shape_pos ... ok
test designer::tests::test_undo_redo ... ok
test gcodeedit::tests::test_gcode_editor_new ... ok
test gcodeedit::tests::test_optimize_gcode_empty ... ok
test gcodeedit::tests::test_parse_empty_gcode ... ok
test gcodeedit::tests::test_parse_gcode_multiple_axes ... ok
test gcodeedit::tests::test_optimize_gcode_with_comments ... FAILED
test gcodeedit::tests::test_parse_gcode_with_arcs ... ok
test gcodeedit::tests::test_parse_gcode_with_comments ... ok
test gcodeedit::tests::test_parse_simple_gcode ... ok
test gcodeedit::tests::test_search_case_insensitive ... ok
test gcodeedit::tests::test_search_empty_query ... ok
test gcodeedit::tests::test_optimize_gcode_remove_empty_lines ... FAILED
test gcodeedit::tests::test_search_multiple_results ... ok
test gcodeedit::tests::test_search_next ... ok
test gcodeedit::tests::test_search_next_empty_results ... ok
test gcodeedit::tests::test_search_prev ... ok
test gcodeedit::tests::test_search_single_result ... ok
test jobs::manager::tests::test_cancel_job ... ok
test jobs::manager::tests::test_complete_job ... ok
test jobs::manager::tests::test_create_job_from_generated_gcode ... ok
test jobs::manager::tests::test_create_job_from_generated_gcode_empty ... ok
test jobs::manager::tests::test_create_job_from_generated_gcode_no_material ... ok
test jobs::manager::tests::test_fail_job ... ok
test jobs::manager::tests::test_get_current_job ... ok
test jobs::manager::tests::test_get_current_job_id ... ok
test jobs::manager::tests::test_get_current_job_no_active ... ok
test jobs::manager::tests::test_job_manager_new ... ok
test jobs::manager::tests::test_pause_job ... ok
test jobs::manager::tests::test_replace_job_queue ... ok
test jobs::manager::tests::test_resume_job ... ok
test jobs::manager::tests::test_resume_job_cannot_resume ... ok
test jobs::manager::tests::test_resume_job_not_found ... ok
test jobs::manager::tests::test_start_job ... ok
test jobs::manager::tests::test_start_job_not_found ... ok
test jobs::manager::tests::test_update_job_progress ... ok
test jobs::tests::test_job_creation ... ok
test jobs::tests::test_job_lifecycle ... ok
test jobs::tests::test_job_queue ... ok
test jobs::manager::tests::test_save_jobs_to_file ... ok
test jobs::manager::tests::test_load_jobs_from_file ... ok
test jobs::tests::test_job_resumption_integration ... ok
test jobs::tests::test_job_resumption_with_invalid_job ... ok
test materials::properties::tests::test_builder_pattern_chaining ... ok
test materials::properties::tests::test_material_properties_default_values ... ok
test materials::properties::tests::test_material_properties_new ... ok
test jobs::tests::test_job_queue_save_load_empty ... ok
test materials::properties::tests::test_with_density ... ok
test jobs::tests::test_job_queue_save_load ... ok
test materials::properties::tests::test_with_hardness ... ok
test jobs::manager::tests::test_save_load_integration ... ok
test materials::properties::tests::test_with_machining_params ... ok
test materials::properties::tests::test_with_tool_recommendations ... ok
test materials::properties::tests::test_with_tool_recommendations_no_coating ... ok
test materials::types::tests::test_all_material_types_serialize_deserialize ... ok
test materials::types::tests::test_all_material_subtypes_serialize_deserialize ... ok
test materials::types::tests::test_material_subtype_equality ... ok
test materials::types::tests::test_material_subtype_serialization ... ok
test materials::types::tests::test_material_type_equality ... ok
test materials::types::tests::test_material_type_serialization ... ok
test postprocessor::tests::test_command_to_gcode ... ok
test postprocessor::tests::test_grbl_post_processor ... ok
test postprocessor::tests::test_parse_comment_only ... ok
test postprocessor::tests::test_parse_empty_line ... ok
test postprocessor::tests::test_parse_gcode_with_comment ... ok
test postprocessor::tests::test_parse_simple_gcode ... ok
test postprocessor::tests::test_post_processor_manager ... ok
test widgets::calibration::tests::test_show_calibration_widget_compiles ... ok
test widgets::jog::tests::test_show_jog_widget_compiles ... ok
test widgets::machine_control::tests::test_show_machine_control_widget_compiles ... ok
test widgets::overrides::tests::test_show_overrides_widget_compiles ... ok
test widgets::safety::tests::test_show_safety_widget_compiles ... ok
test widgets::tool_management::tests::test_show_tool_management_widget_compiles ... ok

failures:

---- communication::g2core::tests::test_attempt_recovery stdout ----
[RECOVERY] Attempting recovery for error: some error
[RECOVERY] Classified as unknown error, attempting controller reset
[RECOVERY] Recovery action taken: ResetController

thread 'communication::g2core::tests::test_attempt_recovery' panicked at src/communication/g2core.rs:603:9:
assertion `left == right` failed
  left: ResetController
 right: RetryCommand
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- communication::g2core::tests::test_recovery_state_reset stdout ----
[RECOVERY] Attempting recovery for error: error
[RECOVERY] Classified as unknown error, attempting controller reset
[RECOVERY] Recovery action taken: ResetController

thread 'communication::g2core::tests::test_recovery_state_reset' panicked at src/communication/g2core.rs:614:9:
assertion `left == right` failed
  left: 1
 right: 0

---- gcodeedit::tests::test_optimize_gcode_with_comments stdout ----

thread 'gcodeedit::tests::test_optimize_gcode_with_comments' panicked at src/gcodeedit/mod.rs:888:9:
assertion `left == right` failed
  left: "G-code optimized: 5 -> 2 lines, 89 -> 22 bytes (75% reduction)"
 right: "G-code optimized: 5 -> 2 lines"

---- gcodeedit::tests::test_optimize_gcode_remove_empty_lines stdout ----

thread 'gcodeedit::tests::test_optimize_gcode_remove_empty_lines' panicked at src/gcodeedit/mod.rs:148:14:
attempt to subtract with overflow


failures:
    communication::g2core::tests::test_attempt_recovery
    communication::g2core::tests::test_recovery_state_reset
    gcodeedit::tests::test_optimize_gcode_remove_empty_lines
    gcodeedit::tests::test_optimize_gcode_with_comments

test result: FAILED. 186 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
