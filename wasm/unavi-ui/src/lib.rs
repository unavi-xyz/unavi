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
mod wired_math_impls;

static ELEMENT_ID: AtomicUsize = AtomicUsize::new(0);

/// Each item in this vec will be automatically updated every tick.
/// To add an element:
/// - Increment [ELEMENT_ID] and expose it in the [Updatable] trait.
/// - Push the element into this vec.
///
/// To remove the item, call [Updatable::remove_element].
/// You will likely want to add this call to [Drop].
static mut ELEMENTS: LazyCell<RefCell<Vec<Rc<dyn Updatable>>>> = LazyCell::new(RefCell::default);

#[allow(static_mut_refs)]
trait Updatable {
    fn id(&self) -> usize;
    fn update(&self, delta: f32);
    fn remove_element(&self) {
        let mut to_remove = Vec::default();

        unsafe {
            for (i, item) in ELEMENTS.borrow().iter().enumerate() {
                if item.id() == self.id() {
                    to_remove.push(i);
                }
            }
        }

        to_remove.sort();

        for i in to_remove {
            unsafe {
                ELEMENTS.borrow_mut().remove(i);
            }
        }
    }
}

struct GuestImpl;

#[allow(static_mut_refs)]
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
