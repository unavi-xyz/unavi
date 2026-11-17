//! A tiling: a tree of splits with items at its leaves, divided in cells.
//!
//! A leaf holds one or more items as tabs, and the first of them is the one
//! shown. Nothing here knows what an item is.

/// A size in cells: columns across, lines down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cells {
    pub w: u32,
    pub h: u32,
}

/// A rectangle of cells, from the top left of the region it was cut from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// Children side by side, left to right.
    Across,
    /// Children stacked, top to bottom.
    Down,
}

/// Why a tree was refused as a tiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Invalid {
    EmptyLeaf,
    EmptySplit,
    NoWeight,
    Twice,
}

impl std::fmt::Display for Invalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::EmptyLeaf => "a leaf holds no pane",
            Self::EmptySplit => "a split holds no part",
            Self::NoWeight => "a part has no weight",
            Self::Twice => "a pane is in the tiling twice",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part<T> {
    pub weight: u32,
    pub node:   Node<T>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node<T> {
    /// Items as tabs, the first shown.
    Tabs(Vec<T>),
    Split { axis: Axis, parts: Vec<Part<T>> },
}

/// A leaf and the cells it was given.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Leaf<'a, T> {
    pub slot: Slot,
    pub tabs: &'a [T],
}

/// A tree of splits, or nothing once every item has left it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tiling<T> {
    root: Option<Node<T>>,
}

impl<T: Copy + PartialEq> Tiling<T> {
    pub fn new(root: Node<T>) -> Result<Self, Invalid> {
        let mut seen = Vec::new();
        check(&root, &mut seen)?;
        Ok(Self { root: Some(root) })
    }

    /// Whether every item has left it.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    #[must_use]
    pub fn contains(&self, item: &T) -> bool {
        self.root.as_ref().is_some_and(|root| holds(root, item))
    }

    /// Every item, in reading order.
    #[must_use]
    pub fn items(&self) -> Vec<T> {
        self.leaves(Cells::default())
            .into_iter()
            .flat_map(|leaf| leaf.tabs.iter().copied())
            .collect()
    }

    /// Each leaf and the cells it gets from `region`, in reading order.
    #[must_use]
    pub fn leaves(&self, region: Cells) -> Vec<Leaf<'_, T>> {
        let mut out = Vec::new();
        if let Some(root) = &self.root {
            let whole = Slot {
                x: 0,
                y: 0,
                w: region.w,
                h: region.h,
            };
            collect(root, whole, &mut out);
        }
        out
    }

    /// The least region in which every leaf has at least the cells of the
    /// largest item in it, given what each item needs.
    #[must_use]
    pub fn natural(&self, size: impl Fn(&T) -> Cells) -> Cells {
        self.root
            .as_ref()
            .map_or_else(Cells::default, |root| natural(root, &size))
    }

    /// Takes `item` out. A leaf left empty goes, and a split left with one
    /// part becomes that part. Returns whether it was there.
    pub fn remove(&mut self, item: &T) -> bool {
        let Some(root) = self.root.take() else {
            return false;
        };
        let (root, found) = without(root, item);
        self.root = root;
        found
    }

    /// Moves `item` to the front of the leaf `with` is in, making it the one
    /// shown. Does nothing unless both are in the tiling.
    pub fn join(&mut self, item: T, with: &T) -> bool {
        if !self.contains(&item) || !self.contains(with) {
            return false;
        }
        if item != *with {
            self.remove(&item);
        }
        let Some(root) = &mut self.root else {
            return false;
        };
        front(root, with, item)
    }
}

fn check<T: Copy + PartialEq>(node: &Node<T>, seen: &mut Vec<T>) -> Result<(), Invalid> {
    match node {
        Node::Tabs(items) => check_tabs(items, seen),
        Node::Split { parts, .. } => check_parts(parts, seen),
    }
}

fn check_tabs<T: Copy + PartialEq>(items: &[T], seen: &mut Vec<T>) -> Result<(), Invalid> {
    if items.is_empty() {
        return Err(Invalid::EmptyLeaf);
    }
    for item in items {
        if seen.contains(item) {
            return Err(Invalid::Twice);
        }
        seen.push(*item);
    }
    Ok(())
}

fn check_parts<T: Copy + PartialEq>(parts: &[Part<T>], seen: &mut Vec<T>) -> Result<(), Invalid> {
    if parts.is_empty() {
        return Err(Invalid::EmptySplit);
    }
    for part in parts {
        if part.weight == 0 {
            return Err(Invalid::NoWeight);
        }
        check(&part.node, seen)?;
    }
    Ok(())
}

fn holds<T: PartialEq>(node: &Node<T>, item: &T) -> bool {
    match node {
        Node::Tabs(items) => items.contains(item),
        Node::Split { parts, .. } => parts.iter().any(|part| holds(&part.node, item)),
    }
}

/// `total` cut by `weights` into runs that meet exactly: each edge is where
/// its share of the total falls, so the runs add to `total`.
fn divide(total: u32, weights: &[u32]) -> Vec<(u32, u32)> {
    let sum = weights.iter().map(|weight| u64::from(*weight)).sum::<u64>();
    let mut at = 0;
    let mut start = 0;
    weights
        .iter()
        .map(|weight| {
            at += u64::from(*weight);
            let end = (u64::from(total) * at / sum.max(1)) as u32;
            let run = (start, end - start);
            start = end;
            run
        })
        .collect()
}

fn collect<'a, T>(node: &'a Node<T>, slot: Slot, out: &mut Vec<Leaf<'a, T>>) {
    match node {
        Node::Tabs(items) => out.push(Leaf { slot, tabs: items }),
        Node::Split { axis, parts } => {
            let weights = parts.iter().map(|part| part.weight).collect::<Vec<_>>();
            let total = match axis {
                Axis::Across => slot.w,
                Axis::Down => slot.h,
            };
            for ((start, run), part) in divide(total, &weights).into_iter().zip(parts) {
                let child = match axis {
                    Axis::Across => Slot {
                        x: slot.x + start,
                        w: run,
                        ..slot
                    },
                    Axis::Down => Slot {
                        y: slot.y + start,
                        h: run,
                        ..slot
                    },
                };
                collect(&part.node, child, out);
            }
        }
    }
}

fn natural<T>(node: &Node<T>, size: &impl Fn(&T) -> Cells) -> Cells {
    match node {
        Node::Tabs(items) => items.iter().map(size).fold(Cells::default(), |most, cells| Cells {
            w: most.w.max(cells.w),
            h: most.h.max(cells.h),
        }),
        Node::Split { axis, parts } => {
            let needs = parts
                .iter()
                .map(|part| (natural(&part.node, size), part.weight))
                .collect::<Vec<_>>();
            let weights = needs
                .iter()
                .map(|(_, weight)| u64::from(*weight))
                .sum::<u64>();
            let along = |pick: fn(Cells) -> u32| {
                needs
                    .iter()
                    .map(|(cells, weight)| {
                        (u64::from(pick(*cells)) * weights).div_ceil(u64::from(*weight))
                    })
                    .max()
                    .unwrap_or(0)
            };
            let across = |pick: fn(Cells) -> u32| {
                needs
                    .iter()
                    .map(|(cells, _)| pick(*cells))
                    .max()
                    .unwrap_or(0)
            };
            match axis {
                Axis::Across => Cells {
                    w: along(|cells| cells.w) as u32,
                    h: across(|cells| cells.h),
                },
                Axis::Down => Cells {
                    w: across(|cells| cells.w),
                    h: along(|cells| cells.h) as u32,
                },
            }
        }
    }
}

/// `node` without `item`, and whether it was in it.
fn without<T: Copy + PartialEq>(node: Node<T>, item: &T) -> (Option<Node<T>>, bool) {
    match node {
        Node::Tabs(mut items) => {
            let Some(at) = items.iter().position(|held| held == item) else {
                return (Some(Node::Tabs(items)), false);
            };
            items.remove(at);
            ((!items.is_empty()).then_some(Node::Tabs(items)), true)
        }
        Node::Split { axis, parts } => {
            let mut found = false;
            let mut kept = Vec::new();
            for part in parts {
                if found {
                    kept.push(part);
                    continue;
                }
                let (node, was) = without(part.node, item);
                found = was;
                if let Some(node) = node {
                    kept.push(Part {
                        weight: part.weight,
                        node,
                    });
                }
            }
            let node = match kept.len() {
                0 => None,
                1 => kept.pop().map(|part| part.node),
                _ => Some(Node::Split { axis, parts: kept }),
            };
            (node, found)
        }
    }
}

/// Puts `item` first in the leaf holding `with`.
fn front<T: Copy + PartialEq>(node: &mut Node<T>, with: &T, item: T) -> bool {
    match node {
        Node::Tabs(items) => {
            if !items.contains(with) {
                return false;
            }
            items.retain(|held| *held != item);
            items.insert(0, item);
            true
        }
        Node::Split { parts, .. } => parts
            .iter_mut()
            .any(|part| front(&mut part.node, with, item)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tab(item: u32) -> Node<u32> {
        Node::Tabs(vec![item])
    }

    fn part(weight: u32, node: Node<u32>) -> Part<u32> {
        Part { weight, node }
    }

    fn across(parts: Vec<Part<u32>>) -> Node<u32> {
        Node::Split {
            axis: Axis::Across,
            parts,
        }
    }

    fn down(parts: Vec<Part<u32>>) -> Node<u32> {
        Node::Split {
            axis: Axis::Down,
            parts,
        }
    }

    const REGION: Cells = Cells { w: 100, h: 40 };

    #[test]
    fn a_split_divides_its_cells_between_its_children() {
        let tiling = Tiling::new(across(vec![part(1, tab(1)), part(3, tab(2))])).unwrap();
        let slots = tiling
            .leaves(REGION)
            .into_iter()
            .map(|leaf| leaf.slot)
            .collect::<Vec<_>>();
        assert_eq!(
            slots,
            [
                Slot { x: 0, y: 0, w: 25, h: 40 },
                Slot { x: 25, y: 0, w: 75, h: 40 },
            ]
        );
    }

    #[test]
    fn shares_that_do_not_divide_evenly_still_add_up() {
        let tiling =
            Tiling::new(down(vec![part(1, tab(1)), part(1, tab(2)), part(1, tab(3))])).unwrap();
        let region = Cells { w: 10, h: 11 };
        let heights = tiling
            .leaves(region)
            .into_iter()
            .map(|leaf| leaf.slot.h)
            .collect::<Vec<_>>();
        assert_eq!(heights.iter().sum::<u32>(), 11);
        assert!(heights.iter().all(|h| (3..=4).contains(h)));
    }

    #[test]
    fn splits_nest() {
        let tiling = Tiling::new(across(vec![
            part(1, tab(1)),
            part(1, down(vec![part(1, tab(2)), part(1, tab(3))])),
        ]))
        .unwrap();
        let leaves = tiling.leaves(REGION);
        assert_eq!(leaves[1].slot, Slot { x: 50, y: 0, w: 50, h: 20 });
        assert_eq!(leaves[2].slot, Slot { x: 50, y: 20, w: 50, h: 20 });
    }

    #[test]
    fn removing_a_leaf_collapses_its_split() {
        let mut tiling = Tiling::new(across(vec![
            part(1, tab(1)),
            part(1, down(vec![part(1, tab(2)), part(1, tab(3))])),
        ]))
        .unwrap();
        assert!(tiling.remove(&3));
        let slots = tiling
            .leaves(REGION)
            .into_iter()
            .map(|leaf| (leaf.tabs.to_vec(), leaf.slot))
            .collect::<Vec<_>>();
        assert_eq!(
            slots,
            [
                (vec![1], Slot { x: 0, y: 0, w: 50, h: 40 }),
                (vec![2], Slot { x: 50, y: 0, w: 50, h: 40 }),
            ],
            "the down split left with one part is that part"
        );
    }

    #[test]
    fn removing_the_last_item_empties_the_tiling() {
        let mut tiling = Tiling::new(tab(1)).unwrap();
        assert!(tiling.remove(&1));
        assert!(tiling.is_empty());
        assert!(!tiling.remove(&1));
    }

    #[test]
    fn removing_a_tab_keeps_its_leaf() {
        let mut tiling = Tiling::new(Node::Tabs(vec![1, 2])).unwrap();
        assert!(tiling.remove(&1));
        assert_eq!(tiling.leaves(REGION)[0].tabs, [2]);
    }

    #[test]
    fn joining_puts_an_item_first_in_another_leaf() {
        let mut tiling = Tiling::new(across(vec![part(1, tab(1)), part(1, tab(2))])).unwrap();
        assert!(tiling.join(1, &2));
        let leaves = tiling.leaves(REGION);
        assert_eq!(leaves.len(), 1, "the leaf it left was empty");
        assert_eq!(leaves[0].tabs, [1, 2]);
    }

    #[test]
    fn joining_its_own_leaf_makes_it_the_one_shown() {
        let mut tiling = Tiling::new(Node::Tabs(vec![1, 2, 3])).unwrap();
        assert!(tiling.join(3, &1));
        assert_eq!(tiling.leaves(REGION)[0].tabs, [3, 1, 2]);
        assert!(!tiling.join(9, &1), "an item that is not in it");
    }

    #[test]
    fn a_tiling_with_an_empty_part_or_a_repeat_is_refused() {
        assert_eq!(Tiling::new(Node::Tabs(Vec::<u32>::new())), Err(Invalid::EmptyLeaf));
        assert_eq!(Tiling::<u32>::new(across(Vec::new())), Err(Invalid::EmptySplit));
        assert_eq!(Tiling::new(across(vec![part(0, tab(1))])), Err(Invalid::NoWeight));
        assert_eq!(
            Tiling::new(across(vec![part(1, tab(1)), part(1, tab(1))])),
            Err(Invalid::Twice)
        );
    }

    #[test]
    fn the_natural_region_gives_every_leaf_room_for_its_item() {
        let tiling = Tiling::new(across(vec![
            part(1, tab(1)),
            part(1, down(vec![part(1, tab(2)), part(3, tab(3))])),
        ]))
        .unwrap();
        let size = |item: &u32| match item {
            1 => Cells { w: 40, h: 30 },
            2 => Cells { w: 80, h: 10 },
            _ => Cells { w: 64, h: 20 },
        };
        let region = tiling.natural(size);
        for leaf in tiling.leaves(region) {
            let need = size(&leaf.tabs[0]);
            assert!(
                leaf.slot.w >= need.w && leaf.slot.h >= need.h,
                "{:?} does not fit {:?}",
                leaf.slot,
                need
            );
        }
    }

    #[test]
    fn items_read_in_order() {
        let tiling = Tiling::new(across(vec![
            part(1, Node::Tabs(vec![4, 5])),
            part(1, tab(6)),
        ]))
        .unwrap();
        assert_eq!(tiling.items(), [4, 5, 6]);
    }
}
