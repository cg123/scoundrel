use scoundrel_algorithm::{Passability, a_star};
use scoundrel_geometry::{Grid2D, Point, Vector2};

fn _make_path_grid(
    width: i32,
    height: i32,
    walls: Vec<Vector2<i32>>,
) -> Grid2D<Passability> {
    // Create a grid filled with passable cells
    let mut grid = Grid2D::new(width, height, Passability::Passable);

    // Mark wall cells as impassable
    for wall in walls {
        // Convert Vector2<i32> to Point for Grid2D compatibility
        let point = Point::new(wall.x, wall.y);
        grid.set(point, Passability::Impassable);
    }

    grid
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

    let grid = _make_path_grid(7, 7, walls);

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
        // Convert Vector2 to Point for Grid2D compatibility
        let point = Point::new(pos.x, pos.y);
        assert_eq!(
            grid.get(point),
            Some(&Passability::Passable),
            "Path should not go through walls"
        );
    }
}

#[test]
fn test_path_efficiency() {
    // Create an open grid with no walls
    let grid = _make_path_grid(10, 10, vec![]);

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
