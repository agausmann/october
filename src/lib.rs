use std::collections::HashMap;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl Index {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }

    fn bit(&self, height: u32) -> (usize, usize, usize) {
        (
            (self.x as usize >> height) & 1,
            (self.y as usize >> height) & 1,
            (self.z as usize >> height) & 1,
        )
    }

    fn branch_at(&self, height: u32) -> BranchIndex {
        let mask = !((1 << height) - 1);
        BranchIndex {
            base: Index {
                x: self.x & mask,
                y: self.y & mask,
                z: self.z & mask,
            },
            height,
        }
    }
}

impl From<(u32, u32, u32)> for Index {
    fn from((x, y, z): (u32, u32, u32)) -> Self {
        Self { x, y, z }
    }
}

impl From<[u32; 3]> for Index {
    fn from([x, y, z]: [u32; 3]) -> Self {
        Self { x, y, z }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct BranchIndex {
    base: Index,
    height: u32,
}

impl BranchIndex {
    fn root(height: u32) -> Self {
        Self {
            base: Index { x: 0, y: 0, z: 0 },
            height,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum RawNode {
    False,
    True,
    Branch,
}

impl From<bool> for RawNode {
    fn from(x: bool) -> Self {
        match x {
            false => Self::False,
            true => Self::True,
        }
    }
}

struct Branch {
    children: [[[RawNode; 2]; 2]; 2],
}

/// A three-dimensional bitmap, implemented as an octree.
pub struct OctreeBitmap {
    branches: HashMap<BranchIndex, Branch>,
    height: u32,
}

impl OctreeBitmap {
    /// Creates a new, empty bitmap.
    ///
    /// The indexes allowed in the set are limited to a certain range, specified
    /// by the `width` parameter; the values of indexes on each dimension must
    /// be within the range `0..width`.
    pub fn new(width: u32) -> Self {
        // ceil(log2(width))
        let height = u32::BITS - width.next_power_of_two().leading_zeros();
        let mut nodes = HashMap::new();
        nodes.insert(
            BranchIndex::root(height),
            Branch {
                children: [[[RawNode::False; 2]; 2]; 2],
            },
        );
        Self {
            branches: nodes,
            height,
        }
    }

    /// Clears the map.
    ///
    /// After this is called, [`get`] will return `false` for all indexes.
    pub fn clear(&mut self) {
        self.branches.clear();
        self.branches.insert(
            BranchIndex::root(self.height),
            Branch {
                children: [[[RawNode::False; 2]; 2]; 2],
            },
        );
    }

    /// The width of the map. Index values in each dimension must be within the
    /// range `0..map.width()`.
    ///
    /// If the map is constructed with [`new`], this is guaranteed to be greater
    /// than or equal to the specified value of `width`. In the current
    /// implementation, it is rounded up to the next power of two less than
    /// or equal the specified width.
    pub fn width(&self) -> u32 {
        1 << self.height
    }

    /// Get the current value of the bit at the given index.
    pub fn get(&self, idx: &Index) -> bool {
        let mut current_height = self.height;
        loop {
            let current_branch = &self.branches[&idx.branch_at(current_height)];
            let (x, y, z) = idx.bit(current_height - 1);
            match current_branch.children[z][y][x] {
                RawNode::False => return false,
                RawNode::True => return true,
                RawNode::Branch => {
                    current_height -= 1;
                    if current_height == 0 {
                        unreachable!("branch node at height zero");
                    }
                }
            }
        }
    }

    /// Set the value at the given index.
    pub fn set(&mut self, idx: &Index, value: bool) {
        let desired_state = RawNode::from(value);
        let mut current_height = self.height;
        loop {
            let current_index = idx.branch_at(current_height);
            let current_branch = self.branches.get_mut(&current_index).unwrap();
            let (x, y, z) = idx.bit(current_height - 1);
            match current_branch.children[z][y][x] {
                RawNode::Branch => {
                    current_height -= 1;
                    if current_height == 0 {
                        unreachable!("branch node at height zero");
                    }
                }
                other => {
                    if desired_state == other {
                        // Already
                        return;
                    } else if current_height == 1 {
                        current_branch.children[z][y][x] = desired_state;
                        self.compress(idx, desired_state);
                        return;
                    } else {
                        current_branch.children[z][y][x] = RawNode::Branch;
                        self.branches.insert(
                            idx.branch_at(current_height - 1),
                            Branch {
                                children: [[[other; 2]; 2]; 2],
                            },
                        );
                    }
                }
            }
        }
    }

    /// Traverse the tree from the specified leaf to the root, replacing all
    /// branches that have uniform child values with a single node of that
    /// value.
    fn compress(&mut self, idx: &Index, state: RawNode) {
        // Root node (==self.height) is intentionally excluded as it is always
        // a branch node.
        for current_height in 1..self.height {
            let current_index = idx.branch_at(current_height);
            let current_branch = self.branches.get_mut(&current_index).unwrap();
            if current_branch.children != [[[state; 2]; 2]; 2] {
                return;
            }
            self.branches.remove(&current_index);
            let (x, y, z) = idx.bit(current_height);
            self.branches
                .get_mut(&idx.branch_at(current_height + 1))
                .unwrap()
                .children[z][y][x] = state;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut octree = OctreeBitmap::new(5);
        assert!(octree.width() >= 5);

        let a = Index::new(1, 2, 3);
        let b = Index::new(0, 3, 4);

        octree.set(&a, true);
        assert!(octree.get(&a));
        assert!(!octree.get(&b));

        octree.set(&b, true);
        assert!(octree.get(&a));
        assert!(octree.get(&b));

        octree.set(&a, false);
        assert!(!octree.get(&a));
        assert!(octree.get(&b));

        octree.set(&b, false);
        assert!(!octree.get(&a));
        assert!(!octree.get(&a));
    }
}
