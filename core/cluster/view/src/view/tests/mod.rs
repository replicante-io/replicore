use crate::ClusterView;
use crate::ClusterViewCorrupt;

mod fixtures;

#[test]
fn fail_when_starting_with_different_cluster_ids() {
    let discovery = self::fixtures::cluster_mongodb::discovery();
    let settings = self::fixtures::cluster_zookeeper::settings();
    let builder = ClusterView::builder(settings, discovery);
    let err = builder
        .err()
        .expect("different cluster ids did not fail building")
        .downcast::<ClusterViewCorrupt>()
        .expect("unexpected error type");
    match err {
        ClusterViewCorrupt::ClusterIdClash(namespace, expected, found) => {
            assert_eq!(namespace, self::fixtures::cluster_mongodb::NAMESPACE);
            assert_eq!(expected, self::fixtures::cluster_zookeeper::CLUSTER_ID);
            assert_eq!(found, self::fixtures::cluster_mongodb::CLUSTER_ID);
        }
        _ => panic!("unexpected error value"),
    };
}
