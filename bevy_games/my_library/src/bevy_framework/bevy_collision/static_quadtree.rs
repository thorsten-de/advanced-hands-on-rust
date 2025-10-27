//! Static QuadTree structure for recursive division of colliding entities in space
use crate::Rect2D;
use bevy::{platform::collections::HashSet, prelude::*};

/// A node in the static tree representation of recursive quadrants
#[derive(Debug)]
pub struct StaticQuadTreeNode {
    /// Size of this node
    bounds: Rect2D,
    /// Children quadrants of this node. None if this is
    /// on the maximal depth level
    children: Option<[usize; 4]>,
}

/// Resource for nodes in a quad-tree collision detection algorithm
#[derive(Debug, Resource)]
pub struct StaticQuadTree {
    nodes: Vec<StaticQuadTreeNode>,
}

impl StaticQuadTree {
    /// Creates a new static QuadTree
    pub fn new(screen_size: Vec2, max_depth: usize) -> Self {
        let mut nodes = Vec::new();

        let half = screen_size / 2.0;
        let top = StaticQuadTreeNode {
            bounds: Rect2D::new(Vec2::ZERO - half, half),
            children: None,
        };
        nodes.push(top);
        Self::subdivide(&mut nodes, 0, 1, max_depth);
        Self { nodes }
    }

    /// Recursively divides the quadrants of a node in the QuadTree
    fn subdivide(
        nodes: &mut Vec<StaticQuadTreeNode>,
        index: usize,
        depth: usize,
        max_depth: usize,
    ) {
        let mut children = nodes[index].bounds.quadrants();
        let n = nodes.len();
        let child_index = [n, n + 1, n + 2, n + 3];
        nodes[index].children = Some(child_index);
        children.drain(0..4).for_each(|quad| {
            nodes.push(StaticQuadTreeNode {
                bounds: quad,
                children: None,
            })
        });

        if depth < max_depth {
            for index in child_index {
                Self::subdivide(nodes, index, depth + 1, max_depth);
            }
        }
    }

    /// Finds the smallest quadrant that completely contains an entity
    pub fn smallest_node(&self, target: &Rect2D) -> usize {
        let mut current_index = 0;

        #[allow(clippy::while_let_loop)]
        loop {
            if let Some(children) = self.nodes[current_index].children {
                let matches: Vec<usize> = children
                    .iter()
                    .filter_map(|child| {
                        if self.nodes[*child].bounds.intersect(target) {
                            Some(*child)
                        } else {
                            None
                        }
                    })
                    .collect();

                if matches.len() == 1 {
                    // The target is contained in only one quadrant, so dive deeper
                    // within that quadrant
                    current_index = matches[0];
                } else {
                    // If there is more than one match, the target is contained in
                    // more than one quadrants, thus overlapping a boundary. We
                    // cannot get deeper than we are.
                    break;
                }
            } else {
                // There are no children quadrants defined at this level, so
                // don't dive deeper.
                break;
            }
        }
        current_index
    }

    /// finds all intersecting nodes of the tree with a given bounding box rectangle
    pub fn intersecting_nodes(&self, target: &Rect2D) -> HashSet<usize> {
        let mut result = HashSet::new();
        self.intersect(0, &mut result, target);
        result
    }

    fn intersect(&self, index: usize, result: &mut HashSet<usize>, target: &Rect2D) {
        if self.nodes[index].bounds.intersect(target) {
            result.insert(index);
            if let Some(children) = &self.nodes[index].children {
                for child in children {
                    self.intersect(*child, result, target);
                }
            }
        }
    }
}
