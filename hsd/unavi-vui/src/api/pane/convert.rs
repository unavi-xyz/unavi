//! Converts between the interface's pane types and the crate's own.

use smol_str::SmolStr;

use crate::{
    component::pane::{
        self,
        block::{
            BarItem,
            Block,
            Column,
            FormRow,
            Height,
            Kind,
            Node,
            Span,
            Table,
            TableRow,
            Tone,
        },
        control::Control,
    },
    exports::unavi::vui::api::{
        Block as ScriptBlock,
        BlockHeight,
        BlockKind,
        Control as ScriptControl,
        PaneEvent,
        Tone as ScriptTone,
    },
};

pub fn block(block: ScriptBlock) -> Block {
    Block {
        id:     block.id,
        height: match block.height {
            BlockHeight::Lines(lines) => Height::Lines(lines),
            BlockHeight::Share => Height::Share,
        },
        kind:   match block.kind {
            BlockKind::Text(spans) => Kind::Text(
                spans
                    .into_iter()
                    .map(|span| Span {
                        text: SmolStr::new(span.text),
                        tone: tone(span.tone),
                        link: span.link,
                    })
                    .collect(),
            ),
            BlockKind::Form(rows) => Kind::Form(
                rows.into_iter()
                    .map(|row| FormRow {
                        id:      row.id,
                        label:   SmolStr::new(row.label),
                        control: control(row.control),
                    })
                    .collect(),
            ),
            BlockKind::Table(table) => Kind::Table(Table {
                columns: table
                    .columns
                    .into_iter()
                    .map(|column| Column {
                        title:   SmolStr::new(column.title),
                        columns: column.columns,
                    })
                    .collect(),
                rows:    table
                    .rows
                    .into_iter()
                    .map(|row| TableRow {
                        id:       row.id,
                        cells:    row.cells.into_iter().map(SmolStr::new).collect(),
                        selected: row.selected,
                    })
                    .collect(),
            }),
            BlockKind::Tree(nodes) => Kind::Tree(
                nodes
                    .into_iter()
                    .map(|node| Node {
                        id:       node.id,
                        label:    SmolStr::new(node.label),
                        depth:    node.depth,
                        open:     node.open,
                        selected: node.selected,
                    })
                    .collect(),
            ),
            BlockKind::Bar(items) => Kind::Bar(
                items
                    .into_iter()
                    .map(|item| BarItem {
                        id:      item.id,
                        control: control(item.control),
                    })
                    .collect(),
            ),
        },
    }
}

fn control(control: ScriptControl) -> Control {
    match control {
        ScriptControl::Value(value) => Control::Value(SmolStr::new(value)),
        ScriptControl::Action(action) => Control::Action {
            label:   SmolStr::new(action.label),
            primary: action.primary,
        },
        ScriptControl::Toggle(on) => Control::Toggle(on),
        ScriptControl::Slider(value) => Control::Slider(value),
        ScriptControl::Meter(value) => Control::Meter(value),
        ScriptControl::Number(number) => Control::Number {
            value:      number.value,
            per_column: number.per_column,
        },
        ScriptControl::Choice(choice) => Control::Choice {
            options:  choice.options.into_iter().map(SmolStr::new).collect(),
            selected: choice.selected as usize,
        },
        ScriptControl::Link(label) => Control::Link(SmolStr::new(label)),
    }
}

const fn tone(tone: ScriptTone) -> Tone {
    match tone {
        ScriptTone::Body => Tone::Body,
        ScriptTone::Muted => Tone::Muted,
        ScriptTone::Warning => Tone::Warning,
    }
}

pub const fn event(event: pane::Event) -> PaneEvent {
    match event {
        pane::Event::Pressed(id) => PaneEvent::Pressed(id),
        pane::Event::Toggled(id, on) => PaneEvent::Toggled((id, on)),
        pane::Event::Slid(id, value) => PaneEvent::Slid((id, value)),
        pane::Event::Scrubbed(id, by) => PaneEvent::Scrubbed((id, by)),
        pane::Event::Released(id) => PaneEvent::Released(id),
        pane::Event::Chosen(id, index) => PaneEvent::Chosen((id, index)),
        pane::Event::Selected(id) => PaneEvent::Selected(id),
        pane::Event::Folded(id, open) => PaneEvent::Folded((id, open)),
        pane::Event::Followed(id) => PaneEvent::Followed(id),
    }
}
