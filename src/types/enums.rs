#[derive(Clone, Default, PartialEq, Debug)]
pub enum Tab {
    #[default]
    Designer,
    GcodeEditor,
    Visualizer3D,
    DeviceConsole,
    JobManager,
    FeedsSpeeds,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum MachineMode {
    #[default]
    CNC,
    Laser,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum MoveType {
    #[default]
    Rapid,
    Feed,
    Arc,
}

#[derive(Clone, Debug, Default)]
pub struct PathSegment {
    pub start: crate::types::MachinePosition,
    pub end: crate::types::MachinePosition,
    pub move_type: MoveType,
    pub line_number: usize,
}
