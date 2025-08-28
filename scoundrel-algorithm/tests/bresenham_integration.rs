use scoundrel_algorithm::Bresenham;
use scoundrel_geometry::Point;

#[test]
fn test_bresenham_connectivity() {
    // Test that all points along lines are connected without gaps
    let test_cases = [
        (Point::new(0, 0), Point::new(10, 15)),
        (Point::new(-5, -8), Point::new(7, 12)),
        (Point::new(20, 10), Point::new(5, 30)),
    ];

    for (start, end) in test_cases.iter() {
        let line: Vec<Point> = Bresenham::new(*start, *end).collect();

        // Check that the line starts and ends at the correct points
        assert_eq!(&line[0], start);
        assert_eq!(&line[line.len() - 1], end);

        // Verify each point is connected to the next with no diagonal gaps
        for i in 1..line.len() {
            let current = line[i];
            let previous = line[i - 1];

            // Points should be adjacent (using Manhattan distance)
            let manhattan_distance =
                (current.x - previous.x).abs() + (current.y - previous.y).abs();
            assert!(
                manhattan_distance <= 2,
                "Points should be adjacent: {:?} and {:?}, but Manhattan distance is {}",
                previous,
                current,
                manhattan_distance
            );
        }
    }
}

#[test]
fn test_bresenham_approximate_symmetry() {
    // Bresenham lines should be approximately symmetric - reversing start and end should
    // produce similar (but not necessarily identical) lines due to how the algorithm works
    let test_cases = [
        (Point::new(0, 0), Point::new(10, 7)),
        (Point::new(-5, -3), Point::new(5, 8)),
    ];

    for (start, end) in test_cases.iter() {
        let forward: Vec<Point> = Bresenham::new(*start, *end).collect();
        let mut backward: Vec<Point> = Bresenham::new(*end, *start).collect();

        // Reverse the backward line for comparison
        backward.reverse();

        // Both lines should have the same length
        assert_eq!(
            forward.len(),
            backward.len(),
            "Forward and backward lines should have the same length"
        );

        // Start and end points must match exactly
        assert_eq!(forward[0], backward[0], "Starting points should match");
        assert_eq!(
            forward[forward.len() - 1],
            backward[backward.len() - 1],
            "Ending points should match"
        );

        // For non-trivial lines, allow small differences in the middle points
        // Count how many points differ
        let differences = forward
            .iter()
            .zip(backward.iter())
            .filter(|(a, b)| a != b)
            .count();

        // For our test lines, we'll allow at most 1-2 points to be different
        // This is a reasonable tolerance for the Bresenham algorithm
        assert!(
            differences <= 2,
            "Forward and backward lines should be similar, but {} points differ.\nForward: {:?}\nBackward: {:?}",
            differences,
            forward,
            backward
        );

        // Check that any differences are small
        for (i, (a, b)) in forward.iter().zip(backward.iter()).enumerate() {
            // Manhattan distance should be small for any differing points
            let distance = (a.x - b.x).abs() + (a.y - b.y).abs();
            assert!(
                distance <= 2,
                "Point at index {} differs too much: {:?} vs {:?}, Manhattan distance: {}",
                i,
                a,
                b,
                distance
            );
        }
    }
}

#[test]
fn test_bresenham_special_cases() {
    // Test degenerate cases and special configurations

    // Single point
    let point = Point::new(42, 42);
    let line: Vec<Point> = Bresenham::new(point, point).collect();
    assert_eq!(
        line,
        vec![point],
        "Single point line should only contain that point"
    );

    // Orthogonal lines - horizontal
    let horiz_line: Vec<Point> =
        Bresenham::new(Point::new(3, 5), Point::new(8, 5)).collect();
    assert_eq!(
        horiz_line.len(),
        6,
        "Horizontal line should have correct length"
    );
    for point in &horiz_line {
        assert_eq!(point.y, 5, "All points should have the same y coordinate");
    }

    // Orthogonal lines - vertical
    let vert_line: Vec<Point> =
        Bresenham::new(Point::new(3, 5), Point::new(3, 10)).collect();
    assert_eq!(
        vert_line.len(),
        6,
        "Vertical line should have correct length"
    );
    for point in &vert_line {
        assert_eq!(point.x, 3, "All points should have the same x coordinate");
    }
}

#[test]
fn test_bresenham_line_properties() {
    // Test the mathematical properties expected of Bresenham lines

    // 45-degree diagonal should contain exactly the same number of x and y steps
    let diagonal: Vec<Point> =
        Bresenham::new(Point::new(0, 0), Point::new(10, 10)).collect();
    assert_eq!(diagonal.len(), 11, "Diagonal should have correct length");

    // Each step should increment both x and y by 1
    for i in 1..diagonal.len() {
        assert_eq!(
            diagonal[i].x - diagonal[i - 1].x,
            1,
            "X should increment by 1"
        );
        assert_eq!(
            diagonal[i].y - diagonal[i - 1].y,
            1,
            "Y should increment by 1"
        );
    }

    // For a 2:1 slope, we should see 2 y steps for every x step
    let steep_line: Vec<Point> =
        Bresenham::new(Point::new(0, 0), Point::new(5, 10)).collect();

    // Count total y steps
    let mut y_steps = 0;
    for i in 1..steep_line.len() {
        y_steps += (steep_line[i].y - steep_line[i - 1].y).abs();
    }

    assert_eq!(
        y_steps, 10,
        "Line should have correct number of y increments"
    );
}
