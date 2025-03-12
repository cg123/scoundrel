use crate::graph::LabeledSpatialGraph;
use scoundrel_util::PQEntry;
use std::collections::{BinaryHeap, HashMap};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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
    use scoundrel_geometry::{Grid2D, Vector2};

    #[test]
    fn test_a_star_direct_path() {
        // 5x5 grid with no walls
        let grid = Grid2D::from_sparse_points(
            5,
            5,
            Passability::Passable,
            vec![],
            Passability::Impassable,
        );

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
        let grid =
            Grid2D::from_sparse_points(5, 5, Passability::Passable, walls, Passability::Impassable);

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
            assert!(grid.get(*pos) == Some(&Passability::Passable));
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
        let grid =
            Grid2D::from_sparse_points(5, 5, Passability::Passable, walls, Passability::Impassable);

        let start = Vector2::new(0, 2);
        let end = Vector2::new(4, 2);

        let path = a_star(&grid, start, end);
        assert!(path.is_none());
    }
}
