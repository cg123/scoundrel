use scoundrel_algorithm::{a_star, BaseGraph, LabeledGraph, Passability, SpatialGraph};
use scoundrel_geometry::Vector2;
use scoundrel_util::NonNaN32;

// Integration test grid implementation
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
        // Include diagonal moves for integration test
        let dirs = [
            Vector2::new(0, 1),   // up
            Vector2::new(1, 1),   // up-right
            Vector2::new(1, 0),   // right
            Vector2::new(1, -1),  // down-right
            Vector2::new(0, -1),  // down
            Vector2::new(-1, -1), // down-left
            Vector2::new(-1, 0),  // left
            Vector2::new(-1, 1),  // up-left
        ];

        dirs.iter()
            .map(|dir| node + *dir)
            .filter(|pos| self.in_bounds(*pos))
            .collect()
    }
}

impl LabeledGraph<Passability> for TestGrid {
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
fn test_complex_path_finding() {
    // Create a more complex maze-like grid for integration testing

    let start = Vector2::new(1, 1);
    let end = Vector2::new(3, 3);

    let walls = vec![
        // Outer walls
        Vector2::new(0, 0),
        Vector2::new(1, 0),
        Vector2::new(2, 0),
        Vector2::new(3, 0),
        Vector2::new(4, 0),
        Vector2::new(5, 0),
        Vector2::new(6, 0),
        Vector2::new(0, 6),
        Vector2::new(1, 6),
        Vector2::new(2, 6),
        Vector2::new(3, 6),
        Vector2::new(4, 6),
        Vector2::new(5, 6),
        Vector2::new(6, 6),
        Vector2::new(0, 1),
        Vector2::new(0, 2),
        Vector2::new(0, 3),
        Vector2::new(0, 4),
        Vector2::new(0, 5),
        Vector2::new(6, 1),
        Vector2::new(6, 2),
        Vector2::new(6, 3),
        Vector2::new(6, 4),
        Vector2::new(6, 5),
        // Internal maze structure
        Vector2::new(2, 2),
        Vector2::new(3, 2),
        Vector2::new(4, 2),
        Vector2::new(2, 3),
        Vector2::new(4, 3),
        Vector2::new(2, 4),
        Vector2::new(4, 4),
        Vector2::new(2, 5),
    ];

    let grid = TestGrid::new(7, 7, walls);

    // Test finding a path through the maze

    let path = a_star(&grid, start, end);
    assert!(path.is_some(), "Should find a path through the maze");

    let path_vec = path.unwrap();
    assert_eq!(
        path_vec[0], start,
        "Path should start at the start position"
    );
    assert_eq!(
        path_vec[path_vec.len() - 1],
        end,
        "Path should end at the end position"
    );

    // Verify path doesn't go through walls
    for pos in &path_vec {
        assert!(
            !grid.walls.contains(pos),
            "Path should not go through walls"
        );
    }
}

#[test]
fn test_path_efficiency() {
    // Create an open grid with no walls
    let grid = TestGrid::new(10, 10, vec![]);

    // Test that we get the most efficient path (diagonal in this case)
    let start = Vector2::new(0, 0);
    let end = Vector2::new(9, 9);

    let path = a_star(&grid, start, end).unwrap();

    // The most efficient path with diagonal movement should be a straight diagonal
    // So the length should be around 9-10 steps (not 18 which would be Manhattan distance)
    assert!(
        path.len() <= 10,
        "Path should be efficient with diagonal movement"
    );

    // Check path is actually diagonal (x and y should increase together)
    let mut last_pos = path[0];
    for pos in path.iter().skip(1) {
        assert!(
            pos.x >= last_pos.x,
            "X coordinate should never decrease along optimal path"
        );
        assert!(
            pos.y >= last_pos.y,
            "Y coordinate should never decrease along optimal path"
        );
        last_pos = *pos;
    }
}
