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

    fn node_at(&self, height: u32) -> NodeIndex {
        let mask = !((1 << height) - 1);
        NodeIndex {
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
struct NodeIndex {
    base: Index,
    height: u32,
}

impl NodeIndex {
    fn root(height: u32) -> Self {
        Self {
            base: Index { x: 0, y: 0, z: 0 },
            height,
        }
    }

    fn child(&self, x: u32, y: u32, z: u32) -> NodeIndex {
        debug_assert!(x < 2 && y < 2 && z < 2);
        let child_height = self.height - 1;
        NodeIndex {
            base: Index {
                x: self.base.x + (x << child_height),
                y: self.base.y + (y << child_height),
                z: self.base.z + (z << child_height),
            },
            height: child_height,
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
    nodes: HashMap<NodeIndex, RawNode>,
    height: u32,
}

impl OctreeBitset {
    pub fn new(width: u32) -> Self {
        // ceil(log2(width))
        let height = u32::BITS - width.next_power_of_two().leading_zeros();
        let mut nodes = HashMap::new();
        nodes.insert(NodeIndex::root(height), RawNode::Empty);
        Self { nodes, height }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.nodes
            .insert(NodeIndex::root(self.height), RawNode::Empty);
    }

    pub fn width(&self) -> u32 {
        1 << self.height
    }

    pub fn contains(&self, idx: &Index) -> bool {
        let mut current_node = &self.nodes[&NodeIndex::root(self.height)];
        let mut current_height = self.height;
        loop {
            match current_node {
                RawNode::Empty => return false,
                RawNode::Full => return true,
                RawNode::Branch => {
                    if current_height == 0 {
                        unreachable!("branch node at height zero");
                    }
                    current_height -= 1;
                    current_node = &self.nodes[&idx.node_at(current_height)];
                }
            }
        }
    }

    pub fn insert(&mut self, idx: &Index) -> bool {
        let mut current_height = self.height;
        loop {
            let current_index = idx.node_at(current_height);
            match self.nodes[&current_index] {
                RawNode::Full => return false,
                RawNode::Empty => {
                    if current_height == 0 {
                        self.nodes.insert(current_index, RawNode::Full);
                        //TODO compress
                        return true;
                    } else {
                        for x in 0..2 {
                            for y in 0..2 {
                                for z in 0..2 {
                                    self.nodes
                                        .insert(current_index.child(x, y, z), RawNode::Empty);
                                }
                            }
                        }
                        self.nodes.insert(current_index, RawNode::Branch);
                    }
                }
                RawNode::Branch => {
                    if current_height == 0 {
                        unreachable!("branch node at height zero");
                    }
                    current_height -= 1;
                }
            }
        }
    }

    pub fn remove(&mut self, idx: &Index) -> bool {
        let mut current_height = self.height;
        loop {
            let current_index = idx.node_at(current_height);
            match self.nodes[&current_index] {
                RawNode::Empty => return false,
                RawNode::Full => {
                    if current_height == 0 {
                        self.nodes.insert(current_index, RawNode::Empty);
                        //TODO compress
                        return true;
                    } else {
                        for x in 0..2 {
                            for y in 0..2 {
                                for z in 0..2 {
                                    self.nodes
                                        .insert(current_index.child(x, y, z), RawNode::Full);
                                }
                            }
                        }
                        self.nodes.insert(current_index, RawNode::Branch);
                    }
                }
                RawNode::Branch => {
                    if current_height == 0 {
                        unreachable!("branch node at height zero");
                    }
                    current_height -= 1;
                }
            }
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
