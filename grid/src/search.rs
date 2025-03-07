use std::collections::{HashMap, HashSet};
use cgmath::Vector2;
use crate::Grid;

/// Do a breadth-first traversal of the grid, finding all cells reachable from a given start point, with reachability
/// defined by a callback passed in.
pub fn bft<T, F: Fn(&T) -> bool>(grid: &impl Grid<CellType=T>, start: impl Into<Vector2<i32>>, traversable: F) -> Vec<Vector2<i32>> {
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

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct UnreachableError{}

/// Perform a breadth-first search of the grid, visiting cells reachable from a given start point, with traversability
/// defined by a callback passed in, until the goal cell is reached. The resulting path should be the shortest, and will
/// start with the start cell and end with the goal cell.
pub fn bfs<T, F: Fn(&T) -> bool>(grid: &impl Grid<CellType=T>, start: impl Into<Vector2<i32>>, goal: impl Into<Vector2<i32>>, diagonals: bool, traversable: F) -> Result<Vec<Vector2<i32>>, UnreachableError> {
    let (start, goal) = (start.into(), goal.into());

    // Degenerate case:
    if start == goal { return Ok(vec![start]) }

    let mut open = vec![start];
    let mut closed: HashSet<Vector2<i32>> = HashSet::new();

    // The backpath gives you the previous cell along the shortest path for a given cell.
    // For a cell c, backpath[c] is the cell you visit right before that cell along the
    // shortest path toward it.
    let mut backpath: HashMap<Vector2<i32>, Vector2<i32>> = HashMap::new();

    while !open.is_empty() {
        let curr = open.remove(0);
        closed.insert(curr);
        if curr == start || traversable(grid.get(curr).unwrap()) {
            let neighbors: Vec<_> = if diagonals {
                grid.adjacent_coords(curr).collect()
            } else {
                grid.neighbor_coords(curr).collect()
            };
            for nbr in neighbors {
                if !closed.contains(&nbr) && !open.contains(&nbr) && !backpath.contains_key(&nbr) {
                    open.push(nbr);
                    backpath.insert(nbr, curr);
                }
            }
            if backpath.contains_key(&goal) { break }
        }
    }

    if !backpath.contains_key(&goal) { return Err(UnreachableError::default()) }

    let mut path = vec![goal];
    let mut curr = goal;

    while curr != start {
        let n = backpath[&curr];
        path.insert(0, n);
        curr = n
    }

    Ok(path)
}

#[cfg(test)]
mod tests {
    use crate::VecGrid;
    use super::*;

    #[test]
    fn test_bfs() {
        let grid = VecGrid::from([
            "######",
            "#  # #",
            "#  # #",
            "#    #",
            "#  # #",
            "######"
        ].join("\n").as_str());

        let path = bfs(&grid, (1, 1), (4, 1), false, |c| *c == ' ').expect("Unreachable");
        assert_eq!(path[0], (1, 1).into());
        assert_eq!(path[1], (2, 1).into());
        assert_eq!(path[2], (2, 2).into());
        assert_eq!(path[3], (2, 3).into());
        assert_eq!(path[4], (3, 3).into());
        assert_eq!(path[5], (4, 3).into());
        assert_eq!(path[6], (4, 2).into());
        assert_eq!(path[7], (4, 1).into());
        assert_eq!(path.len(), 8);
    }

    #[test]
    fn test_bfs_degen() {
        let grid = VecGrid::from([
            "######",
            "#  # #",
            "#  # #",
            "#    #",
            "#  # #",
            "######"
        ].join("\n").as_str());

        let path = bfs(&grid, (1, 1), (1, 1), false, |c| *c == ' ').expect("Unreachable");
        assert_eq!(path[0], (1, 1).into());
        assert_eq!(path.len(), 1);
    }

    #[test]
    fn test_bfs_unreachable() {
        let grid = VecGrid::from([
            "######",
            "#  # #",
            "#  # #",
            "#  # #",
            "#  # #",
            "######"
        ].join("\n").as_str());

        let path = bfs(&grid, (1, 1), (4, 1), false, |c| *c == ' ');
        assert_eq!(path, Err(UnreachableError {}));
    }

    #[test]
    fn test_bfs_diagonal() {
        let grid = VecGrid::from([
            "######",
            "#  # #",
            "#  # #",
            "#    #",
            "#  # #",
            "######"
        ].join("\n").as_str());

        let path = bfs(&grid, (1, 1), (4, 1), true, |c| *c == ' ').expect("Unreachable");
        assert_eq!(path[0], (1, 1).into());
        assert_eq!(path[1], (2, 2).into());
        assert_eq!(path[2], (3, 3).into());
        assert_eq!(path[3], (4, 2).into());
        assert_eq!(path[4], (4, 1).into());
        assert_eq!(path.len(), 5);
    }
}