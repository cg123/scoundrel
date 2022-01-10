use crate::{Point, Rect};

pub enum NodePayload<T> {
    Contents(Vec<(T, Point)>),
    // x-y-, x+y-, x-y+, x+y+
    Children(Box<[Node<T>; 4]>),
}
pub struct Node<T> {
    bounds: Rect,
    payload: NodePayload<T>,
}

impl<T> Node<T> {
    fn gt_centroid(&self, query: Point) -> (bool, bool) {
        let twice_centroid = self.bounds.min + self.bounds.max;
        let twice_query = query * 2;
        (twice_query.x > twice_centroid.x, twice_query.y > twice_centroid.y)
    }
    fn child_coord(&self, query: Point) -> (usize, usize) {
        let (x_gt, y_gt) = self.gt_centroid(query);
        (if x_gt { 1 } else { 0 }, if y_gt { 1 } else { 0 })
    }
    fn coord_index(cx: usize, cy: usize) -> usize {
        cy * 2 + cx
    }
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
                let (cx, cy) = self.child_coord(query);
                best = children[Self::coord_index(cx, cy)].nearest(query, best);
                best = children[Self::coord_index(1 - cx, cy)].nearest(query, best);
                best = children[Self::coord_index(cx, 1 - cy)].nearest(query, best);
                best = children[Self::coord_index(1 - cx, 1 - cy)].nearest(query, best);
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
