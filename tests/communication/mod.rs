use gcodekit::communication::*;

#[cfg(test)]
mod grbl_tests {
    use super::*;
    use gcodekit::MachinePosition;

    #[test]
    fn test_grbl_communication_creation() {
        let grbl = GrblCommunication::default();
        assert_eq!(*grbl.get_connection_state(), ConnectionState::Disconnected);
        assert_eq!(grbl.get_selected_port(), "");
        assert!(!grbl.is_connected());
    }

    #[test]
    fn test_grbl_set_port() {
        let mut grbl = GrblCommunication::default();
        grbl.set_port("/dev/ttyUSB0".to_string());
        assert_eq!(grbl.get_selected_port(), "/dev/ttyUSB0");
    }

    #[test]
    fn test_grbl_status_message() {
        let grbl = GrblCommunication::default();
        let status = grbl.get_status_message();
        assert!(!status.is_empty());
    }

    #[test]
    fn test_grbl_version() {
        let grbl = GrblCommunication::default();
        let version = grbl.get_version();
        assert!(!version.is_empty());
    }

    #[test]
    fn test_grbl_recovery_config() {
        let grbl = GrblCommunication::default();
        let config = grbl.get_recovery_config();
        assert_eq!(config.max_reconnect_attempts, 3);
        assert_eq!(config.reconnect_delay_ms, 2000);
        assert_eq!(config.max_command_retries, 3);
        assert_eq!(config.command_retry_delay_ms, 1000);
        assert!(config.reset_on_critical_error);
        assert!(config.auto_recovery_enabled);
    }

    #[test]
    fn test_grbl_set_recovery_config() {
        let mut grbl = GrblCommunication::default();
        let new_config = ErrorRecoveryConfig {
            max_reconnect_attempts: 5,
            reconnect_delay_ms: 3000,
            max_command_retries: 4,
            command_retry_delay_ms: 1500,
            reset_on_critical_error: false,
            auto_recovery_enabled: true,
        };
        grbl.set_recovery_config(new_config.clone());
        let config = grbl.get_recovery_config();
        assert_eq!(config.max_reconnect_attempts, 5);
        assert_eq!(config.reconnect_delay_ms, 3000);
        assert_eq!(config.max_command_retries, 4);
        assert_eq!(config.command_retry_delay_ms, 1500);
        assert!(!config.reset_on_critical_error);
        assert!(config.auto_recovery_enabled);
    }

    #[test]
    fn test_grbl_recovery_state() {
        let grbl = GrblCommunication::default();
        let state = grbl.get_recovery_state();
        assert_eq!(state.reconnect_attempts, 0);
        assert_eq!(state.command_retry_count, 0);
        assert!(state.last_reconnect_attempt.is_none());
        assert!(state.last_error.is_none());
        assert!(state.recovery_actions_taken.is_empty());
    }

    #[test]
    fn test_grbl_not_recovering_initially() {
        let grbl = GrblCommunication::default();
        assert!(!grbl.is_recovering());
    }

    #[test]
    fn test_grbl_health_metrics() {
        let grbl = GrblCommunication::default();
        let metrics = grbl.get_health_metrics();
        assert_eq!(metrics.connection_stability, 1.0);
        assert_eq!(metrics.command_success_rate, 1.0);
        assert_eq!(metrics.uptime_percentage, 1.0);
        assert!(metrics.error_patterns.is_empty());
    }

    #[test]
    fn test_health_metrics_update_error_pattern() {
        let mut metrics = HealthMetrics::default();
        metrics.update_error_pattern("connection timeout");

        assert_eq!(metrics.error_patterns.len(), 1);
        assert_eq!(metrics.error_patterns[0].error_type, "connection timeout");
        assert_eq!(metrics.error_patterns[0].frequency, 1);
        assert_eq!(metrics.error_patterns[0].severity_score, 0.1);

        // Add same error again
        metrics.update_error_pattern("connection timeout");
        assert_eq!(metrics.error_patterns.len(), 1);
        assert_eq!(metrics.error_patterns[0].frequency, 2);
    }

    #[test]
    fn test_health_metrics_predict_connection_issues() {
        let mut metrics = HealthMetrics::default();

        // Add multiple connection errors
        for _ in 0..6 {
            metrics.update_error_pattern("connection timeout");
        }

        let issues = metrics.predict_potential_issues();
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.contains("connection")));
    }

    #[test]
    fn test_health_metrics_predict_command_issues() {
        let mut metrics = HealthMetrics::default();

        // Add multiple command errors
        for _ in 0..4 {
            metrics.update_error_pattern("command syntax error");
        }

        let issues = metrics.predict_potential_issues();
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.contains("command")));
    }

    #[test]
    fn test_connection_state_transitions() {
        assert_eq!(ConnectionState::default(), ConnectionState::Disconnected);

        let states = vec![
            ConnectionState::Disconnected,
            ConnectionState::Connecting,
            ConnectionState::Connected,
            ConnectionState::Error,
            ConnectionState::Recovering,
        ];

        for state in states {
            let cloned = state.clone();
            assert_eq!(state, cloned);
        }
    }

    #[test]
    fn test_recovery_action_types() {
        let actions = vec![
            RecoveryAction::Reconnect,
            RecoveryAction::RetryCommand,
            RecoveryAction::ResetController,
            RecoveryAction::SkipCommand,
            RecoveryAction::AbortJob,
        ];

        for action in actions {
            let cloned = action.clone();
            assert_eq!(action, cloned);
        }
    }

    #[test]
    fn test_controller_type() {
        let controller = ControllerType::Grbl;
        assert_eq!(controller, ControllerType::Grbl);
    }
}

#[cfg(test)]
mod error_recovery_tests {
    use super::*;

    #[test]
    fn test_recovery_config_default() {
        let config = ErrorRecoveryConfig::default();
        assert_eq!(config.max_reconnect_attempts, 3);
        assert_eq!(config.reconnect_delay_ms, 2000);
        assert_eq!(config.max_command_retries, 3);
        assert_eq!(config.command_retry_delay_ms, 1000);
        assert!(config.reset_on_critical_error);
        assert!(config.auto_recovery_enabled);
    }

    #[test]
    fn test_recovery_state_default() {
        let state = RecoveryState::default();
        assert_eq!(state.reconnect_attempts, 0);
        assert_eq!(state.command_retry_count, 0);
        assert!(state.last_reconnect_attempt.is_none());
        assert!(state.last_error.is_none());
        assert!(state.recovery_actions_taken.is_empty());
    }

    #[test]
    fn test_grbl_attempt_recovery_connection_error() {
        let mut grbl = GrblCommunication::default();
        let result = grbl.attempt_recovery("connection lost");

        assert!(result.is_ok());
        let action = result.unwrap();
        assert_eq!(action, RecoveryAction::Reconnect);

        // Check recovery state updated
        let state = grbl.get_recovery_state();
        assert!(state.last_error.is_some());
        assert_eq!(state.last_error.as_ref().unwrap(), "connection lost");
    }

    #[test]
    fn test_grbl_reset_recovery_state() {
        let mut grbl = GrblCommunication::default();

        // Trigger recovery
        grbl.attempt_recovery("test error").ok();

        // Reset
        grbl.reset_recovery_state();

        // Verify state is clean
        let state = grbl.get_recovery_state();
        assert_eq!(state.reconnect_attempts, 0);
        assert_eq!(state.command_retry_count, 0);
        assert!(state.recovery_actions_taken.is_empty());
    }

    #[test]
    fn test_grbl_perform_health_check() {
        let mut grbl = GrblCommunication::default();
        let warnings = grbl.perform_health_check();

        // Initially should have no warnings
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_grbl_optimize_settings() {
        let mut grbl = GrblCommunication::default();
        let optimizations = grbl.optimize_settings_based_on_health();

        // With perfect health, no optimizations needed
        assert!(optimizations.is_empty());
    }
}
