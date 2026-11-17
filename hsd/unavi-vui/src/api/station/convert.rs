//! Converts the interface's tiles into the crate's tiling.

use crate::{
    component::{
        pane::panes::PaneId,
        station::tiling::{
            Axis,
            Node,
            Part,
            Tiling,
        },
    },
    exports::unavi::vui::api::{
        Axis as ScriptAxis,
        Tile,
    },
    wired::core::error::Error,
};

/// `station.new`: at most this many panes and this many tiles.
pub const MAX_PANES: usize = 16;
pub const MAX_TILES: usize = 64;

/// Reads the caller's tiles as a tiling over `panes`, refusing anything the
/// interface says is invalid.
pub fn tiling(panes: &[PaneId], tiles: &[Tile]) -> Result<Tiling<PaneId>, Error> {
    if panes.len() > MAX_PANES {
        return Err(refused(format!(
            "{} panes, over the limit of {MAX_PANES}",
            panes.len()
        )));
    }
    if tiles.is_empty() {
        return Err(refused("no tiles"));
    }
    if tiles.len() > MAX_TILES {
        return Err(refused(format!(
            "{} tiles, over the limit of {MAX_TILES}",
            tiles.len()
        )));
    }
    let mut walked = vec![false; tiles.len()];
    let root = tile(tiles, 0, panes, &mut walked)?;
    if let Some(at) = walked.iter().position(|seen| !seen) {
        return Err(refused(format!("tile {at} is not part of the station")));
    }
    Tiling::new(root).map_err(|why| Error::InvalidArgument(why.to_string()))
}

fn tile(
    tiles: &[Tile],
    at: usize,
    panes: &[PaneId],
    walked: &mut [bool],
) -> Result<Node<PaneId>, Error> {
    let Some(seen) = walked.get_mut(at) else {
        return Err(refused(format!("no tile {at}")));
    };
    if std::mem::replace(seen, true) {
        return Err(refused(format!("tile {at} is in the station twice")));
    }
    let Some(here) = tiles.get(at) else {
        return Err(refused(format!("no tile {at}")));
    };
    match here {
        Tile::Tabs(tabs) => {
            let mut items = Vec::with_capacity(tabs.len());
            for tab in tabs {
                let pane = panes
                    .get(*tab as usize)
                    .ok_or_else(|| refused(format!("no pane {tab}")))?;
                items.push(*pane);
            }
            Ok(Node::Tabs(items))
        }
        Tile::Split(split) => {
            let mut parts = Vec::with_capacity(split.parts.len());
            for part in &split.parts {
                parts.push(Part {
                    weight: part.weight,
                    node:   tile(tiles, part.tile as usize, panes, walked)?,
                });
            }
            Ok(Node::Split {
                axis: axis(split.axis),
                parts,
            })
        }
    }
}

const fn axis(axis: ScriptAxis) -> Axis {
    match axis {
        ScriptAxis::Across => Axis::Across,
        ScriptAxis::Down => Axis::Down,
    }
}

fn refused(why: impl std::fmt::Display) -> Error {
    Error::InvalidArgument(why.to_string())
}
