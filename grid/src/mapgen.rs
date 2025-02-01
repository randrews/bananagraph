use std::collections::HashSet;
use std::ops::Range;
use cgmath::Vector2;
use line_drawing::WalkGrid;
use rand::prelude::{StdRng};
use rand::Rng;
use crate::{Coord, Grid, VecGrid, CountableNeighbors};

pub struct CellularMap {
    size: Vector2<i32>,
    probability: f32,
    born: Range<i32>,
    survive: Range<i32>,
    generations: i32,
    connect: bool
}

impl CellularMap {
    pub fn new(size: impl Into<Vector2<i32>>) -> Self {
        Self {
            size: size.into(),
            probability: 0.5,
            born: 5..9,
            survive: 4..9,
            generations: 5,
            connect: true
        }
    }

    /// How likely should it be that a cell starts as a wall? 0..1
    pub fn with_probability(mut self, probability: f32) -> Self {
        self.probability = probability;
        self
    }

    /// What range of wall-neighbor-count should an empty cell become a wall?
    pub fn with_born(mut self, born: Range<i32>) -> Self {
        self.born = born;
        self
    }

    /// What range of wall-neighbor-count should a wall cell continue as a wall/
    pub fn with_survive(mut self, survive: Range<i32>) -> Self {
        self.survive = survive;
        self
    }

    /// How many generations to run the automata. More generations = smoother cave
    pub fn with_generations(mut self, generations: i32) -> Self {
        self.generations = generations;
        self
    }

    /// Whether or not to dig tunnels so all the non-wall "false" cells connect
    pub fn with_connect(mut self, connect: bool) -> Self {
        self.connect = connect;
        self
    }

    /// Build a cellular-automata random map
    pub fn build(self, rand: &mut StdRng) -> VecGrid<bool> {
        let mut grid = VecGrid::new(self.size, true);

        for pt in grid.size().iter() {
            grid[pt] = rand.gen_ratio((self.probability * 1000.0) as u32, 1000u32);
        }

        for _ in 0..self.generations {
            let old = grid.clone();
            for pt in old.size().iter() {
                let nbrs = (old.neighbors_equal(pt, true).count() +
                    old.diagonals_equal(pt, true).count()) as i32;
                if !old[pt] && self.born.contains(&nbrs) {
                    grid[pt] = true // Born!
                } else if old[pt] && !self.survive.contains(&nbrs) {
                    grid[pt] = false // Dies.
                }
            }
        }

        if self.connect { grid = connect_groups(grid) }

        grid
    }
}

fn bft<T, F: Fn(&T) -> bool>(grid: &impl Grid<CellType=T>, start: impl Into<Vector2<i32>>, traversable: F) -> Vec<Vector2<i32>> {
    let start = start.into();
    let mut open = vec![start];
    let mut visited: Vec<Vector2<i32>> = vec![];
    let mut closed: HashSet<Vector2<i32>> = HashSet::new();

    while !open.is_empty() {
        let curr = open.remove(0);
        closed.insert(curr);
        if traversable(grid.get(curr).unwrap()) {
            visited.push(curr);
            let mut to_add= vec![];
            for nbr in grid.neighbor_coords(curr).filter(|c| !closed.contains(c) && !open.contains(c) && !visited.contains(c)) {

                to_add.push(nbr)
            }
            open.append(&mut to_add);
        }
    }

    visited
}

fn closest_between(group1: &Vec<Vector2<i32>>, group2: &Vec<Vector2<i32>>) -> (Vector2<i32>, Vector2<i32>, i32) {
    let mut min = (group1[0], group2[0], group1[0].manhattan_dist_to(group2[0]));

    for pt_a in group1 {
        for pt_b in group2 {
            let dist = (*pt_a).manhattan_dist_to(*pt_b);
            if dist < min.2 {
                min = (*pt_a, *pt_b, dist);
            }
        }
    }

    min
}

fn shortest_tunnel(groups: &Vec<Vec<Vector2<i32>>>) -> (usize, Vector2<i32>, usize, Vector2<i32>) {
    let mut pts = (Vector2::from((0, 0)), Vector2::from((0, 0)));
    let mut group_nums = (0, 0);
    let mut min_dist = i32::MAX;

    for a in 0..groups.len() {
        for b in 0..groups.len() {
            if a == b { continue }
            if a == group_nums.0 && b == group_nums.1 || a == group_nums.1 && b == group_nums.0 { continue }
            let (pt_a, pt_b, dist) = closest_between(&groups[a], &groups[b]);
            if dist < min_dist {
                pts = (pt_a, pt_b);
                min_dist = dist;
                group_nums = (a, b);
            }
        }
    }

    (group_nums.0, pts.0, group_nums.1, pts.1)
}

fn connect_groups(grid: VecGrid<bool>) -> VecGrid<bool> {
    // 0: wall; -1: unassigned group; 1+: some group
    let mut group_num_grid: VecGrid<i32> = VecGrid::new(grid.size(), 0);

    // First, replace all the empty spaces with -1, signifying unassigned:
    for c in group_num_grid.size().iter() {
        if !grid[c] { group_num_grid[c] = -1 }
    }

    let mut group_num = 1;
    let mut groups: Vec<Vec<Vector2<i32>>> = vec![];

    loop {
        // First, find some unassigned cell:
        if let Some(start) = group_num_grid.find(|c| *c == -1) {
            // Now, fill all the things that it's connected to:
            let group_coords = bft(&group_num_grid, start, |c| *c == -1);
            for g in group_coords.iter() {
                group_num_grid[*g] = group_num
            }
            groups.push(group_coords);
            group_num += 1;
        } else {
            break // All cells assigned, we're done!
        }
    }

    // While more than one group remains:
    while groups.len() > 1 {
        // Find the shortest tunnel that will connect two groups (expensive!)
        let (idx_a, pt_a, idx_b, pt_b) = shortest_tunnel(&groups);
        let tgt = group_num_grid[pt_a]; // we'll turn all of b into a, so dig a tunnel of a
        let old = group_num_grid[pt_b];

        // Draw that tunnel with bresenham
        for lp in WalkGrid::new(pt_a.into(), pt_b.into()) {
            let lp: Vector2<i32> = lp.into();
            group_num_grid[lp] = tgt;
            groups[idx_a].push(lp);
        }

        // Change the cells in the group we just joined
        for pt in group_num_grid.size().iter() {
            if group_num_grid[pt] == old { group_num_grid[pt] = tgt }
        }

        // Append that in our vec of which points are with which
        let mut old_coords = groups[idx_b].clone();
        groups[idx_a].append(&mut old_coords);
        groups.swap_remove(idx_b);
    }

    // Convert this back to a VecGrid<bool> for return
    let mut new_grid = VecGrid::new(grid.size(), false);
    for c in group_num_grid.size().iter() { new_grid[c] = group_num_grid[c] == 0 }
    new_grid
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bft() {
        let grid = VecGrid::from("....\n.++.\n.+..");
        let cs = bft(&grid, (1, 1), |ch| *ch == '+');
        assert!(cs.contains(&Vector2::from((1, 1))));
        assert!(cs.contains(&Vector2::from((2, 1))));
        assert!(cs.contains(&Vector2::from((1, 2))));
        assert_eq!(cs.len(), 3);
    }
}