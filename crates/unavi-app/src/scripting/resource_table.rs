#[derive(Default)]
pub struct ResourceTable {
    next_id: u32,
}

impl ResourceTable {
    pub fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}
