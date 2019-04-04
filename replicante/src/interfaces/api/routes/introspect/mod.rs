use super::APIVersion;
use super::RouterBuilder;

mod threads;

pub fn mount(router: &mut RouterBuilder) {
    let mut unstable = router.for_version(APIVersion::Unstable);
    unstable.get("/introspect/threads", threads::handler, "/introspect/threads");
}
