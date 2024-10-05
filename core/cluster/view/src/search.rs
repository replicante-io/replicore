//! Logic to search and sort cluster nodes.
use std::cmp::Ordering;
use std::iter::Take;
use std::sync::Arc;
use std::vec::IntoIter;

use serde_json::Number;

use replisdk::core::models::node::AttributeMatcher;
use replisdk::core::models::node::AttributeMatcherComplex;
use replisdk::core::models::node::AttributeMatcherOp;
use replisdk::core::models::node::AttributeValueRef;
use replisdk::core::models::node::Node;
use replisdk::core::models::node::NodeSearchMatches;

/// Logic to detect and apply ascending and descending sort order during comparison.
struct CompareDirection<'a> {
    attribute: &'a str,
    greater: Ordering,
    less: Ordering,
}

impl<'a> CompareDirection<'a> {
    /// Check a comparison result and adapt it to the detected sorting direction.
    fn direct(&self, result: Ordering) -> Ordering {
        match result {
            Ordering::Equal => Ordering::Equal,
            Ordering::Greater => self.greater,
            Ordering::Less => self.less,
        }
    }

    /// Detect a sorting direction and attribute from a "rule" string.
    fn new(attribute: &'a str) -> CompareDirection<'a> {
        if let Some(attribute) = attribute.strip_prefix("-") {
            CompareDirection {
                attribute,
                greater: Ordering::Less,
                less: Ordering::Greater,
            }
        } else {
            let attr = attribute.strip_prefix("+").unwrap_or(attribute);
            CompareDirection {
                attribute: attr,
                greater: Ordering::Greater,
                less: Ordering::Less,
            }
        }
    }
}

/// Compare two [`Node`]s based on configurable sorting criteria.
///
/// The logic implemented here follows the rules detailed in the [`NodeSearch`] documentation.
///
/// [`NodeSearch`]: replisdk::core::models::node::NodeSearch
pub fn compare(order: &[String]) -> impl Fn(&Arc<Node>, &Arc<Node>) -> Ordering + '_ {
    |left, right| {
        for attribute in order.iter() {
            let direction = CompareDirection::new(attribute);
            let l_value = left.attribute(direction.attribute);
            let r_value = right.attribute(direction.attribute);

            // Order nodes where the attribute is set before those without it.
            let (l_value, r_value) = match (l_value, r_value) {
                (None, None) => continue,
                (Some(_), None) => return direction.less,
                (None, Some(_)) => return direction.greater,
                (Some(l_value), Some(r_value)) => (l_value, r_value),
            };

            // Order nodes where attributes were found:
            //  - When types match use the natural order of values.
            //  - When types don't match order as: number < string < bool < null.
            match l_value {
                AttributeValueRef::Number(left) => match r_value {
                    AttributeValueRef::Number(right) => match compare_num(left, right) {
                        Ordering::Equal => (),
                        result => return direction.direct(result),
                    },
                    _ => return direction.less,
                },
                AttributeValueRef::String(left) => match r_value {
                    AttributeValueRef::String(right) => match left.cmp(right) {
                        Ordering::Equal => (),
                        result => return direction.direct(result),
                    },
                    AttributeValueRef::Number(_) => return direction.greater,
                    _ => return direction.less,
                },
                AttributeValueRef::Boolean(left) => match r_value {
                    AttributeValueRef::Boolean(right) => match left.cmp(&right) {
                        Ordering::Equal => (),
                        result => return direction.direct(result),
                    },
                    AttributeValueRef::String(_) => return direction.greater,
                    AttributeValueRef::Number(_) => return direction.greater,
                    _ => return direction.less,
                },
                AttributeValueRef::Null => match r_value {
                    AttributeValueRef::Null => (),
                    _ => return direction.greater,
                },
            }
        }

        Ordering::Equal
    }
}

/// Compare [`Number`s](serde_json::Number) for node ordering purposes.
///
/// This logic is opinionated in some respects:
///
/// - Numeric types are sorted: `i64` < `u64` < `f64`.
/// - The floating point `NaN` values are considered equal (which is wrong but fine for this).
fn compare_num(left: &Number, right: &Number) -> Ordering {
    // Favour signed 64-bit integers.
    match (left.as_i64(), right.as_i64()) {
        (None, None) => (),
        (Some(_), None) => return Ordering::Less,
        (None, Some(_)) => return Ordering::Greater,
        (Some(left), Some(right)) => return left.cmp(&right),
    };

    // Next are unsigned 64-bit integers.
    match (left.as_u64(), right.as_u64()) {
        (None, None) => (),
        (Some(_), None) => return Ordering::Less,
        (None, Some(_)) => return Ordering::Greater,
        (Some(left), Some(right)) => return left.cmp(&right),
    };

    // Finally compare floats.
    match (left.as_f64(), right.as_f64()) {
        (None, None) => Ordering::Equal,
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (Some(left), Some(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
    }
}

/// Apply potentially complex matching logic to an attribute value.
fn apply_matcher(matcher: &AttributeMatcher, value: AttributeValueRef) -> bool {
    match matcher {
        AttributeMatcher::Complex(complex) => apply_matcher_complex(complex, value),
        AttributeMatcher::Eq(expected) => value == AttributeValueRef::from(expected),
        AttributeMatcher::In(expected) => expected
            .iter()
            .any(|expected| value == AttributeValueRef::from(expected)),
    }
}

/// Apply potentially complex matching logic to an attribute value.
///
/// If the operand for the matching operator is missing the attribute silently does not match.
fn apply_matcher_complex(complex: &AttributeMatcherComplex, value: AttributeValueRef) -> bool {
    match complex.op {
        AttributeMatcherOp::Eq => {
            let expected = match complex.value.as_ref() {
                Some(expected) => expected,
                None => return false,
            };
            value == AttributeValueRef::from(expected)
        }
        AttributeMatcherOp::In => {
            let expected = match complex.values.as_ref() {
                Some(expected) => expected,
                None => return false,
            };
            expected
                .iter()
                .any(|expected| value == AttributeValueRef::from(expected))
        }
        AttributeMatcherOp::Ne => {
            let expected = match complex.value.as_ref() {
                Some(expected) => expected,
                None => return false,
            };
            value != AttributeValueRef::from(expected)
        }
        AttributeMatcherOp::NotIn => {
            let expected = match complex.values.as_ref() {
                Some(expected) => expected,
                None => return false,
            };
            expected
                .iter()
                .all(|expected| value != AttributeValueRef::from(expected))
        }
        AttributeMatcherOp::Set => true,
        AttributeMatcherOp::Unset => false,
    }
}

/// Returns a closure checking if a [`Node`] is selected by the [`NodeSearchMatches`].
///
/// ## Usage
///
/// The function is intended to help keep calling code cleaner and limit the need
/// to "carry" the search around. This results in a bit on indirection as shown below:
///
/// ```ignore
/// let filter = select(&search.matches);
/// let selected_nodes = nodes.iter().filter(filter);
/// ```
pub fn select(matches: &NodeSearchMatches) -> impl Fn(&&Arc<Node>) -> bool + '_ {
    |node| {
        for (attribute, expected) in matches.iter() {
            let actual = match node.attribute(attribute) {
                Some(actual) => actual,
                None => {
                    // Check if the matching operation is `Unset`.
                    if let AttributeMatcher::Complex(complex) = expected {
                        return matches!(complex.op, AttributeMatcherOp::Unset);
                    }
                    return false;
                }
            };
            if !apply_matcher(expected, actual) {
                return false;
            }
        }

        // If all attributes are present on the node and have matching values the node matches.
        true
    }
}

/// Iterate over cluster node search results.
pub enum Iter {
    Take(Take<IntoIter<Arc<Node>>>),
    Vec(IntoIter<Arc<Node>>),
}

impl Iter {
    /// Return the first node found by the search and drop all other results.
    pub fn one(mut self) -> Option<Arc<Node>> {
        self.next()
    }
}

impl From<Take<IntoIter<Arc<Node>>>> for Iter {
    fn from(value: Take<IntoIter<Arc<Node>>>) -> Self {
        Iter::Take(value)
    }
}

impl From<IntoIter<Arc<Node>>> for Iter {
    fn from(value: IntoIter<Arc<Node>>) -> Self {
        Iter::Vec(value)
    }
}

impl Iterator for Iter {
    type Item = Arc<Node>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Take(inner) => inner.next(),
            Iter::Vec(inner) => inner.next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use std::sync::Arc;

    use replisdk::agent::models::AgentVersion;
    use replisdk::agent::models::AttributeValue;
    use replisdk::agent::models::StoreVersion;
    use replisdk::core::models::node::AttributeMatcher;
    use replisdk::core::models::node::AttributeMatcherComplex;
    use replisdk::core::models::node::AttributeMatcherOp;
    use replisdk::core::models::node::AttributeValueRef;
    use replisdk::core::models::node::Node;
    use replisdk::core::models::node::NodeDetails;
    use replisdk::core::models::node::NodeStatus;

    use super::NodeSearchMatches;

    fn mock_node_1() -> Node {
        let details = NodeDetails {
            address: Default::default(),
            agent_version: AgentVersion {
                checkout: "agent-sha".into(),
                number: "1.2.3".into(),
                taint: "test".into(),
            },
            attributes: {
                let mut map = std::collections::BTreeMap::new();
                map.insert("test.attribute".into(), "value".into());
                map.insert("sort.number".into(), serde_json::Number::from(42).into());
                map.insert("sort.bool".into(), true.into());
                map.insert("sort.null".into(), "set".into());
                map
            },
            store_id: "test-store".into(),
            store_version: StoreVersion {
                checkout: None,
                number: "4.5.6".into(),
                extra: Some("mocked".into()),
            },
        };
        Node {
            ns_id: "test-ns-1".into(),
            cluster_id: "test-cluster".into(),
            node_id: "test-node-b".into(),
            details: Some(details),
            node_status: NodeStatus::Unhealthy,
        }
    }

    fn mock_node_2() -> Node {
        let details = NodeDetails {
            address: Default::default(),
            agent_version: AgentVersion {
                checkout: "agent-sha".into(),
                number: "1.2.3".into(),
                taint: "test".into(),
            },
            attributes: {
                let mut map = std::collections::BTreeMap::new();
                map.insert("test.attribute".into(), "value".into());
                map.insert("sort.number".into(), serde_json::Number::from(16).into());
                map.insert("sort.bool".into(), false.into());
                map.insert("sort.null".into(), AttributeValue::Null);
                map
            },
            store_id: "test-store".into(),
            store_version: StoreVersion {
                checkout: None,
                number: "4.5.6".into(),
                extra: Some("mocked".into()),
            },
        };
        Node {
            ns_id: "test-ns-2".into(),
            cluster_id: "test-cluster".into(),
            node_id: "test-node-a".into(),
            details: Some(details),
            node_status: NodeStatus::Unhealthy,
        }
    }

    #[rstest::rstest]
    #[case(vec!["ns_id"], Ordering::Less)]
    #[case(vec!["cluster_id"], Ordering::Equal)]
    #[case(vec!["node_id"], Ordering::Greater)]
    #[case(
        vec![
            "cluster_id",
            "ns_id",
        ],
        Ordering::Less
    )]
    #[case(
        vec![
            "cluster_id",
            "node_id",
        ],
        Ordering::Greater
    )]
    #[case(vec!["sort.number"], Ordering::Greater)]
    #[case(vec!["sort.bool"], Ordering::Greater)]
    #[case(vec!["sort.null"], Ordering::Less)]
    #[case(vec!["-sort.number"], Ordering::Less)]
    fn compare_nodes(#[case] sort: Vec<&str>, #[case] expected: Ordering) {
        let left = Arc::new(mock_node_1());
        let right = Arc::new(mock_node_2());
        let sort: Vec<String> = sort.into_iter().map(String::from).collect();

        let compare = super::compare(&sort);
        let actual = compare(&left, &right);
        assert_eq!(actual, expected);
    }

    #[rstest::rstest]
    #[case(NodeSearchMatches::new(), true)]
    #[case({
        let mut matches = NodeSearchMatches::new();
        matches.insert("store_id".into(), AttributeMatcher::Eq("test-store".into()));
        matches
    }, true)]
    #[case({
        let mut matches = NodeSearchMatches::new();
        matches.insert("store_id".into(), AttributeMatcher::Eq("test-store".into()));
        matches.insert("test.attribute".into(), AttributeMatcher::Eq("value".into()));
        matches
    }, true)]
    #[case({
        let mut matches = NodeSearchMatches::new();
        matches.insert("store_id".into(), AttributeMatcher::Eq("test-store".into()));
        matches.insert("missing.attribute".into(), AttributeMatcher::Eq(true.into()));
        matches
    }, false)]
    #[case({
        let mut matches = NodeSearchMatches::new();
        let complex = AttributeMatcherComplex {
            op: AttributeMatcherOp::Set,
            value: None,
            values: None,
        };
        matches.insert("store_id".into(), AttributeMatcher::Complex(complex));
        matches
    }, true)]
    #[case({
        let mut matches = NodeSearchMatches::new();
        let complex = AttributeMatcherComplex {
            op: AttributeMatcherOp::Unset,
            value: None,
            values: None,
        };
        matches.insert("store_id".into(), AttributeMatcher::Complex(complex));
        matches
    }, false)]
    fn select_nodes(#[case] matches: NodeSearchMatches, #[case] expected: bool) {
        let node = mock_node_1();
        let select = super::select(&matches);
        let actual = select(&&Arc::new(node));
        assert_eq!(actual, expected);
    }

    #[rstest::rstest]
    #[case("ABC".into(), AttributeMatcher::Eq("ABC".into()))]
    #[case(
        "DEF".into(),
        AttributeMatcher::In(
            vec![
                "ABC".into(),
                "DEF".into(),
            ]
        )
    )]
    #[case(
        serde_json::Number::from(123).into(),
        AttributeMatcher::Complex(
            AttributeMatcherComplex {
                op: AttributeMatcherOp::Eq,
                value: Some(serde_json::Number::from(123).into()),
                values: None,
            }
        )
    )]
    #[case(
        "DEF".into(),
        AttributeMatcher::Complex(
            AttributeMatcherComplex {
                op: AttributeMatcherOp::In,
                value: None,
                values: Some(vec![
                    serde_json::Number::from(123).into(),
                    "DEF".into(),
                ]),
            }
        )
    )]
    #[case(
        serde_json::Number::from(123).into(),
        AttributeMatcher::Complex(
            AttributeMatcherComplex {
                op: AttributeMatcherOp::Ne,
                value: Some(serde_json::Number::from(321).into()),
                values: None,
            }
        )
    )]
    #[case(
        "ABC".into(),
        AttributeMatcher::Complex(
            AttributeMatcherComplex {
                op: AttributeMatcherOp::NotIn,
                value: None,
                values: Some(vec![
                    serde_json::Number::from(123).into(),
                    "DEF".into(),
                ]),
            }
        )
    )]
    fn apply_matcher(#[case] actual: AttributeValue, #[case] matcher: AttributeMatcher) {
        let actual = AttributeValueRef::from(&actual);
        let matched = super::apply_matcher(&matcher, actual);
        assert!(matched);
    }
}
