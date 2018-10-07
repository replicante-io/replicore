use replicante_data_models::Event;

use super::Result;


/// Private interface to the events streaming system.
///
/// Allows multiple possible backends to be used as well as mocks for testing.
pub trait StreamInterface: Send + Sync {
    /// Emit events to the events stream.
    fn emit(&self, event: Event) -> Result<()>;

    /// Scan for events matching the given filters, old to new.
    fn scan(&self, filters: ScanFilters, options: ScanOptions) -> Result<Iter>;
}


/// Iterator over events returned by a scan operation.
pub struct Iter(Box<Iterator<Item=Result<Event>>>);

impl Iter {
    pub fn new<I>(iter: I) -> Iter
        where I: Iterator<Item=Result<Event>> + 'static
    {
        Iter(Box::new(iter))
    }
}

impl Iterator for Iter {
    type Item = Result<Event>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}


/// Filters to apply when scanning events.
pub struct ScanFilters {}

impl ScanFilters {
    /// Return all events, don't skip any.
    pub fn all() -> ScanFilters {
        Self::default()
    }
}

impl Default for ScanFilters {
    fn default() -> ScanFilters {
        ScanFilters { }
    }
}


/// Options to apply when scanning events.
pub struct ScanOptions {
    /// Max number of events to return.
    pub limit: Option<i64>,

    /// By default events are returned old to new, set to true to reverse the order.
    pub reverse: bool,
}

impl Default for ScanOptions {
    fn default() -> ScanOptions {
        ScanOptions {
            limit: None,
            reverse: false,
        }
    }
}
