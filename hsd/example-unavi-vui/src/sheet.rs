//! A pane the gallery floats beside the orbit: a bar, a form with each kind
//! of control, a tree that folds and a line of prose. Every value it shows is
//! one this script keeps and sets back after the pane asks for a change.

use wired_guest::math::Vec3;

use crate::unavi::vui::api::{
    ActionControl,
    BarItem,
    Block,
    BlockHeight,
    BlockKind,
    ChoiceControl,
    Control,
    FormRow,
    NumberControl,
    Pane,
    PaneEvent,
    Span,
    Tone,
    TreeNode,
};

const COLUMNS: u32 = 40;
const LINES: u32 = 18;
const AT: Vec3 = Vec3::new(0.9, 1.3, -1.0);

const PRESS: u32 = 1;
const VISIBLE: u32 = 2;
const LEVEL: u32 = 3;
const OFFSET: u32 = 4;
const MODE: u32 = 5;
const LOAD: u32 = 6;
const ROOT: u32 = 10;
const LAYERS: u32 = 11;
const MODES: [&str; 3] = ["Low", "Mid", "High"];

pub struct Sheet {
    pane:     Pane,
    presses:  u32,
    visible:  bool,
    level:    f32,
    offset:   f32,
    mode:     u32,
    root:     bool,
    layers:   bool,
    selected: u32,
}

fn block(id: u32, lines: Option<u32>, kind: BlockKind) -> Block {
    Block {
        id,
        height: lines.map_or(BlockHeight::Share, BlockHeight::Lines),
        kind,
    }
}

fn node(id: u32, label: &str, depth: u32, open: Option<bool>, selected: u32) -> TreeNode {
    TreeNode {
        id,
        label: label.to_owned(),
        depth,
        open,
        selected: id == selected,
    }
}

impl Sheet {
    pub fn new() -> anyhow::Result<Self> {
        let pane = Pane::new(COLUMNS, Some(LINES))?;
        pane.set_title("Gallery")?;
        pane.set_floating(AT);
        let sheet = Self {
            pane,
            presses: 0,
            visible: true,
            level: 0.4,
            offset: 0.0,
            mode: 1,
            root: true,
            layers: true,
            selected: ROOT,
        };
        sheet.draw()?;
        sheet.pane.set_shown(true);
        Ok(sheet)
    }

    fn draw(&self) -> anyhow::Result<()> {
        let form = vec![
            FormRow {
                id:      VISIBLE,
                label:   "Visible".to_owned(),
                control: Control::Toggle(self.visible),
            },
            FormRow {
                id:      LEVEL,
                label:   "Level".to_owned(),
                control: Control::Slider(self.level),
            },
            FormRow {
                id:      OFFSET,
                label:   "Offset".to_owned(),
                control: Control::Number(NumberControl {
                    value:      self.offset,
                    per_column: 0.1,
                }),
            },
            FormRow {
                id:      MODE,
                label:   "Mode".to_owned(),
                control: Control::Choice(ChoiceControl {
                    options:  MODES.iter().map(|mode| (*mode).to_owned()).collect(),
                    selected: self.mode,
                }),
            },
            FormRow {
                id:      LOAD,
                label:   "Load".to_owned(),
                control: Control::Meter(self.level),
            },
        ];

        let mut tree = vec![node(ROOT, "Document", 0, Some(self.root), self.selected)];
        if self.root {
            tree.push(node(LAYERS, "Layers", 1, Some(self.layers), self.selected));
            if self.layers {
                tree.push(node(12, "base", 2, None, self.selected));
                tree.push(node(13, "overrides", 2, None, self.selected));
            }
            tree.push(node(14, "Prims", 1, None, self.selected));
        }

        let blocks = vec![
            block(
                1,
                Some(1),
                BlockKind::Bar(vec![BarItem {
                    id:      PRESS,
                    control: Control::Action(ActionControl {
                        label:   "Press".to_owned(),
                        primary: true,
                    }),
                }]),
            ),
            block(2, Some(6), BlockKind::Form(form)),
            block(3, Some(5), BlockKind::Tree(tree)),
            block(
                4,
                None,
                BlockKind::Text(vec![Span {
                    text: "A pane lays itself out in columns and lines, so nothing on it \
                           waits on the host to be measured."
                        .to_owned(),
                    tone:  Tone::Muted,
                    link:  None,
                }]),
            ),
        ];
        self.pane.set_blocks(&blocks)?;
        self.pane
            .set_status(Some(&format!("{} presses", self.presses)))?;
        Ok(())
    }

    pub fn update(&mut self) -> anyhow::Result<()> {
        let events = self.pane.events();
        if events.is_empty() {
            return Ok(());
        }
        for event in events {
            match event {
                PaneEvent::Pressed(PRESS) => self.presses += 1,
                PaneEvent::Toggled((VISIBLE, on)) => self.visible = on,
                PaneEvent::Slid((LEVEL, value)) => self.level = value,
                PaneEvent::Scrubbed((OFFSET, by)) => self.offset += by,
                PaneEvent::Chosen((MODE, index)) => self.mode = index,
                PaneEvent::Folded((ROOT, open)) => self.root = open,
                PaneEvent::Folded((LAYERS, open)) => self.layers = open,
                PaneEvent::Selected(id) => self.selected = id,
                _ => {}
            }
        }
        self.draw()
    }
}
