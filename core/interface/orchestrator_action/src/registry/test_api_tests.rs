use crate::OrchestratorAction;
use crate::OrchestratorActionRegistry;
use crate::OrchestratorActionRegistryBuilder;

/// Dummy action to test types and interfaces.
#[derive(Default)]
struct TestAction {}
impl OrchestratorAction for TestAction {
    fn describe(&self) -> crate::OrchestratorActionDescriptor {
        crate::OrchestratorActionDescriptor {
            summary: "A test action".into(),
        }
    }
}

#[test]
#[should_panic(expected = "accessed current OrchestratorActionRegistry before it is initialised")]
fn access_without_registry_panics() {
    let _ = OrchestratorActionRegistry::current();
}

#[test]
fn registry_access() {
    let mut builder = OrchestratorActionRegistryBuilder::empty();
    builder
        .register_type::<TestAction>("core.replicante.io/test.action")
        .expect("action should be registered");
    builder.build_as_current();

    let registry = OrchestratorActionRegistry::current();
    let action = registry.lookup("core.replicante.io/test.action");
    assert!(action.is_some());
}

#[test]
fn registry_clear_api() {
    let mut builder = OrchestratorActionRegistryBuilder::empty();
    builder
        .register_type::<TestAction>("core.replicante.io/test.action")
        .expect("action should be registered");
    builder.build_as_current();
    let registry = OrchestratorActionRegistry::current();
    assert_eq!(registry.actions.len(), 1);

    OrchestratorActionRegistryBuilder::clear_current();

    let mut builder = OrchestratorActionRegistryBuilder::empty();
    builder
        .register_type::<TestAction>("core.replicante.io/test.action")
        .expect("action should be registered");
    builder
        .register_type::<TestAction>("core.replicante.io/test.action.two")
        .expect("action should be registered");
    builder.build_as_current();
    let registry = OrchestratorActionRegistry::current();
    assert_eq!(registry.actions.len(), 2);
}

#[test]
fn registry_clear_guard() {
    let mut builder = OrchestratorActionRegistryBuilder::empty();
    builder
        .register_type::<TestAction>("core.replicante.io/test.action")
        .expect("action should be registered");
    builder.build_as_current();

    // Start nested scope after which the registry is cleared.
    {
        let _guard = crate::TestRegistryClearGuard {};
        let registry = OrchestratorActionRegistry::current();
        assert_eq!(registry.actions.len(), 1);
    }

    // Re-building a new registry will work again now.
    let mut builder = OrchestratorActionRegistryBuilder::empty();
    builder
        .register_type::<TestAction>("core.replicante.io/test.action")
        .expect("action should be registered");
    builder.build_as_current();
}

#[test]
fn registry_is_different_for_threads() {
    let thread_one = std::thread::spawn(move || {
        let mut builder = OrchestratorActionRegistryBuilder::empty();
        builder
            .register_type::<TestAction>("core.replicante.io/test.action")
            .expect("action should be registered");
        builder.build_as_current();

        let registry = OrchestratorActionRegistry::current();
        assert_eq!(registry.actions.len(), 1);
    });
    let thread_two = std::thread::spawn(move || {
        let mut builder = OrchestratorActionRegistryBuilder::empty();
        builder
            .register_type::<TestAction>("core.replicante.io/test.action")
            .expect("action should be registered");
        builder
            .register_type::<TestAction>("core.replicante.io/test.action.two")
            .expect("action should be registered");
        builder.build_as_current();

        let registry = OrchestratorActionRegistry::current();
        assert_eq!(registry.actions.len(), 2);
    });

    assert!(thread_one.join().is_ok());
    assert!(thread_two.join().is_ok());
}

#[test]
#[should_panic(
    expected = "attempted dual initialisation of the current OrchestratorActionRegistry"
)]
fn registry_set_twice_panics() {
    let mut builder = OrchestratorActionRegistryBuilder::empty();
    builder
        .register_type::<TestAction>("core.replicante.io/test.action")
        .expect("action should be registered");
    builder.build_as_current();

    let mut builder = OrchestratorActionRegistryBuilder::empty();
    builder
        .register_type::<TestAction>("core.replicante.io/test.action")
        .expect("action should be registered");
    builder
        .register_type::<TestAction>("core.replicante.io/test.action.two")
        .expect("action should be registered");
    builder.build_as_current();
}
