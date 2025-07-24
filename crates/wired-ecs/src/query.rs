use crate::{Component, types::ParamData};

pub struct Query<T> {
    pub items: Vec<T>,
}

impl<T> Query<T>
where
    T: Component,
{
    pub fn from_param_data(data: ParamData) -> Self {
        let ParamData::Query(query_data) = data;

        let mut items = Vec::with_capacity(query_data.len());
        for q_item in &query_data {
            let a = T::from_bytes(q_item);
            items.push(a);
        }

        Self { items }
    }
}
