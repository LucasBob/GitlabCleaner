use tiny_tokio_actor::SystemEvent;

/// Event used in the event bus of the system.
#[derive(Clone, Debug)]
pub struct Event(String);

/// Implement the `SystemEvent` trait for the `Event` struct.
impl SystemEvent for Event {}

