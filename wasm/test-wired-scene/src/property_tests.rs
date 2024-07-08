use crate::{
    bindings::wired::log::api::{log, LogLevel},
    panic_log,
};

pub trait Property {
    fn id(&self) -> u32;
}

pub fn test_property<T: Property>(
    create: impl Fn() -> T,
    add: impl Fn(&T),
    list: impl Fn() -> Vec<T>,
    remove: impl Fn(T),
) {
    log(LogLevel::Debug, "starting property tests");

    log(LogLevel::Debug, "calling create");
    let item = create();
    log(LogLevel::Debug, "calling add");
    add(&item);
    log(LogLevel::Debug, "calling list");
    let found_items = list();

    if found_items.len() != 1 {
        let err = format!(
            "failed to create item. found {} items, expected 1",
            found_items.len()
        );
        panic_log(&err);
    }

    if found_items[0].id() != item.id() {
        let err = format!(
            "found item does not match. found id {}, expected {}",
            found_items[0].id(),
            item.id()
        );
        panic_log(&err);
    };

    log(LogLevel::Debug, "calling remove");
    remove(item);
    log(LogLevel::Debug, "calling list");
    let found_items = list();

    if !found_items.is_empty() {
        let err = format!(
            "failed to remove item. list len {}, expected 0",
            found_items.len()
        );
        panic_log(&err);
    }

    log(LogLevel::Debug, "completed property tests");
}
