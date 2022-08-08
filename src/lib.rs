#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl Index {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }

    fn bit(&self, idx: u32) -> Self {
        Self {
            x: (self.x >> idx) & 1,
            y: (self.y >> idx) & 1,
            z: (self.z >> idx) & 1,
        }
    }
}

impl From<(usize, usize, usize)> for Index {
    fn from((x, y, z): (usize, usize, usize)) -> Self {
        Self { x, y, z }
    }
}

impl From<[usize; 3]> for Index {
    fn from([x, y, z]: [usize; 3]) -> Self {
        Self { x, y, z }
    }
}

pub struct OctreeBitset {
    root: RawNode,
    height: u32,
}

impl OctreeBitset {
    pub const fn new(width: usize) -> Self {
        Self {
            root: RawNode::Empty,
            // ceil(log2(width))
            height: usize::BITS - width.next_power_of_two().leading_zeros(),
        }
    }

    pub fn clear(&mut self) {
        self.root = RawNode::Empty;
    }

    pub fn width(&self) -> usize {
        1 << self.height
    }

    pub fn contains(&self, idx: &Index) -> bool {
        let mut current_node = &self.root;
        let mut current_height = self.height;
        loop {
            match current_node {
                RawNode::Empty => return false,
                RawNode::Full => return true,
                RawNode::Branch { children } => {
                    if current_height == 0 {
                        unreachable!("branch node at height zero");
                    }
                    current_height -= 1;
                    let bit_idx = idx.bit(current_height);
                    current_node = &children[bit_idx.z][bit_idx.y][bit_idx.x];
                }
            }
        }
    }

    pub fn insert(&mut self, idx: &Index) -> bool {
        fn insert_inner(idx: &Index, current_node: &mut RawNode, current_height: u32) -> bool {
            let result = match current_node {
                RawNode::Empty => {
                    if current_height == 0 {
                        *current_node = RawNode::Full;
                        true
                    } else {
                        *current_node = RawNode::empty_branch();
                        insert_inner(idx, current_node, current_height)
                    }
                }
                RawNode::Full => return false,
                RawNode::Branch { children } => {
                    if current_height == 0 {
                        unreachable!("branch node at height zero");
                    }
                    let bit_idx = idx.bit(current_height - 1);
                    insert_inner(
                        idx,
                        &mut children[bit_idx.z][bit_idx.y][bit_idx.x],
                        current_height - 1,
                    )
                }
            };
            if result {
                current_node.shallow_compress();
            }
            result
        }
        insert_inner(idx, &mut self.root, self.height)
    }

    pub fn remove(&mut self, idx: &Index) -> bool {
        fn remove_inner(idx: &Index, current_node: &mut RawNode, current_height: u32) -> bool {
            let result = match current_node {
                RawNode::Empty => return false,
                RawNode::Full => {
                    if current_height == 0 {
                        *current_node = RawNode::Empty;
                        true
                    } else {
                        *current_node = RawNode::full_branch();
                        remove_inner(idx, current_node, current_height)
                    }
                }
                RawNode::Branch { children } => {
                    if current_height == 0 {
                        unreachable!("branch node at height zero");
                    }
                    let bit_idx = idx.bit(current_height - 1);
                    remove_inner(
                        idx,
                        &mut children[bit_idx.z][bit_idx.y][bit_idx.x],
                        current_height - 1,
                    )
                }
            };
            if result {
                current_node.shallow_compress();
            }
            result
        }
        remove_inner(idx, &mut self.root, self.height)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum RawNode {
    Empty,
    Full,
    Branch {
        /// Z-Y-X indexes. 0 is first half of axis partition; 1 is second half.
        children: Box<[[[RawNode; 2]; 2]; 2]>,
    },
}

impl RawNode {
    const EMPTY_CHILDREN: [[[Self; 2]; 2]; 2] = [
        [[Self::Empty, Self::Empty], [Self::Empty, Self::Empty]],
        [[Self::Empty, Self::Empty], [Self::Empty, Self::Empty]],
    ];
    const FULL_CHILDREN: [[[Self; 2]; 2]; 2] = [
        [[Self::Full, Self::Full], [Self::Full, Self::Full]],
        [[Self::Full, Self::Full], [Self::Full, Self::Full]],
    ];

    fn empty_branch() -> Self {
        Self::Branch {
            children: Box::new(Self::EMPTY_CHILDREN.clone()),
        }
    }

    fn full_branch() -> Self {
        Self::Branch {
            children: Box::new(Self::FULL_CHILDREN.clone()),
        }
    }

    fn shallow_compress(&mut self) {
        match self {
            Self::Branch { children } => {
                if *children.as_ref() == Self::EMPTY_CHILDREN {
                    *self = Self::Empty;
                } else if *children.as_ref() == Self::FULL_CHILDREN {
                    *self = Self::Full;
                }
            }
            _ => {}
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
