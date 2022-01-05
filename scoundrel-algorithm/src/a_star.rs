use crate::space::{MapOf, Passability};
use scoundrel_util::PQEntry;
use std::collections::{BinaryHeap, HashMap};

pub fn a_star<M: MapOf<Passability>>(
    map: M,
    start: M::Coordinate,
    end: M::Coordinate,
) -> Option<Vec<M::Coordinate>> {
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

        for candidate in map.neighbors(current) {
            if let Some(Passability::Passable) = map.get(current) {
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
