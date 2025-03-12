use std::cmp::Ordering;

/// A priority queue entry with a value and a priority.
#[derive(Debug, Clone)]
pub struct MinHeapEntry<T, P> {
    pub value: T,
    pub priority: P,
}

impl<T, P: Ord> PartialEq<Self> for MinHeapEntry<T, P> {
    /// Returns `true` if the priorities of `self` and `other` are equal.
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl<T, P: Ord> Eq for MinHeapEntry<T, P> {}

impl<T, P: Ord> PartialOrd<Self> for MinHeapEntry<T, P> {
    /// Compares the priorities of `self` and `other`.
    ///
    /// Returns `Some(Ordering)` if the priorities are comparable, and `None`
    /// otherwise.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // flip the ordering to get a min-heap
        other.priority.partial_cmp(&self.priority)
    }
}

impl<T, P: Ord> Ord for MinHeapEntry<T, P> {
    /// Compares the priorities of `self` and `other`.
    fn cmp(&self, other: &Self) -> Ordering {
        // flip the ordering to get a min-heap
        other.priority.cmp(&self.priority)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BinaryHeap;

    #[test]
    fn test_construction() {
        // Test with integers
        let entry = MinHeapEntry {
            value: "item1",
            priority: 10,
        };
        assert_eq!(entry.value, "item1");
        assert_eq!(entry.priority, 10);

        // Test with custom types
        let entry = MinHeapEntry {
            value: vec![1, 2, 3],
            priority: 'A',
        };
        assert_eq!(entry.value, vec![1, 2, 3]);
        assert_eq!(entry.priority, 'A');
    }

    #[test]
    fn test_clone() {
        let entry1 = MinHeapEntry {
            value: "original",
            priority: 5,
        };

        let entry2 = entry1.clone();

        assert_eq!(entry2.value, "original");
        assert_eq!(entry2.priority, 5);
    }

    #[test]
    fn test_debug_output() {
        let entry = MinHeapEntry {
            value: "test",
            priority: 42,
        };

        let debug_str = format!("{:?}", entry);
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_equality() {
        // Entries with same priority are equal, even with different values
        let entry1 = MinHeapEntry {
            value: "value1",
            priority: 10,
        };

        let entry2 = MinHeapEntry {
            value: "different_value",
            priority: 10,
        };

        let entry3 = MinHeapEntry {
            value: "value1", // Same value as entry1
            priority: 20,    // Different priority
        };

        // Equality is based only on priority
        assert_eq!(entry1, entry2);
        assert_ne!(entry1, entry3);
    }

    #[test]
    fn test_ordering() {
        // Test that higher priority comes before lower priority (min-heap behavior)
        let low_priority = MinHeapEntry {
            value: "low",
            priority: 5,
        };

        let high_priority = MinHeapEntry {
            value: "high",
            priority: 10,
        };

        // For a min-heap, higher priority should be "less than" lower priority
        assert!(high_priority < low_priority);
        assert!(low_priority > high_priority);

        // Test using Ord directly
        assert_eq!(high_priority.cmp(&low_priority), Ordering::Less);
        assert_eq!(low_priority.cmp(&high_priority), Ordering::Greater);

        // Equal priorities
        let equal_priority = MinHeapEntry {
            value: "equal",
            priority: 10, // Same as high_priority
        };

        assert_eq!(high_priority, equal_priority);
        assert_eq!(high_priority.cmp(&equal_priority), Ordering::Equal);
    }

    #[test]
    fn test_in_binary_heap() {
        // Create a binary heap (which is a max-heap)
        // But since we flip the ordering, it will behave like a min-heap
        let mut queue = BinaryHeap::new();

        // Add entries in arbitrary order
        queue.push(MinHeapEntry {
            value: "medium",
            priority: 5,
        });

        queue.push(MinHeapEntry {
            value: "lowest",
            priority: 1,
        });

        queue.push(MinHeapEntry {
            value: "highest",
            priority: 10,
        });

        // Pop entries in priority order
        let first = queue.pop().unwrap();
        assert_eq!(first.value, "lowest");
        assert_eq!(first.priority, 1);

        let second = queue.pop().unwrap();
        assert_eq!(second.value, "medium");
        assert_eq!(second.priority, 5);

        let third = queue.pop().unwrap();
        assert_eq!(third.value, "highest");
        assert_eq!(third.priority, 10);

        assert!(queue.is_empty());
    }

    #[test]
    fn test_with_custom_types() {
        // Define a custom priority type
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
        struct CustomPriority(i32);

        // Create entries with custom priority type
        let entry1 = MinHeapEntry {
            value: "item1",
            priority: CustomPriority(10),
        };

        let entry2 = MinHeapEntry {
            value: "item2",
            priority: CustomPriority(5),
        };

        // Test ordering with custom type
        assert!(entry2 > entry1); // Lower priority comes first

        // Test in binary heap
        let mut queue = BinaryHeap::new();
        queue.push(entry1);
        queue.push(entry2);

        let top = queue.pop().unwrap();
        assert_eq!(top.value, "item2");
        assert_eq!(top.priority, CustomPriority(5));
    }

    #[test]
    fn test_multiple_entries_same_priority() {
        let mut queue = BinaryHeap::new();

        queue.push(MinHeapEntry {
            value: "first",
            priority: 10,
        });

        queue.push(MinHeapEntry {
            value: "second",
            priority: 10, // Same priority
        });

        // Both entries have the same priority, so they are equivalent
        // from the heap's perspective. The order in which they come out
        // is implementation-defined but stable.
        let first_out = queue.pop().unwrap();
        let second_out = queue.pop().unwrap();

        // The values should be either "first" then "second" or vice versa
        assert!(
            (first_out.value == "first" && second_out.value == "second")
                || (first_out.value == "second" && second_out.value == "first")
        );

        // But priorities are definitely the same
        assert_eq!(first_out.priority, 10);
        assert_eq!(second_out.priority, 10);
    }
}
