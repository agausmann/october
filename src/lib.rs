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
    Empty,
    Full,
    Branch,
}

pub struct OctreeBitset {
    branches: HashMap<BranchIndex, [[[RawNode; 2]; 2]; 2]>,
    height: u32,
}

impl OctreeBitset {
    pub fn new(width: u32) -> Self {
        // ceil(log2(width))
        let height = u32::BITS - width.next_power_of_two().leading_zeros();
        let mut nodes = HashMap::new();
        nodes.insert(BranchIndex::root(height), [[[RawNode::Empty; 2]; 2]; 2]);
        Self {
            branches: nodes,
            height,
        }
    }

    pub fn clear(&mut self) {
        self.branches.clear();
        self.branches.insert(
            BranchIndex::root(self.height),
            [[[RawNode::Empty; 2]; 2]; 2],
        );
    }

    pub fn width(&self) -> u32 {
        1 << self.height
    }

    pub fn contains(&self, idx: &Index) -> bool {
        let mut current_height = self.height;
        loop {
            let current_branch = &self.branches[&idx.branch_at(current_height)];
            let (x, y, z) = idx.bit(current_height - 1);
            match current_branch[z][y][x] {
                RawNode::Empty => return false,
                RawNode::Full => return true,
                RawNode::Branch => {
                    current_height -= 1;
                    if current_height == 0 {
                        unreachable!("branch node at height zero");
                    }
                }
            }
        }
    }

    pub fn insert(&mut self, idx: &Index) -> bool {
        let mut current_height = self.height;
        loop {
            let current_index = idx.branch_at(current_height);
            let current_branch = self.branches.get_mut(&current_index).unwrap();
            let (x, y, z) = idx.bit(current_height - 1);
            match current_branch[z][y][x] {
                RawNode::Full => return false,
                RawNode::Empty => {
                    if current_height == 1 {
                        current_branch[z][y][x] = RawNode::Full;
                        self.compress(idx, RawNode::Full);
                        return true;
                    } else {
                        current_branch[z][y][x] = RawNode::Branch;
                        self.branches.insert(
                            idx.branch_at(current_height - 1),
                            [[[RawNode::Empty; 2]; 2]; 2],
                        );
                    }
                }
                RawNode::Branch => {
                    current_height -= 1;
                    if current_height == 0 {
                        unreachable!("branch node at height zero");
                    }
                }
            }
        }
    }

    pub fn remove(&mut self, idx: &Index) -> bool {
        let mut current_height = self.height;
        loop {
            let current_index = idx.branch_at(current_height);
            let current_branch = self.branches.get_mut(&current_index).unwrap();
            let (x, y, z) = idx.bit(current_height - 1);
            match current_branch[z][y][x] {
                RawNode::Empty => return false,
                RawNode::Full => {
                    if current_height == 1 {
                        current_branch[z][y][x] = RawNode::Empty;
                        self.compress(idx, RawNode::Empty);
                        return true;
                    } else {
                        current_branch[z][y][x] = RawNode::Branch;
                        self.branches.insert(
                            idx.branch_at(current_height - 1),
                            [[[RawNode::Full; 2]; 2]; 2],
                        );
                    }
                }
                RawNode::Branch => {
                    current_height -= 1;
                    if current_height == 0 {
                        unreachable!("branch node at height zero");
                    }
                }
            }
        }
    }

    fn compress(&mut self, idx: &Index, state: RawNode) {
        for current_height in 1..self.height {
            let current_index = idx.branch_at(current_height);
            let current_branch = self.branches.get_mut(&current_index).unwrap();
            if *current_branch != [[[state; 2]; 2]; 2] {
                return;
            }
            self.branches.remove(&current_index);
            let (x, y, z) = idx.bit(current_height);
            self.branches
                .get_mut(&idx.branch_at(current_height + 1))
                .unwrap()[z][y][x] = state;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut octree = OctreeBitset::new(5);
        assert!(octree.width() >= 5);

        let a = Index::new(1, 2, 3);
        let b = Index::new(0, 3, 4);

        octree.insert(&a);
        assert!(octree.contains(&a));
        assert!(!octree.contains(&b));

        octree.insert(&b);
        assert!(octree.contains(&a));
        assert!(octree.contains(&b));

        octree.remove(&a);
        assert!(!octree.contains(&a));
        assert!(octree.contains(&b));

        octree.remove(&b);
        assert!(!octree.contains(&a));
        assert!(!octree.contains(&a));
    }
}
