//! The default home space: a pale ground slab leaning faintly toward the
//! document's own hue, and one wrong thing standing on it. The id picks the
//! thing and where it stands.

mod ground;
mod horizon;
mod thing;

use crate::wired::scene::document::script_document;

wired_guest::generate_script!(Script);

struct Script;

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let doc = script_document()?;
        let id = doc.id();

        ground::build(&doc, id)?;
        thing::build(&doc, id)?;

        println!("Welcome home! =)");

        Ok(Self)
    }
}
