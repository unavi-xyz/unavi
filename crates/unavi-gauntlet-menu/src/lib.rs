//! Pure navigation state for the gauntlet wheel: a nested menu tree, an open
//! path, and single-active tool tracking. Free of rendering and host bindings
//! so it can be unit tested on the host target.

pub type DocId = Vec<u8>;

pub const HOME_LABEL: &str = "Home";
pub const TOOLS_LABEL: &str = "Tools";
pub const BACK_LABEL: &str = "Back";
pub const CONFIRM_LABEL: &str = "Confirm";

/// Glyph hint for the renderer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Home,
    Tools,
    Back,
    Confirm,
    Tool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    GoHome,
    ActivateTool(DocId),
}

#[derive(Clone, Debug)]
pub enum NodeKind {
    Action(Action),
    Submenu(Vec<MenuNode>),
}

#[derive(Clone, Debug)]
pub struct MenuNode {
    pub label: String,
    pub icon:  Icon,
    pub kind:  NodeKind,
}

/// A single wheel sector as presented to the renderer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub label:  String,
    pub icon:   Icon,
    /// Drawn with the active-tool indicator (outline).
    pub active: bool,
}

/// The tool transition a selection produced, so the caller can emit the
/// matching activate/deactivate to the tool documents.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ToolChange {
    pub activated:   Option<DocId>,
    pub deactivated: Option<DocId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    None,
    Home,
    Tool(ToolChange),
}

pub struct Menu {
    tools:  Vec<(DocId, String)>,
    stack:  Vec<usize>,
    open:   bool,
    active: Option<DocId>,
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

impl Menu {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            tools:  Vec::new(),
            stack:  Vec::new(),
            open:   false,
            active: None,
        }
    }

    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.open
    }

    #[must_use]
    pub const fn active_tool(&self) -> Option<&DocId> {
        self.active.as_ref()
    }

    #[must_use]
    pub const fn depth(&self) -> usize {
        self.stack.len()
    }

    pub const fn open(&mut self) {
        self.open = true;
    }

    pub const fn close(&mut self) {
        self.open = false;
    }

    pub const fn toggle_open(&mut self) {
        if self.open {
            self.close();
        } else {
            self.open();
        }
    }

    /// Replaces the discovered tool set, dropping the active tool and any open
    /// submenu path that the new set invalidates.
    pub fn set_tools(&mut self, tools: Vec<(DocId, String)>) {
        if let Some(active) = &self.active
            && !tools.iter().any(|(id, _)| id == active)
        {
            self.active = None;
        }
        self.tools = tools;
        if !self.path_valid() {
            self.stack.clear();
        }
    }

    /// The sectors for the currently open level, with a leading `Back` when
    /// nested.
    #[must_use]
    pub fn slots(&self) -> Vec<Slot> {
        let mut slots = Vec::new();
        if !self.stack.is_empty() {
            slots.push(Slot {
                label:  BACK_LABEL.to_string(),
                icon:   Icon::Back,
                active: false,
            });
        }
        for node in self.level_nodes() {
            let active = match &node.kind {
                NodeKind::Action(Action::ActivateTool(id)) => self.active.as_ref() == Some(id),
                _ => false,
            };
            slots.push(Slot {
                label: node.label,
                icon: node.icon,
                active,
            });
        }
        slots
    }

    /// Applies a selection on the slot at `slot_index` (as returned by
    /// [`Menu::slots`]), mutating navigation state and returning any action for
    /// the caller to carry out.
    pub fn select(&mut self, slot_index: usize) -> Outcome {
        let has_back = !self.stack.is_empty();
        if has_back && slot_index == 0 {
            self.stack.pop();
            return Outcome::None;
        }

        let node_index = slot_index - usize::from(has_back);
        let Some(node) = self.level_nodes().into_iter().nth(node_index) else {
            return Outcome::None;
        };

        match node.kind {
            NodeKind::Submenu(_) => {
                self.stack.push(node_index);
                Outcome::None
            }
            NodeKind::Action(Action::GoHome) => {
                self.close();
                Outcome::Home
            }
            NodeKind::Action(Action::ActivateTool(id)) => {
                let change = self.toggle_tool(id);
                self.close();
                Outcome::Tool(change)
            }
        }
    }

    fn toggle_tool(&mut self, id: DocId) -> ToolChange {
        if self.active.as_ref() == Some(&id) {
            self.active = None;
            ToolChange {
                activated:   None,
                deactivated: Some(id),
            }
        } else {
            let deactivated = self.active.take();
            self.active = Some(id.clone());
            ToolChange {
                activated: Some(id),
                deactivated,
            }
        }
    }

    fn root(&self) -> Vec<MenuNode> {
        vec![
            MenuNode {
                label: HOME_LABEL.to_string(),
                icon:  Icon::Home,
                kind:  NodeKind::Submenu(vec![MenuNode {
                    label: CONFIRM_LABEL.to_string(),
                    icon:  Icon::Confirm,
                    kind:  NodeKind::Action(Action::GoHome),
                }]),
            },
            MenuNode {
                label: TOOLS_LABEL.to_string(),
                icon:  Icon::Tools,
                kind:  NodeKind::Submenu(self.tool_nodes()),
            },
        ]
    }

    fn tool_nodes(&self) -> Vec<MenuNode> {
        self.tools
            .iter()
            .map(|(id, name)| MenuNode {
                label: name.clone(),
                icon:  Icon::Tool,
                kind:  NodeKind::Action(Action::ActivateTool(id.clone())),
            })
            .collect()
    }

    fn path_valid(&self) -> bool {
        let mut nodes = self.root();
        for &index in &self.stack {
            match nodes.into_iter().nth(index).map(|n| n.kind) {
                Some(NodeKind::Submenu(children)) => nodes = children,
                _ => return false,
            }
        }
        true
    }

    fn level_nodes(&self) -> Vec<MenuNode> {
        let mut nodes = self.root();
        for &index in &self.stack {
            match nodes.into_iter().nth(index).map(|n| n.kind) {
                Some(NodeKind::Submenu(children)) => nodes = children,
                _ => return Vec::new(),
            }
        }
        nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOME_SLOT: usize = 0;
    const TOOLS_SLOT: usize = 1;

    fn id(byte: u8) -> DocId {
        vec![byte]
    }

    fn menu_with_tools() -> Menu {
        let mut menu = Menu::new();
        menu.set_tools(vec![(id(1), "Physgun".into()), (id(2), "Spawner".into())]);
        menu
    }

    fn labels(menu: &Menu) -> Vec<String> {
        menu.slots().into_iter().map(|s| s.label).collect()
    }

    fn open_tools(menu: &mut Menu) {
        menu.open();
        menu.select(TOOLS_SLOT);
    }

    #[test]
    fn root_shows_home_and_tools() {
        let menu = menu_with_tools();
        assert_eq!(labels(&menu), vec!["Home", "Tools"]);
        let icons = menu.slots().into_iter().map(|s| s.icon).collect::<Vec<_>>();
        assert_eq!(icons, vec![Icon::Home, Icon::Tools]);
        assert!(!menu.is_open());
    }

    #[test]
    fn open_toggle_tracks_state() {
        let mut menu = Menu::new();
        menu.toggle_open();
        assert!(menu.is_open());
        menu.toggle_open();
        assert!(!menu.is_open());
    }

    #[test]
    fn entering_tools_shows_back_and_tool_sectors() {
        let mut menu = menu_with_tools();
        open_tools(&mut menu);
        assert_eq!(menu.depth(), 1);
        assert_eq!(labels(&menu), vec!["Back", "Physgun", "Spawner"]);
    }

    #[test]
    fn back_restores_root() {
        let mut menu = menu_with_tools();
        open_tools(&mut menu);
        assert_eq!(menu.select(0), Outcome::None); // Back
        assert_eq!(menu.depth(), 0);
        assert_eq!(labels(&menu), vec!["Home", "Tools"]);
    }

    #[test]
    fn home_requires_confirmation() {
        let mut menu = menu_with_tools();
        menu.open();
        assert_eq!(menu.select(HOME_SLOT), Outcome::None); // descend into Home
        assert_eq!(labels(&menu), vec!["Back", "Confirm"]);
        assert_eq!(menu.select(1), Outcome::Home); // Confirm
        assert!(!menu.is_open());
    }

    #[test]
    fn home_can_be_cancelled() {
        let mut menu = menu_with_tools();
        menu.open();
        menu.select(HOME_SLOT); // descend into Home
        assert_eq!(menu.select(0), Outcome::None); // Back = cancel
        assert!(menu.is_open());
        assert_eq!(labels(&menu), vec!["Home", "Tools"]);
    }

    #[test]
    fn selecting_tool_activates_and_closes() {
        let mut menu = menu_with_tools();
        open_tools(&mut menu);
        let outcome = menu.select(1); // Physgun (slot 0 is Back)
        assert_eq!(
            outcome,
            Outcome::Tool(ToolChange {
                activated:   Some(id(1)),
                deactivated: None,
            })
        );
        assert_eq!(menu.active_tool(), Some(&id(1)));
        assert!(!menu.is_open());
    }

    #[test]
    fn reopens_at_last_level() {
        let mut menu = menu_with_tools();
        open_tools(&mut menu);
        assert_eq!(menu.depth(), 1);
        menu.close();
        menu.open();
        assert_eq!(menu.depth(), 1);
        assert_eq!(labels(&menu), vec!["Back", "Physgun", "Spawner"]);
    }

    #[test]
    fn only_one_tool_active_at_a_time() {
        let mut menu = menu_with_tools();
        open_tools(&mut menu);
        menu.select(1); // Physgun, closes at Tools level

        menu.open(); // reopens at Tools
        let outcome = menu.select(2); // Spawner
        assert_eq!(
            outcome,
            Outcome::Tool(ToolChange {
                activated:   Some(id(2)),
                deactivated: Some(id(1)),
            })
        );
        assert_eq!(menu.active_tool(), Some(&id(2)));
    }

    #[test]
    fn reselecting_active_tool_deactivates_it() {
        let mut menu = menu_with_tools();
        open_tools(&mut menu);
        menu.select(1); // Physgun on, closes at Tools level

        menu.open(); // reopens at Tools
        let outcome = menu.select(1); // Physgun off
        assert_eq!(
            outcome,
            Outcome::Tool(ToolChange {
                activated:   None,
                deactivated: Some(id(1)),
            })
        );
        assert_eq!(menu.active_tool(), None);
    }

    #[test]
    fn active_slot_is_flagged() {
        let mut menu = menu_with_tools();
        open_tools(&mut menu);
        menu.select(1); // Physgun on, closes at Tools level

        menu.open(); // reopens at Tools
        let slots = menu.slots();
        assert!(slots[1].active); // Physgun
        assert!(!slots[2].active); // Spawner
    }

    #[test]
    fn dropping_active_tool_from_set_clears_it() {
        let mut menu = menu_with_tools();
        open_tools(&mut menu);
        menu.select(1); // Physgun on
        assert_eq!(menu.active_tool(), Some(&id(1)));

        menu.set_tools(vec![(id(2), "Spawner".into())]);
        assert_eq!(menu.active_tool(), None);
    }

    #[test]
    fn shrinking_tools_while_nested_resets_path() {
        let mut menu = menu_with_tools();
        open_tools(&mut menu);
        assert_eq!(menu.depth(), 1);
        menu.set_tools(Vec::new());
        assert_eq!(labels(&menu), vec!["Back"]);
    }
}
