use crate::{Point, Rect};

#[derive(Debug)]
pub enum NodePayload<T> {
    Contents(Vec<(T, Point)>),
    // x-y-, x+y-, x-y+, x+y+
    Children(Box<[Node<T>; 4]>),
}

#[derive(Debug)]
pub struct Node<T> {
    bounds: Rect,
    payload: NodePayload<T>,
}

impl<T> Node<T> {
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

pub fn build_quadtree<T>(items: Vec<(T, Point)>, bounds: Rect, max_depth: usize) -> Node<T> {
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
            payload: NodePayload::Contents(vec![(0, Point::new(-3, -3)), (1, Point::new(3, 3))]),
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
        assert!(nearest3 == &(0, Point::new(-3, -3)) || nearest3 == &(1, Point::new(3, 3)));
    }

    #[test]
    fn test_node_query_rect() {
        let node = Node {
            bounds: Rect::with_points(Point::new(0, 0), Point::new(100, 100)),
            payload: NodePayload::Contents(vec![(0, Point::new(50, 50)), (1, Point::new(25, 25))]),
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
            payload: NodePayload::Contents(vec![(0, Point::new(50, 50)), (1, Point::new(25, 25))]),
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
            payload: NodePayload::Contents(vec![(1, Point::new(5, 4)), (2, Point::new(7, 7))]),
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
            payload: NodePayload::Contents(vec![(1, Point::new(1, 1)), (2, Point::new(9, 9))]),
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
        assert_eq!(tree.nearest(Point::new(50, 50), None).unwrap().1 .0, 4); // (60,40) is closest
        assert_eq!(tree.nearest(Point::new(90, 90), None).unwrap().1 .0, 1); // (75,75) is closest
        assert_eq!(tree.nearest(Point::new(100, 10), None).unwrap().1 .0, 5); // (60,10) is closest
    }
}
