/// Macro to create a `registry_entry` function for an `OrchestratorAction` impl with `Default`.
///
/// The function is created as a public function in the given handler and it will:
///
///  * Instantiate the handler type with its `Default` implementation.
///  * Create a metadata object with the given information.
///  * "Package" all the information for registration with an `OrchestratorActionRegistryBuilder`.
#[macro_export]
macro_rules! registry_entry_factory {
    (
        handler: $handler:ty,
        schedule_mode: $schedule_mode:expr,
        summary: $summary:expr,
        timeout: $timeout:expr,
    ) => {
        impl $handler {
            /// Create an entry to register the action for global lookup.
            pub fn registry_entry() -> $crate::OrchestratorActionRegistryEntry {
                let handler = Box::new(<$handler>::default());
                let metadata = $crate::OrchestratorActionMetadata {
                    schedule_mode: $schedule_mode,
                    summary: String::from($summary),
                    timeout: std::time::Duration::from($timeout),
                };
                $crate::OrchestratorActionRegistryEntry { handler, metadata }
            }
        }
    };
}
