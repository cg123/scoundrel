use crate::*;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub type NodeHandle<T> = Rc<RefCell<Node<T>>>;
pub type NodeWeakHandle<T> = Weak<RefCell<Node<T>>>;
pub type HalfSpace = AxialHalfSpace<i32>;

#[derive(Clone)]
pub struct HalfEdge<T> {
    pub line: OrthoLine,
    pub neighbor: NodeWeakHandle<T>,
}
impl<T> HalfEdge<T> {
    pub fn split(&self, half_space: HalfSpace) -> Option<HalfEdge<T>> {
        half_space.clip_line(self.line).map(|new_line| HalfEdge {
            line: new_line,
            neighbor: self.neighbor.clone(),
        })
    }
}

pub struct Node<T> {
    pub bounds: Rect,
    pub parent: Option<NodeWeakHandle<T>>,
    pub children: Option<[NodeHandle<T>; 2]>,
    pub edges: Vec<HalfEdge<T>>,

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
    pub fn with_parent(mut self, parent: NodeWeakHandle<T>) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Sets the edges of the node and returns the updated node.
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
    /// Constructs a new Tree with the specified bounds and root node payload.
    pub fn new(bounds: Rect, root_contents: T) -> Self {
        Self {
            root: Rc::new(RefCell::new(Node::new(bounds, root_contents))),
        }
    }

    /// Splits the tree along the specified half-space, updating the tree structure
    /// and invoking the provided function `f` to generate new contents for the child nodes.
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

        let (above, below) =
            Self::create_children(&handle, &bounds, half_space, contents, &old_edges, &mut f);
        Self::update_neighbor_edges(&above, &below, &bounds, half_space, old_edges, &handle);
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
                        if let Some(edge_below) = half_space.opposite().clip_line(ep.line) {
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
}
