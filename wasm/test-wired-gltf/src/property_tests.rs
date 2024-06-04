use crate::panic_log;

pub trait Property {
    fn id(&self) -> u32;
}

pub fn test_property<T: Property>(
    list: impl Fn() -> Vec<T>,
    create: impl Fn() -> T,
    remove: impl Fn(T),
) {
    let item = create();
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

    remove(item);
    let found_items = list();

    if !found_items.is_empty() {
        let err = format!(
            "failed to remove item. list len {}, expected 0",
            found_items.len()
        );
        panic_log(&err);
    }
}
