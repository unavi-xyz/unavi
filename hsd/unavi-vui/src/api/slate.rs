//! The `slate` resource: a pane of rows standing where the script puts it.

use smol_str::SmolStr;
use wired_guest::math::{
    Color,
    Vec2,
    Vec3,
};

use crate::{
    api::{
        drive,
        put_up,
    },
    exports::unavi::vui::api::{
        GuestSlate,
        Row as ScriptRow,
        RowKind,
        SlateEvent as ScriptEvent,
    },
    render::{
        Vui,
        slate::{
            SlateEvent,
            SlateId,
        },
    },
    slate::{
        self,
        Kind,
        Row,
    },
    wired::core::error::Error,
};

pub struct SlateRes(SlateId);

impl GuestSlate for SlateRes {
    fn new() -> Result<Self, Error> {
        Ok(Self(put_up(Vui::slate)?))
    }

    fn set_rows(&self, rows: Vec<ScriptRow>) -> Result<(), Error> {
        let rows = rows.into_iter().map(row).collect::<Vec<_>>();
        slate::check(&rows).map_err(|refused| Error::InvalidArgument(refused.to_string()))?;
        drive(|vui| {
            vui.slate_rows(self.0, rows);
            Ok(())
        })
    }

    fn size(&self) -> Vec2 {
        let mut size = Vec2::ZERO;
        if let Err(err) = drive(|vui| {
            size = vui.slate_size(self.0);
            Ok(())
        }) {
            eprintln!("vui: slate size: {err:?}");
        }
        size
    }

    fn set_position(&self, at: Vec3) {
        if let Err(err) = drive(|vui| {
            vui.slate_at(self.0, at);
            Ok(())
        }) {
            eprintln!("vui: slate position: {err:?}");
        }
    }

    fn position(&self) -> Vec3 {
        let mut at = Vec3::ZERO;
        if let Err(err) = drive(|vui| {
            at = vui.slate_position(self.0).unwrap_or_default();
            Ok(())
        }) {
            eprintln!("vui: slate position: {err:?}");
        }
        at
    }

    fn set_tether(&self, to: Option<Vec3>) {
        if let Err(err) = drive(|vui| {
            vui.slate_tether(self.0, to);
            Ok(())
        }) {
            eprintln!("vui: slate tether: {err:?}");
        }
    }

    fn set_accent(&self, accent: Option<Color>) {
        if let Err(err) = drive(|vui| {
            vui.slate_accent(self.0, accent);
            Ok(())
        }) {
            eprintln!("vui: slate accent: {err:?}");
        }
    }

    fn set_shown(&self, value: bool) {
        if let Err(err) = drive(|vui| {
            vui.slate_shown(self.0, value);
            Ok(())
        }) {
            eprintln!("vui: slate shown: {err:?}");
        }
    }

    fn events(&self) -> Vec<ScriptEvent> {
        let mut events = Vec::new();
        if let Err(err) = drive(|vui| {
            events = vui.slate_drain(self.0);
            Ok(())
        }) {
            eprintln!("vui: slate events: {err:?}");
        }
        events.into_iter().map(event).collect()
    }
}

fn row(row: ScriptRow) -> Row {
    Row {
        id:    row.id,
        label: SmolStr::new(row.label),
        kind:  match row.kind {
            RowKind::Header => Kind::Header,
            RowKind::Field(value) => Kind::Field(SmolStr::new(value)),
            RowKind::Number(number) => Kind::Number {
                value:     number.value,
                per_metre: number.per_metre,
            },
            RowKind::Toggle(on) => Kind::Toggle(on),
            RowKind::Action => Kind::Action,
            RowKind::Tabs(tabs) => Kind::Tabs {
                labels:   tabs.labels.into_iter().map(SmolStr::new).collect(),
                selected: tabs.selected,
            },
            RowKind::Section(section) => Kind::Section {
                open:    section.open,
                summary: SmolStr::new(section.summary),
            },
            RowKind::Vector(vector) => Kind::Vector {
                values:    vector.values.into(),
                per_metre: vector.per_metre,
            },
        },
        dot:   row.dot,
        mark:  row.mark,
    }
}

const fn event(event: SlateEvent) -> ScriptEvent {
    match event {
        SlateEvent::Pressed(id) => ScriptEvent::Pressed(id),
        SlateEvent::Toggled(id, on) => ScriptEvent::Toggled((id, on)),
        SlateEvent::Scrubbed(id, part, by) => ScriptEvent::Scrubbed((id, part, by)),
        SlateEvent::Released(id) => ScriptEvent::Released(id),
        SlateEvent::Picked(id, index) => ScriptEvent::Picked((id, index)),
    }
}

/// Frees the slate's prims once nothing holds this handle.
impl Drop for SlateRes {
    fn drop(&mut self) {
        if let Err(err) = drive(|vui| vui.slate_remove(self.0)) {
            eprintln!("vui: could not free a dropped slate: {err:?}");
        }
    }
}
