use std::{
    cell::{LazyCell, RefCell},
    rc::Rc,
    sync::atomic::AtomicUsize,
};

use bindings::exports::unavi::ui::api::Guest;

#[allow(warnings)]
mod bindings;

mod button;
mod text;
mod wired_input_impls;

static ELEMENT_ID: AtomicUsize = AtomicUsize::new(0);
static mut ELEMENTS: LazyCell<RefCell<Vec<Rc<dyn Updatable>>>> = LazyCell::new(RefCell::default);

trait Updatable {
    fn id(&self) -> usize;
    fn update(&self, delta: f32);
}

struct GuestImpl;

impl Guest for GuestImpl {
    fn update_ui(delta: f32) {
        // WASM is single-threaded, mutable statics are fine.
        unsafe {
            for item in ELEMENTS.borrow().iter() {
                item.update(delta);
            }
        }
    }
}

bindings::export!(GuestImpl with_types_in bindings);
