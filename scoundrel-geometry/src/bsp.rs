use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::*;

/// A reference-counted handle to a BSP tree node.
///
/// This type provides shared ownership of a node with interior mutability
/// through RefCell, allowing the tree structure to be modified while
/// maintaining parent-child relationships.
pub type NodeHandle<T> = Rc<RefCell<Node<T>>>;

/// A weak reference to a BSP tree node.
///
/// Used to prevent reference cycles between parent and child nodes, or
/// between neighboring nodes in the BSP tree.
pub type NodeWeakHandle<T> = Weak<RefCell<Node<T>>>;

/// A half-space with integer coordinates.
///
/// This is a type alias for `AxialHalfSpace<i32>`, representing a plane
/// that divides space into two regions, used for BSP tree partitioning.
pub type HalfSpace = AxialHalfSpace<i32>;

/// Represents an edge between two adjacent nodes in a BSP tree.
///
/// A HalfEdge contains a line segment that forms part of the boundary between
/// two nodes, along with a reference to the neighboring node. The term "half" indicates
/// that each boundary between two nodes is represented by two separate edge objects,
/// one in each node pointing to the other.
#[derive(Clone)]
pub struct HalfEdge<T> {
    /// The line segment that forms this edge.
    pub line: OrthoLine,

    /// A weak reference to the neighboring node connected by this edge.
    pub neighbor: NodeWeakHandle<T>,
}
impl<T> HalfEdge<T> {
    /// Splits this edge along the given half-space.
    ///
    /// When a node is split by a half-space, its edges may also need to be split
    /// if they cross the dividing plane. This method creates a new edge that
    /// represents the portion of this edge that lies within the given half-space.
    ///
    /// # Arguments
    /// * `half_space` - The half-space to split this edge along
    ///
    /// # Returns
    /// * `Some(HalfEdge)` if the edge intersects with the half-space
    /// * `None` if the edge does not intersect with the half-space
    pub fn split(&self, half_space: HalfSpace) -> Option<HalfEdge<T>> {
        half_space.clip_line(self.line).map(|new_line| HalfEdge {
            line: new_line,
            neighbor: self.neighbor.clone(),
        })
    }
}

/// A node in a binary space partitioning tree.
///
/// Each node represents a rectangular region of space that may be subdivided into
/// two child nodes. Nodes maintain connections to their parent, children, and
/// neighboring nodes through edges. Each node also contains a payload of type `T`.
pub struct Node<T> {
    /// The rectangular bounds of this node.
    pub bounds: Rect,

    /// A weak reference to the parent node, if any.
    pub parent: Option<NodeWeakHandle<T>>,

    /// References to the two child nodes, if this node has been split.
    pub children: Option<[NodeHandle<T>; 2]>,

    /// Edges connecting this node to adjacent nodes in the BSP tree.
    pub edges: Vec<HalfEdge<T>>,

    /// The payload data associated with this node.
    pub contents: T,
}

impl<T> Node<T> {
    /// Constructs a new Node with the specified bounds and contents.
    pub fn new(bounds: Rect, contents: T) -> Self {
        Self {
            bounds,
            contents,
            parent: None,
            children: None,
            edges: vec![],
        }
    }

    /// Sets the parent of the node and returns the updated node.
    ///
    /// This method is primarily used during tree construction to establish
    /// parent-child relationships between nodes.
    ///
    /// # Arguments
    /// * `parent` - A weak reference to the parent node
    ///
    /// # Returns
    /// The updated node with the parent reference set
    pub fn with_parent(mut self, parent: NodeWeakHandle<T>) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Sets the edges of the node and returns the updated node.
    ///
    /// This method is used during tree construction to establish connections
    /// between adjacent nodes. The edges define the boundaries shared with
    /// neighboring nodes.
    ///
    /// # Arguments
    /// * `edges` - An iterable of HalfEdge objects to set as this node's edges
    ///
    /// # Returns
    /// The updated node with the edges set
    pub fn with_edges<I: IntoIterator<Item = HalfEdge<T>>>(mut self, edges: I) -> Self {
        self.edges = edges.into_iter().collect();
        self
    }
}

/// A binary space partitioning tree with a payload of type `T` attached to each node.
pub struct Tree<T: Copy> {
    pub root: NodeHandle<T>,
}

impl<T: Copy> Tree<T> {
    /// Constructs a new BSP Tree with the specified bounds and root node payload.
    ///
    /// Creates a single root node with the given bounds and contents.
    /// The tree can then be subdivided by calling the `split` method.
    ///
    /// # Arguments
    /// * `bounds` - The rectangular bounds of the entire space represented by the tree
    /// * `root_contents` - The data payload to store in the root node
    ///
    /// # Returns
    /// A new BSP Tree with a single root node
    pub fn new(bounds: Rect, root_contents: T) -> Self {
        Self {
            root: Rc::new(RefCell::new(Node::new(bounds, root_contents))),
        }
    }

    /// Splits a node in the tree along the specified half-space.
    ///
    /// This method divides the specified node into two child nodes along the given
    /// half-space boundary. It updates all relevant tree structures including:
    /// - Creating two new child nodes
    /// - Updating their bounds based on the half-space
    /// - Establishing parent-child relationships
    /// - Creating and updating edges between nodes
    /// - Setting contents for the new nodes using the provided function
    ///
    /// # Arguments
    /// * `handle` - Reference to the node to split
    /// * `half_space` - The half-space to split the node along
    /// * `f` - A function that generates contents for the new child nodes
    ///
    /// # Returns
    /// `true` if the split was successful, `false` if the half-space doesn't
    /// intersect the node's bounds (meaning no split is possible)
    pub fn split<F: FnMut(&T, Rect) -> T>(
        &mut self,
        handle: NodeHandle<T>,
        half_space: HalfSpace,
        mut f: F,
    ) -> bool {
        let (bounds, contents, old_edges) = {
            let n = handle.borrow();
            (n.bounds, n.contents, n.edges.clone())
        };
        if !half_space.intersects_rect(bounds) {
            return false;
        }

        let (above, below) = Self::create_children(
            &handle, &bounds, half_space, contents, &old_edges, &mut f,
        );
        Self::update_neighbor_edges(
            &above, &below, &bounds, half_space, old_edges, &handle,
        );
        handle.borrow_mut().children = Some([above, below]);
        true
    }

    /// Creates child nodes for the given parent node.
    fn create_children<Func: FnMut(&T, Rect) -> T>(
        handle: &NodeHandle<T>,
        bounds: &Rect,
        half_space: HalfSpace,
        contents: T,
        old_edges: &[HalfEdge<T>],
        f: &mut Func,
    ) -> (NodeHandle<T>, NodeHandle<T>) {
        let mut make_child = |hs: HalfSpace| -> NodeHandle<T> {
            let new_bounds = hs.clip_rect(*bounds).unwrap();
            let new_contents = f(&contents, new_bounds);
            let new_edges = old_edges.iter().filter_map(|edge| edge.split(hs));
            Rc::new(RefCell::new(
                Node::new(new_bounds, new_contents)
                    .with_parent(Rc::downgrade(handle))
                    .with_edges(new_edges),
            ))
        };
        let above = make_child(half_space);
        let below = make_child(half_space.opposite());
        (above, below)
    }

    /// Updates neighbor edges after splitting a node.
    fn update_neighbor_edges(
        above: &NodeHandle<T>,
        below: &NodeHandle<T>,
        bounds: &Rect,
        half_space: HalfSpace,
        old_edges: Vec<HalfEdge<T>>,
        handle: &NodeHandle<T>,
    ) {
        let mut split_start = bounds.min;
        split_start[half_space.axis] = half_space.offset;
        let split_length = (bounds.max - bounds.min)[half_space.axis.opposite()];
        let mut split_line = OrthoLine {
            axis: half_space.axis.opposite(),
            start: split_start,
            length: split_length,
        };

        above.borrow_mut().edges.push(HalfEdge {
            line: split_line,
            neighbor: Rc::downgrade(below),
        });
        split_line.start[half_space.axis] -= 1;
        below.borrow_mut().edges.push(HalfEdge {
            line: split_line,
            neighbor: Rc::downgrade(above),
        });
        for edge in old_edges {
            let neighbor = edge.neighbor.upgrade().unwrap();
            let mut neighbor = neighbor.borrow_mut();
            for idx in 0..neighbor.edges.len() {
                if let Some(neighbor_neighbor) = neighbor.edges[idx].neighbor.upgrade() {
                    if Rc::ptr_eq(&neighbor_neighbor, handle) {
                        let ep = neighbor.edges.remove(idx);
                        if let Some(edge_above) = half_space.clip_line(ep.line) {
                            neighbor.edges.push(HalfEdge {
                                line: edge_above,
                                neighbor: Rc::downgrade(above),
                            })
                        }
                        if let Some(edge_below) = half_space.opposite().clip_line(ep.line)
                        {
                            neighbor.edges.push(HalfEdge {
                                line: edge_below,
                                neighbor: Rc::downgrade(below),
                            })
                        }
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tree() -> Tree<u32> {
        let bounds = Rect::with_points(Point::new(0, 0), Point::new(10, 10));
        Tree::new(bounds, 1)
    }

    #[test]
    fn test_tree_creation() {
        let tree = create_test_tree();
        assert_eq!(
            tree.root.borrow().bounds,
            Rect::with_points(Point::new(0, 0), Point::new(10, 10))
        );
        assert_eq!(tree.root.borrow().contents, 1);
        assert!(tree.root.borrow().parent.is_none());
        assert!(tree.root.borrow().children.is_none());
    }

    #[test]
    fn test_node_split() {
        let mut tree = create_test_tree();
        let half_space = HalfSpace {
            axis: Axis2D::X,
            offset: 5,
            positive: true,
        };
        let root_clone = tree.root.clone();
        let split_result = tree.split(root_clone, half_space, |_, _| 2);

        assert!(split_result);
        let root_node = tree.root.borrow();
        assert!(root_node.children.is_some());
        let children = root_node.children.as_ref().unwrap();
        let (above, below) = (&children[0], &children[1]);

        // Test that child nodes are created correctly
        assert_eq!(
            above.borrow().bounds,
            Rect::with_points(Point::new(5, 0), Point::new(10, 10))
        );
        assert_eq!(above.borrow().contents, 2);
        assert_eq!(
            below.borrow().bounds,
            Rect::with_points(Point::new(0, 0), Point::new(5, 10))
        );
        assert_eq!(below.borrow().contents, 2);

        // Test that child nodes have their parent set correctly
        assert!(Rc::ptr_eq(
            &above.borrow().parent.as_ref().unwrap().upgrade().unwrap(),
            &tree.root
        ));
        assert!(Rc::ptr_eq(
            &below.borrow().parent.as_ref().unwrap().upgrade().unwrap(),
            &tree.root
        ));

        // Test that edges are updated correctly
        assert_eq!(above.borrow().edges.len(), 1);
        assert_eq!(below.borrow().edges.len(), 1);
        assert_eq!(
            above.borrow().edges[0].line,
            OrthoLine {
                axis: Axis2D::Y,
                start: Point::new(5, 0),
                length: 10,
            }
        );
        assert_eq!(
            below.borrow().edges[0].line,
            OrthoLine {
                axis: Axis2D::Y,
                start: Point::new(4, 0),
                length: 10,
            }
        );
    }

    #[test]
    fn test_node_split_no_intersection() {
        let mut tree = create_test_tree();
        let half_space = HalfSpace {
            axis: Axis2D::X,
            offset: 20,
            positive: true,
        };
        let root_clone = tree.root.clone();
        let split_result = tree.split(root_clone, half_space, |_, _| 2);

        assert!(!split_result);
        let root_node = tree.root.borrow();
        assert!(root_node.children.is_none());
    }

    #[test]
    fn test_node_new_and_accessors() {
        let bounds = Rect::with_points(Point::new(0, 0), Point::new(10, 10));
        let node = Node::new(bounds, 42u32);

        // Check initial state
        assert_eq!(node.bounds, bounds);
        assert_eq!(node.contents, 42);
        assert!(node.parent.is_none());
        assert!(node.children.is_none());
        assert!(node.edges.is_empty());
    }

    #[test]
    fn test_node_with_parent() {
        let parent_bounds = Rect::with_points(Point::new(0, 0), Point::new(20, 20));
        let parent = Rc::new(RefCell::new(Node::new(parent_bounds, 1u32)));
        let parent_weak = Rc::downgrade(&parent);

        let bounds = Rect::with_points(Point::new(5, 5), Point::new(15, 15));
        let node = Node::new(bounds, 2u32).with_parent(parent_weak.clone());

        // Check parent reference is set
        assert!(node.parent.is_some());
        assert!(Rc::ptr_eq(
            &node.parent.as_ref().unwrap().upgrade().unwrap(),
            &parent
        ));
    }

    #[test]
    fn test_node_with_edges() {
        let bounds = Rect::with_points(Point::new(0, 0), Point::new(10, 10));
        let neighbor_bounds = Rect::with_points(Point::new(10, 0), Point::new(20, 10));
        let neighbor = Rc::new(RefCell::new(Node::new(neighbor_bounds, 2u32)));

        // Create an edge between nodes
        let edge = HalfEdge {
            line: OrthoLine {
                axis: Axis2D::Y,
                start: Point::new(10, 0),
                length: 10,
            },
            neighbor: Rc::downgrade(&neighbor),
        };

        // Create node with the edge
        let node = Node::new(bounds, 1u32).with_edges(vec![edge]);

        // Check edge is set correctly
        assert_eq!(node.edges.len(), 1);
        assert_eq!(node.edges[0].line.axis, Axis2D::Y);
        assert_eq!(node.edges[0].line.start, Point::new(10, 0));
        assert_eq!(node.edges[0].line.length, 10);
        assert!(Rc::ptr_eq(
            &node.edges[0].neighbor.upgrade().unwrap(),
            &neighbor
        ));
    }

    #[test]
    fn test_half_edge_split() {
        let neighbor_bounds = Rect::with_points(Point::new(10, 0), Point::new(20, 10));
        let neighbor = Rc::new(RefCell::new(Node::new(neighbor_bounds, 2u32)));

        // Create a horizontal edge
        let edge = HalfEdge {
            line: OrthoLine {
                axis: Axis2D::X,
                start: Point::new(0, 5),
                length: 10, // spans x=0 to x=9
            },
            neighbor: Rc::downgrade(&neighbor),
        };

        // Test splitting edge with a vertical half-space at x=3 (positive side)
        let half_space = HalfSpace {
            axis: Axis2D::X,
            offset: 3,
            positive: true,
        };

        let split_edge = edge.split(half_space).unwrap();

        // Check split edge properties
        assert_eq!(split_edge.line.axis, Axis2D::X);
        assert_eq!(split_edge.line.start, Point::new(3, 5)); // Start moved to x=3
        assert_eq!(split_edge.line.length, 7); // Length reduced to 7 (x=3 to x=9)
        assert!(Rc::ptr_eq(
            &split_edge.neighbor.upgrade().unwrap(),
            &neighbor
        ));

        // Test splitting with a half-space that doesn't intersect the edge
        let non_intersecting_hs = HalfSpace {
            axis: Axis2D::X,
            offset: 20,
            positive: true,
        };

        let split_result = edge.split(non_intersecting_hs);
        assert!(split_result.is_none());
    }

    #[test]
    fn test_tree_split_y_axis() {
        // Test splitting on Y axis
        let mut tree = create_test_tree();
        let half_space = HalfSpace {
            axis: Axis2D::Y,
            offset: 5,
            positive: true,
        };

        let root_clone = tree.root.clone();
        let split_result = tree.split(root_clone, half_space, |_, _| 3);

        assert!(split_result);
        let root_node = tree.root.borrow();
        let children = root_node.children.as_ref().unwrap();
        let (above, below) = (&children[0], &children[1]);

        // Check bounds are split correctly on Y axis
        assert_eq!(
            above.borrow().bounds,
            Rect::with_points(Point::new(0, 5), Point::new(10, 10))
        );
        assert_eq!(
            below.borrow().bounds,
            Rect::with_points(Point::new(0, 0), Point::new(10, 5))
        );

        // Check contents are updated correctly
        assert_eq!(above.borrow().contents, 3);
        assert_eq!(below.borrow().contents, 3);
    }

    #[test]
    fn test_nested_tree_splits() {
        // Create a tree and perform multiple splits
        let mut tree = create_test_tree();

        // First split on X axis
        let half_space_x = HalfSpace {
            axis: Axis2D::X,
            offset: 5,
            positive: true,
        };

        let root_clone = tree.root.clone();
        tree.split(root_clone, half_space_x, |_, _| 2);

        // Get the right child (above)
        let right_child = tree.root.borrow().children.as_ref().unwrap()[0].clone();

        // Split the right child on Y axis
        let half_space_y = HalfSpace {
            axis: Axis2D::Y,
            offset: 5,
            positive: true,
        };

        let split_result = tree.split(right_child.clone(), half_space_y, |_, _| 3);
        assert!(split_result);

        // Check that the right child now has children
        let right_node = right_child.borrow();
        assert!(right_node.children.is_some());

        // Get the upper and lower parts of the right child
        let right_children = right_node.children.as_ref().unwrap();
        let (upper_right, lower_right) = (&right_children[0], &right_children[1]);

        // Verify their bounds
        assert_eq!(
            upper_right.borrow().bounds,
            Rect::with_points(Point::new(5, 5), Point::new(10, 10))
        );
        assert_eq!(
            lower_right.borrow().bounds,
            Rect::with_points(Point::new(5, 0), Point::new(10, 5))
        );

        // Check content values propagated correctly
        assert_eq!(upper_right.borrow().contents, 3);
        assert_eq!(lower_right.borrow().contents, 3);
    }

    #[test]
    fn test_content_function() {
        // Test a more complex content generation function
        let mut tree = create_test_tree();

        // Split function that uses both parent content and new bounds
        let half_space = HalfSpace {
            axis: Axis2D::X,
            offset: 5,
            positive: true,
        };

        let root_clone = tree.root.clone();
        let split_result =
            tree.split(root_clone, half_space, |parent_content, bounds| {
                // Generate content based on parent content and area of new bounds
                let area = (bounds.max.x - bounds.min.x) * (bounds.max.y - bounds.min.y);
                parent_content + (area as u32)
            });

        assert!(split_result);
        let root_node = tree.root.borrow();
        let children = root_node.children.as_ref().unwrap();
        let (above, below) = (&children[0], &children[1]);

        // Right side: 1 (parent) + 5*10 (area) = 51
        assert_eq!(above.borrow().contents, 51);

        // Left side: 1 (parent) + 5*10 (area) = 51
        assert_eq!(below.borrow().contents, 51);
    }
}
