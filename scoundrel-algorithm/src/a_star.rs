use crate::graph::LabeledSpatialGraph;
use scoundrel_util::PQEntry;
use std::collections::{BinaryHeap, HashMap};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Passability {
    Passable,
    Impassable,
}

/// Computes the shortest path between two points on a map using the A* algorithm.
///
/// Returns the shortest path as a vector of coordinates if one exists, or `None` otherwise.
///
/// # Arguments
///
/// * `map` - The map to compute the path on.
/// * `start` - The starting coordinate for the path.
/// * `end` - The ending coordinate for the path.
pub fn a_star<M: LabeledSpatialGraph<Passability>>(
    map: &M,
    start: M::NodeHandle,
    end: M::NodeHandle,
) -> Option<Vec<M::NodeHandle>> {
    let mut came_from = HashMap::new();
    let mut running_cost = HashMap::new();
    let mut frontier = BinaryHeap::new();

    running_cost.insert(start, M::Distance::default());
    frontier.push(PQEntry {
        value: start,
        priority: Default::default(),
    });

    while let Some(PQEntry { value: current, .. }) = frontier.pop() {
        if current == end {
            break;
        }

        for candidate in map.adjacent_nodes(current) {
            if let Some(Passability::Passable) = map.get(candidate) {
                let new_cost =
                    *running_cost.get(&current).unwrap() + map.distance(current, candidate);
                if !running_cost.contains_key(&candidate)
                    || *running_cost.get(&candidate).unwrap() > new_cost
                {
                    running_cost.insert(candidate, new_cost);
                    came_from.insert(candidate, current);
                    frontier.push(PQEntry {
                        value: candidate,
                        priority: new_cost + map.distance(candidate, end),
                    });
                }
            }
        }
    }

    if !came_from.contains_key(&end) {
        return None;
    }
    let mut path = vec![end];
    let mut cur = end;
    while let Some(pred) = came_from.get(&cur) {
        cur = *pred;
        path.push(cur);
    }
    path.reverse();
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{BaseGraph, SpatialGraph};
    use scoundrel_geometry::Vector2;
    use scoundrel_util::NonNaN32;

    // Mock implementation of a simple grid for testing
    struct TestGrid {
        width: i32,
        height: i32,
        walls: Vec<Vector2<i32>>,
    }

    impl TestGrid {
        fn new(width: i32, height: i32, walls: Vec<Vector2<i32>>) -> Self {
            Self {
                width,
                height,
                walls,
            }
        }

        fn in_bounds(&self, pos: Vector2<i32>) -> bool {
            pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height
        }
    }

    impl BaseGraph for TestGrid {
        type NodeHandle = Vector2<i32>;

        fn adjacent_nodes(&self, node: Self::NodeHandle) -> Vec<Self::NodeHandle> {
            let dirs = [
                Vector2::new(0, 1),  // up
                Vector2::new(1, 0),  // right
                Vector2::new(0, -1), // down
                Vector2::new(-1, 0), // left
            ];

            dirs.iter()
                .map(|dir| node + *dir)
                .filter(|pos| self.in_bounds(*pos))
                .collect()
        }
    }

    impl crate::LabeledGraph<Passability> for TestGrid {
        fn get(&self, node: Self::NodeHandle) -> Option<Passability> {
            if !self.in_bounds(node) {
                return None;
            }

            if self.walls.contains(&node) {
                Some(Passability::Impassable)
            } else {
                Some(Passability::Passable)
            }
        }
    }

    impl SpatialGraph for TestGrid {
        type Distance = NonNaN32;

        fn distance(&self, from: Self::NodeHandle, to: Self::NodeHandle) -> Self::Distance {
            let dx = (to.x - from.x) as f32;
            let dy = (to.y - from.y) as f32;
            NonNaN32::new((dx * dx + dy * dy).sqrt())
        }
    }

    #[test]
    fn test_a_star_direct_path() {
        // 5x5 grid with no walls
        let grid = TestGrid::new(5, 5, vec![]);

        let start = Vector2::new(0, 0);
        let end = Vector2::new(4, 4);

        let path = a_star(&grid, start, end);
        assert!(path.is_some());

        let path = path.unwrap();
        assert_eq!(path[0], start);
        assert_eq!(path[path.len() - 1], end);
    }

    #[test]
    fn test_a_star_with_wall() {
        // 5x5 grid with a wall in the middle
        let walls = vec![
            Vector2::new(2, 0),
            Vector2::new(2, 1),
            Vector2::new(2, 2),
            Vector2::new(2, 3),
        ];
        let grid = TestGrid::new(5, 5, walls);

        let start = Vector2::new(0, 2);
        let end = Vector2::new(4, 2);

        let path = a_star(&grid, start, end);
        assert!(path.is_some());

        // Path should go around the wall
        let path = path.unwrap();
        assert_eq!(path[0], start);
        assert_eq!(path[path.len() - 1], end);

        // Check that the path avoids the wall
        for pos in &path {
            assert!(!grid.walls.contains(pos));
        }
    }

    #[test]
    fn test_a_star_no_path() {
        // 5x5 grid with a wall completely blocking the path
        let walls = vec![
            Vector2::new(2, 0),
            Vector2::new(2, 1),
            Vector2::new(2, 2),
            Vector2::new(2, 3),
            Vector2::new(2, 4),
        ];
        let grid = TestGrid::new(5, 5, walls);

        let start = Vector2::new(0, 2);
        let end = Vector2::new(4, 2);

        let path = a_star(&grid, start, end);
        assert!(path.is_none());
    }
}
