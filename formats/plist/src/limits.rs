// WHAT: the resource limits of the property-list decoder.
// WHY: the threat model of this project is "any device on the local
//      network". A property list arrives over the wire, so every length
//      the peer announces must be checked against a budget *before* the
//      corresponding memory is reserved. The budgets live here, in one
//      place, so that an engine can tighten them from `AppConfig::limits`.
// EXAMPLE: `PlistLimits::default().max_depth` is 64, which rejects a
//          hand-crafted plist that nests dictionaries into each other.

/// The budgets a property-list decoder enforces.
///
/// The defaults are generous enough for every legitimate AirPlay and Cast
/// payload and small enough that a hostile peer cannot exhaust the
/// memory of a receiver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlistLimits {
    /// The maximum nesting depth of containers.
    pub max_depth: usize,
    /// The maximum number of objects in the binary object table.
    pub max_objects: usize,
    /// The maximum number of elements in a single array or dictionary.
    pub max_container_len: usize,
    /// The maximum length of a single string, in bytes.
    pub max_string_len: usize,
    /// The maximum length of a single data blob, in bytes.
    pub max_data_len: usize,
    /// The maximum number of value nodes after shared references are expanded.
    pub max_expanded_nodes: usize,
}

impl Default for PlistLimits {
    fn default() -> Self {
        Self {
            max_depth: 64,
            max_objects: 100_000,
            max_container_len: 1_000_000,
            max_string_len: 4 * 1024 * 1024,
            max_data_len: 16 * 1024 * 1024,
            // WHY: the decoder keeps every decoded object in a cache,
            //      so the budget of the expansion is also the budget of
            //      the memory. One million nodes are far more than a
            //      legitimate property list needs, and they stay well
            //      inside the memory of a small receiver.
            max_expanded_nodes: 1_000_000,
        }
    }
}

impl PlistLimits {
    /// Returns a copy of the limits with every budget reduced to `factor`.
    ///
    /// WHY: a receiver that runs on a small embedded device may want a
    ///      tighter budget than the default without re-typing the whole
    ///      structure.
    ///
    /// EXAMPLE: `PlistLimits::default().scaled_by(2)` doubles every budget.
    pub fn scaled_by(&self, factor: usize) -> Self {
        let factor = factor.max(1);
        Self {
            max_depth: self.max_depth,
            max_objects: self.max_objects.saturating_mul(factor),
            max_container_len: self.max_container_len.saturating_mul(factor),
            max_string_len: self.max_string_len.saturating_mul(factor),
            max_data_len: self.max_data_len.saturating_mul(factor),
            max_expanded_nodes: self.max_expanded_nodes.saturating_mul(factor),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_finite_and_non_zero() {
        let limits = PlistLimits::default();
        assert_eq!(limits.max_depth, 64);
        assert!(limits.max_objects > 0);
        assert!(limits.max_container_len > 0);
        assert!(limits.max_string_len > 0);
        assert!(limits.max_data_len > 0);
        assert!(limits.max_expanded_nodes > 0);
    }

    #[test]
    fn scaled_by_multiplies_every_budget() {
        let limits = PlistLimits::default();
        let doubled = limits.scaled_by(2);
        assert_eq!(doubled.max_objects, limits.max_objects * 2);
        assert_eq!(doubled.max_data_len, limits.max_data_len * 2);
        assert_eq!(doubled.max_depth, limits.max_depth);
    }

    #[test]
    fn scaled_by_zero_does_not_shrink_the_budgets() {
        let limits = PlistLimits::default();
        let scaled = limits.scaled_by(0);
        assert_eq!(scaled.max_objects, limits.max_objects);
    }
}
