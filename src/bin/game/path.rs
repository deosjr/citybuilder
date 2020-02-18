use std::collections::BinaryHeap;
use std::collections::{HashMap,HashSet};
use std::cmp::Ordering;

pub trait Map {
    type Node;
    fn neighbours(&self, n: Self::Node) -> Vec<Self::Node>;
    fn g(&self, n: Self::Node, neighbour: Self::Node) -> i64;
    fn h(&self, n: Self::Node, goal: Self::Node) -> i64;
}

#[derive(PartialEq, Eq)]
struct PQItem<N> {
    fscore: i64,
    node: N
}

impl<N> PQItem<N> {
    fn new(node: N, fscore: i64) -> PQItem<N> {
        PQItem{
            fscore: fscore,
            node:   node,
        }
    }
}

impl<N> Ord for PQItem<N> where N: Eq + PartialEq {
    fn cmp(&self, other: &Self) -> Ordering {
        self.fscore.cmp(&other.fscore)
    }
}

impl<N> PartialOrd for PQItem<N> where N: Eq + PartialEq {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other).reverse())
    }
}

pub fn find_route<T,N>(map: T, start: N, goal: N) -> Option<Vec<N>>
    where T: Map<Node=N>, N: Copy + Ord + std::hash::Hash
    {
    let mut open_set = HashSet::new();
    open_set.insert(start);

    let mut came_from: HashMap<N,N> = HashMap::new();

    let mut g_score: HashMap<N,i64> = HashMap::new();
    g_score.insert(start, 0);

    let start_h = map.h(start, goal);
    let mut f_score: HashMap<N,i64> = HashMap::new();
    f_score.insert(start, start_h);

    let mut pq = BinaryHeap::new();
    pq.push(PQItem::new(start, start_h));

    let mut goal_score = std::i64::MAX;

    while !pq.is_empty() {
        let item = pq.pop().unwrap();
        if item.fscore == std::i64::MAX {
            break
        }
        let current = item.node;
        let current_gscore = *(g_score.get(&current).unwrap());
        if current == goal {
            goal_score = current_gscore;
        }
        open_set.remove(&current);

        for n in map.neighbours(current) {
            let tentative_gscore = current_gscore + map.g(current, n);
            let f = tentative_gscore + map.h(n, goal);

            let interesting_f = match f_score.get(&n) {
                None => true,
                Some(&v) => f < v
            };

            if !open_set.contains(&n) && interesting_f && f < goal_score {
                open_set.insert(n);
                pq.push(PQItem::new(n, f));
            } else if tentative_gscore >= *(g_score.get(&n).or(Some(&0)).unwrap()) {
                continue
            }
            came_from.insert(n, current);
            g_score.insert(n, tentative_gscore);
            f_score.insert(n, f);
        }
    }

    if goal_score == std::i64::MAX {
        None
    } else {
        Some(reconstruct_path(came_from, goal))
    }
}

fn reconstruct_path<N>(came_from: HashMap<N,N>, mut current: N) -> Vec<N>
    where N: Eq + std::hash::Hash + Copy
    {
    let mut path = vec![current];
    loop {
        match came_from.get(&current) {
            None => break,
            Some(&prev) => {
                current = prev;
                path.push(current);
            }
        }
    }
    path
}

