use crate::{Point, Rect};

/// The payload of a quadtree node, which can be either leaf contents or child nodes.
///
/// A quadtree node is either a leaf node containing a list of items, or an internal
/// node with four children corresponding to the four quadrants of the node's bounds.
#[derive(Debug)]
pub enum NodePayload<T> {
    /// A leaf node containing a list of items with their positions.
    Contents(Vec<(T, Point)>),

    /// An internal node with four children in the following order:
    /// - 0: upper-right quadrant (x+, y+)
    /// - 1: upper-left quadrant (x-, y+)  
    /// - 2: lower-left quadrant (x-, y-)
    /// - 3: lower-right quadrant (x+, y-)
    Children(Box<[Node<T>; 4]>),
}

/// A node in a quadtree spatial data structure.
///
/// Each node represents a rectangular region of space and either contains
/// items directly (leaf node) or has been subdivided into four child nodes
/// (internal node). This structure enables efficient spatial queries.
#[derive(Debug)]
pub struct Node<T> {
    /// The rectangular bounds of the space this node represents.
    bounds: Rect,

    /// The payload of this node (either items or child nodes).
    payload: NodePayload<T>,
}

impl<T> Node<T> {
    /// Finds the item nearest to the query point in this node's subtree.
    ///
    /// This method implements a spatial nearest-neighbor search. It uses the
    /// quadtree structure to efficiently prune the search space, checking
    /// nearby quadrants first and avoiding quadrants that cannot contain a
    /// better match than the current best.
    ///
    /// # Arguments
    /// * `query` - The point to find the nearest item to
    /// * `best` - The current best match, if any (used for recursive calls)
    ///
    /// # Returns
    /// An option containing a tuple of (squared distance, reference to item)
    /// for the nearest item found, or None if the node contains no items.
    pub fn nearest<'a>(
        &'a self,
        query: Point,
        mut best: Option<(i32, &'a (T, Point))>,
    ) -> Option<(i32, &'a (T, Point))> {
        let closest_possible = self.bounds.closest_pt(query);
        if best.map_or(false, |b| b.0 < (closest_possible - query).sqr_magnitude()) {
            // if best current candidate is closer than anything inside our bounds, early exit
            return best;
        }

        match &self.payload {
            NodePayload::Contents(items) => {
                for item in items {
                    let sqr_dist = (item.1 - query).sqr_magnitude();
                    if best.map_or(true, |b| b.0 > sqr_dist) {
                        best = Some((sqr_dist, item));
                    }
                }
            }
            NodePayload::Children(children) => {
                let quadrant = self.bounds.containing_quadrant_idx(query);
                best = children[quadrant].nearest(query, best);
                best = children[(quadrant + 1) % 4].nearest(query, best);
                best = children[(quadrant + 2) % 4].nearest(query, best);
                best = children[(quadrant + 3) % 4].nearest(query, best);
            }
        }
        best
    }

    /// Finds all items contained within the specified rectangular region.
    ///
    /// This method efficiently queries the quadtree to find all items whose
    /// positions are inside the given rectangle. It uses the tree structure
    /// to quickly prune branches that don't intersect with the query region.
    ///
    /// # Arguments
    /// * `rect` - The rectangular region to query
    /// * `f` - A callback function that will be invoked for each item found
    ///
    /// # Example
    /// ```
    /// # use scoundrel_geometry::{quadtree, Rect, Point};
    /// # let items = vec![(1, Point::new(5, 5)), (2, Point::new(15, 15))];
    /// # let bounds = Rect::with_points(Point::new(0, 0), Point::new(20, 20));
    /// # let tree = quadtree::build_quadtree(items, bounds, 2);
    /// let mut results = Vec::new();
    /// tree.query_rect(
    ///     Rect::with_points(Point::new(0, 0), Point::new(10, 10)),
    ///     &mut |&(id, _)| { results.push(id); }
    /// );
    /// assert_eq!(results, vec![1]); // Only the item at (5,5) is found
    /// ```
    pub fn query_rect<F: FnMut(&(T, Point))>(&self, rect: Rect, f: &mut F) {
        if !self.bounds.intersects(&rect) {
            return;
        }

        match &self.payload {
            NodePayload::Contents(contents) => {
                for item in contents {
                    if rect.contains(item.1) {
                        f(item)
                    }
                }
            }
            NodePayload::Children(children) => {
                for child in &children[..] {
                    child.query_rect(rect, f);
                }
            }
        }
    }
}

/// Builds a quadtree from a collection of items with associated positions.
///
/// This function recursively constructs a quadtree by dividing space into quadrants
/// and distributing items among those quadrants. The process continues until either:
/// - A node contains 0 or 1 items
/// - The maximum depth limit is reached
/// - All items in a node are at the same position
///
/// # Arguments
/// * `items` - A vector of items with their associated 2D positions
/// * `bounds` - The rectangular bounds of the entire space
/// * `max_depth` - The maximum recursion depth for tree construction
///
/// # Returns
/// A quadtree node representing the root of the constructed tree
///
/// # Example
/// ```
/// use scoundrel_geometry::{quadtree, Point, Rect};
///
/// // Create some items with positions
/// let items = vec![(1, Point::new(1, 1)), (2, Point::new(8, 8))];
///
/// // Build a quadtree with the items
/// let tree = quadtree::build_quadtree(
///     items,
///     Rect::with_points(Point::new(0, 0), Point::new(10, 10)),
///     2
/// );
///
/// // Use the tree for spatial queries
/// ```
pub fn build_quadtree<T>(
    items: Vec<(T, Point)>,
    bounds: Rect,
    max_depth: usize,
) -> Node<T> {
    if items.len() <= 1 || max_depth == 0 {
        return Node {
            bounds,
            payload: NodePayload::Contents(items),
        };
    }

    let mut quadrant_contents = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    for item in items {
        let idx = bounds.containing_quadrant_idx(item.1);
        quadrant_contents[idx].push(item);
    }

    let mut children = Vec::with_capacity(4);
    for (i, contents) in quadrant_contents.into_iter().enumerate() {
        let child_node = build_quadtree(contents, bounds.quadrant(i), max_depth - 1);
        children.push(child_node);
    }

    let bs = children.into_boxed_slice();
    let ti = bs.try_into();
    let children = ti.map_err(|_| "whoops").unwrap();
    Node {
        bounds,
        payload: NodePayload::Children(children), //Box::new(children.into_iter().collect())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_nearest() {
        let node = Node {
            bounds: Rect::with_points(Point::new(-5, -5), Point::new(5, 5)),
            payload: NodePayload::Contents(vec![
                (0, Point::new(-3, -3)),
                (1, Point::new(3, 3)),
            ]),
        };

        // Test nearest to a point that is closer to the point at (-3, -3)
        let query1 = Point::new(-4, -4);
        let nearest1 = node.nearest(query1, None).unwrap().1;
        assert_eq!(nearest1, &(0, Point::new(-3, -3)));

        // Test nearest to a point that is closer to the point at (3, 3)
        let query2 = Point::new(4, 4);
        let nearest2 = node.nearest(query2, None).unwrap().1;
        assert_eq!(nearest2, &(1, Point::new(3, 3)));

        // Test nearest to a point that is equidistant to both points
        let query3 = Point::new(0, 0);
        let nearest3 = node.nearest(query3, None).unwrap().1;
        assert!(
            nearest3 == &(0, Point::new(-3, -3)) || nearest3 == &(1, Point::new(3, 3))
        );
    }

    #[test]
    fn test_node_query_rect() {
        let node = Node {
            bounds: Rect::with_points(Point::new(0, 0), Point::new(100, 100)),
            payload: NodePayload::Contents(vec![
                (0, Point::new(50, 50)),
                (1, Point::new(25, 25)),
            ]),
        };

        let mut results = Vec::new();
        node.query_rect(
            Rect::with_points(Point::new(0, 0), Point::new(40, 40)),
            &mut |&(id, _)| {
                results.push(id);
            },
        );
        assert_eq!(results, vec![1]);

        results.clear();
        node.query_rect(
            Rect::with_points(Point::new(30, 30), Point::new(70, 70)),
            &mut |&(id, _)| {
                results.push(id);
            },
        );
        assert_eq!(results, vec![0]);
    }

    #[test]
    fn test_node_query_rect_empty() {
        let node = Node {
            bounds: Rect::with_points(Point::new(0, 0), Point::new(100, 100)),
            payload: NodePayload::Contents(vec![
                (0, Point::new(50, 50)),
                (1, Point::new(25, 25)),
            ]),
        };

        let mut results = Vec::new();
        node.query_rect(
            Rect::with_points(Point::new(200, 200), Point::new(300, 300)),
            &mut |&(id, _)| {
                results.push(id);
            },
        );
        assert_eq!(results, vec![]);
    }

    #[test]
    fn test_node_query_rect_nested() {
        let node = Node {
            bounds: Rect::with_points(Point::new(0, 0), Point::new(100, 100)),
            payload: NodePayload::Children(Box::new([
                Node {
                    bounds: Rect::with_points(Point::new(50, 50), Point::new(100, 100)),
                    payload: NodePayload::Contents(vec![(3, Point::new(74, 74))]),
                },
                Node {
                    bounds: Rect::with_points(Point::new(0, 50), Point::new(50, 100)),
                    payload: NodePayload::Contents(vec![(2, Point::new(25, 74))]),
                },
                Node {
                    bounds: Rect::with_points(Point::new(0, 0), Point::new(50, 50)),
                    payload: NodePayload::Contents(vec![(0, Point::new(25, 25))]),
                },
                Node {
                    bounds: Rect::with_points(Point::new(50, 0), Point::new(100, 50)),
                    payload: NodePayload::Contents(vec![(1, Point::new(74, 25))]),
                },
            ])),
        };

        let mut results = Vec::new();
        node.query_rect(
            Rect::with_points(Point::new(25, 25), Point::new(75, 75)),
            &mut |&(id, _)| {
                results.push(id);
            },
        );
        results.sort();
        assert_eq!(results, vec![0, 1, 2, 3]);

        results.clear();
        node.query_rect(
            Rect::with_points(Point::new(0, 0), Point::new(26, 26)),
            &mut |&(id, _)| {
                results.push(id);
            },
        );
        assert_eq!(results, vec![0]);
    }

    #[test]
    fn test_build_quadtree() {
        let items = vec![
            (1, Point { x: 0, y: 0 }),
            (2, Point { x: 2, y: 2 }),
            (3, Point { x: 4, y: 4 }),
            (4, Point { x: 6, y: 6 }),
        ];
        let bounds = Rect {
            min: Point { x: 0, y: 0 },
            max: Point { x: 8, y: 8 },
        };
        let max_depth = 2;

        let quadtree = build_quadtree(items, bounds, max_depth);

        assert_eq!(
            quadtree.bounds,
            Rect {
                min: Point { x: 0, y: 0 },
                max: Point { x: 8, y: 8 },
            }
        );

        if let NodePayload::Children(children) = &quadtree.payload {
            assert_eq!(
                children[0].bounds,
                Rect {
                    min: Point { x: 4, y: 4 },
                    max: Point { x: 8, y: 8 },
                }
            );
            if let NodePayload::Contents(contents) = &children[0].payload {
                assert_eq!(
                    contents,
                    &[(3, Point { x: 4, y: 4 }), (4, Point { x: 6, y: 6 })]
                );
            }

            assert_eq!(
                children[1].bounds,
                Rect {
                    min: Point { x: 0, y: 4 },
                    max: Point { x: 4, y: 8 },
                }
            );
            if let NodePayload::Contents(contents) = &children[1].payload {
                assert_eq!(contents, &[]);
            }

            assert_eq!(
                children[2].bounds,
                Rect {
                    min: Point { x: 0, y: 0 },
                    max: Point { x: 4, y: 4 },
                }
            );
            if let NodePayload::Contents(contents) = &children[2].payload {
                assert_eq!(
                    contents,
                    &[(1, Point { x: 0, y: 0 }), (2, Point { x: 2, y: 2 })]
                );
            }

            assert_eq!(
                children[3].bounds,
                Rect {
                    min: Point { x: 4, y: 0 },
                    max: Point { x: 8, y: 4 },
                }
            );
            if let NodePayload::Contents(contents) = &children[3].payload {
                assert_eq!(contents, &[]);
            }
        } else {
            panic!("expected quadtree with children");
        }
    }

    #[test]
    fn test_empty_quadtree() {
        // Test building an empty quadtree
        let items: Vec<(i32, Point)> = vec![];
        let bounds = Rect::with_points(Point::new(0, 0), Point::new(10, 10));
        let tree = build_quadtree(items, bounds, 3);

        // Check it's a leaf node with no contents
        if let NodePayload::Contents(contents) = &tree.payload {
            assert!(contents.is_empty());
        } else {
            panic!("Expected empty quadtree to be a leaf node");
        }

        // Test queries on empty tree
        let mut results = Vec::new();
        tree.query_rect(
            Rect::with_points(Point::new(0, 0), Point::new(10, 10)),
            &mut |&(id, _)| {
                results.push(id);
            },
        );
        assert!(results.is_empty());

        // Test nearest on empty tree
        let nearest = tree.nearest(Point::new(5, 5), None);
        assert!(nearest.is_none());
    }

    #[test]
    fn test_build_quadtree_max_depth_zero() {
        // With max_depth = 0, should create a leaf node regardless of number of items
        let items = vec![
            (1, Point::new(1, 1)),
            (2, Point::new(5, 5)),
            (3, Point::new(8, 8)),
        ];
        let bounds = Rect::with_points(Point::new(0, 0), Point::new(10, 10));
        let tree = build_quadtree(items.clone(), bounds, 0);

        // Check it's a leaf node containing all items
        if let NodePayload::Contents(contents) = &tree.payload {
            assert_eq!(contents.len(), 3);
            assert!(contents.contains(&(1, Point::new(1, 1))));
            assert!(contents.contains(&(2, Point::new(5, 5))));
            assert!(contents.contains(&(3, Point::new(8, 8))));
        } else {
            panic!("Expected tree with max_depth=0 to be a leaf node");
        }
    }

    #[test]
    fn test_all_points_in_same_quadrant() {
        // All points in bottom-left quadrant
        let items = vec![
            (1, Point::new(1, 1)),
            (2, Point::new(2, 2)),
            (3, Point::new(3, 3)),
        ];
        let bounds = Rect::with_points(Point::new(0, 0), Point::new(10, 10));
        let tree = build_quadtree(items, bounds, 1);

        // Should have children, but only one populated
        if let NodePayload::Children(children) = &tree.payload {
            // Check bottom-left quadrant (index 2)
            if let NodePayload::Contents(contents) = &children[2].payload {
                assert_eq!(contents.len(), 3);
            } else {
                panic!("Expected bottom-left child to be a leaf node");
            }

            // Check other quadrants are empty
            for i in [0, 1, 3] {
                if let NodePayload::Contents(contents) = &children[i].payload {
                    assert!(contents.is_empty());
                } else {
                    panic!("Expected empty quadrant to be a leaf node");
                }
            }
        } else {
            panic!("Expected tree to have children");
        }
    }

    #[test]
    fn test_nearest_with_priority() {
        // Test nearest when a best candidate is already provided
        let node = Node {
            bounds: Rect::with_points(Point::new(0, 0), Point::new(10, 10)),
            payload: NodePayload::Contents(vec![
                (1, Point::new(5, 4)),
                (2, Point::new(7, 7)),
            ]),
        };

        // Query point is at (6,6)
        let query = Point::new(6, 6);

        // Without existing best candidate, (7,7) should be closest
        let nearest = node.nearest(query, None).unwrap();
        assert_eq!(nearest.1, &(2, Point::new(7, 7)));

        // With existing best candidate very close to query point, should keep that one
        let best_candidate = (0, &(0, Point::new(6, 6))); // Distance = 0
        let nearest_with_best = node.nearest(query, Some(best_candidate)).unwrap();
        assert_eq!(nearest_with_best.1, &(0, Point::new(6, 6)));
    }

    #[test]
    fn test_nearest_point_outside_bounds() {
        // Test with query point outside the bounds
        let node = Node {
            bounds: Rect::with_points(Point::new(0, 0), Point::new(10, 10)),
            payload: NodePayload::Contents(vec![
                (1, Point::new(1, 1)),
                (2, Point::new(9, 9)),
            ]),
        };

        // Query point outside bounds
        let query = Point::new(15, 15);

        // Should still find (9,9) as closest
        let nearest = node.nearest(query, None).unwrap();
        assert_eq!(nearest.1, &(2, Point::new(9, 9)));
    }

    #[test]
    fn test_nested_nearest() {
        // More complex tree with multiple levels for nearest search
        let tree = build_quadtree(
            vec![
                (0, Point::new(25, 25)),
                (1, Point::new(75, 75)),
                (2, Point::new(10, 90)),
                (3, Point::new(25, 75)),
                (4, Point::new(60, 40)),
                (5, Point::new(60, 10)),
            ],
            Rect::with_points(Point::new(0, 0), Point::new(100, 100)),
            2,
        );

        // Test queries in different areas
        // Query in center
        assert_eq!(tree.nearest(Point::new(50, 50), None).unwrap().1.0, 4); // (60,40) is closest
        assert_eq!(tree.nearest(Point::new(90, 90), None).unwrap().1.0, 1); // (75,75) is closest
        assert_eq!(tree.nearest(Point::new(100, 10), None).unwrap().1.0, 5); // (60,10) is closest
    }
}
