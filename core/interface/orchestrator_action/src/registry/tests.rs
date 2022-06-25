use crate::ActionAlreadyRegistered;
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
fn builder_build() {
    let mut builder = OrchestratorActionRegistryBuilder::empty();
    builder
        .register_type::<TestAction>("core.replicante.io/test.1")
        .expect("action should be registered");
    builder
        .register_type::<TestAction>("core.replicante.io/test.2")
        .expect("action should be registered");
    builder
        .register_type::<TestAction>("core.replicante.io/test.3")
        .expect("action should be registered");
    assert_eq!(builder.actions.len(), 3);

    let registry = builder.build();
    assert_eq!(registry.actions.len(), 3);
    assert!(registry.actions.contains_key("core.replicante.io/test.1"));
    assert!(registry.actions.contains_key("core.replicante.io/test.2"));
    assert!(registry.actions.contains_key("core.replicante.io/test.3"));
}

#[test]
fn builder_empty() {
    let builder = OrchestratorActionRegistryBuilder::empty();
    assert_eq!(builder.actions.len(), 0);
}

#[test]
fn builder_register_boxed() {
    let action = Box::new(TestAction {});
    let mut builder = OrchestratorActionRegistryBuilder::empty();
    builder
        .register_boxed("core.replicante.io/test.boxed", action)
        .expect("action should be registered");
    assert_eq!(builder.actions.len(), 1);
}

#[test]
fn builder_register_generic() {
    let action = TestAction {};
    let mut builder = OrchestratorActionRegistryBuilder::empty();
    builder
        .register("core.replicante.io/test.generic", action)
        .expect("action should be registered");
    assert_eq!(builder.actions.len(), 1);
}

#[test]
fn builder_register_type() {
    let mut builder = OrchestratorActionRegistryBuilder::empty();
    builder
        .register_type::<TestAction>("core.replicante.io/test.generic")
        .expect("action should be registered");
    assert_eq!(builder.actions.len(), 1);
}

#[test]
fn builder_register_error_on_duplicate() {
    let action = TestAction {};
    let mut builder = OrchestratorActionRegistryBuilder::empty();

    // Insert the action the fist time.
    builder
        .register("core.replicante.io/test.generic", action)
        .expect("action should be registered");
    assert_eq!(builder.actions.len(), 1);

    // Then again to check the duplicate logic.
    let action = TestAction {};
    let check = builder.register("core.replicante.io/test.generic", action);
    match check {
        Ok(_) => panic!("should have failed on duplicate action"),
        Err(error) if error.is::<ActionAlreadyRegistered>() => (),
        Err(error) => panic!("unexpected error {:?}", error),
    }
    assert_eq!(builder.actions.len(), 1);
}

#[test]
fn global_registry_lookup_and_init() {
    // Attempts to access the global registry before init result in panic.
    let before_init_thread = std::thread::spawn(move || {
        OrchestratorActionRegistry::current();
    });
    if let Ok(_) = before_init_thread.join() {
        panic!("access before initialisation should panic");
    }

    // Initialise the global registry.
    let mut builder = OrchestratorActionRegistryBuilder::empty();
    builder
        .register_type::<TestAction>("core.replicante.io/test.action")
        .expect("action should be registered");
    builder.build_as_current();

    // Check the registry is set.
    let registry = super::ORCH_ACT_REG
        .read()
        .expect("ORCH_ACT_REG is poisoned")
        .as_ref()
        .map(|registry| registry.clone())
        .expect("ORCH_ACT_REG should be initialised at this point");
    assert_eq!(registry.actions.len(), 1);

    // Fetch the global registry to check it includes the expected action.
    let registry = OrchestratorActionRegistry::current();
    assert_eq!(registry.actions.len(), 1);

    // Ensure attempts to initialise the global registry twice panics.
    let dual_init_thread = std::thread::spawn(move || {
        let builder = OrchestratorActionRegistryBuilder::empty();
        builder.build_as_current();
    });
    if let Ok(_) = dual_init_thread.join() {
        panic!("dual initialisation should panic");
    }
}

#[test]
fn registry_iter() {
    let mut builder = OrchestratorActionRegistryBuilder::empty();
    builder
        .register_type::<TestAction>("core.replicante.io/test.2")
        .expect("action should be registered");
    builder
        .register_type::<TestAction>("core.replicante.io/test.1")
        .expect("action should be registered");
    builder
        .register_type::<TestAction>("core.replicante.io/test.3")
        .expect("action should be registered");
    assert_eq!(builder.actions.len(), 3);

    let registry = builder.build();
    let names: Vec<&str> = registry.iter().map(|(id, _)| id).collect();
    assert_eq!(
        names,
        vec![
            "core.replicante.io/test.1",
            "core.replicante.io/test.2",
            "core.replicante.io/test.3",
        ]
    )
}

#[test]
fn registry_lookup() {
    let mut builder = OrchestratorActionRegistryBuilder::empty();
    builder
        .register_type::<TestAction>("core.replicante.io/test.action")
        .expect("action should be registered");
    let registry = builder.build();
    let action = registry.lookup("core.replicante.io/test.action");
    assert!(action.is_some());
}
