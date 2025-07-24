use crate::{
    Component,
    param::Param,
    types::{Param as BParam, ParamData, Query as BQuery},
};

pub struct Query<T> {
    pub items: Vec<T>,
}

impl<A> Param for Query<A>
where
    A: Component,
{
    fn parse_param(data: ParamData) -> Self {
        let ParamData::Query(query_data) = data;

        let mut items = Vec::with_capacity(query_data.len());
        for q_item in &query_data {
            let a = A::from_bytes(q_item);
            items.push(a);
        }

        Self { items }
    }
    fn register_param() -> BParam {
        BParam::Query(BQuery {
            components: vec![A::register()],
            constraints: Vec::new(),
        })
    }
}

impl<A> Param for Query<(A,)>
where
    A: Component,
{
    fn parse_param(data: ParamData) -> Self {
        let ParamData::Query(query_data) = data;

        let mut items = Vec::with_capacity(query_data.len());
        for q_item in &query_data {
            let a = A::from_bytes(q_item);
            items.push((a,));
        }

        Self { items }
    }
    fn register_param() -> BParam {
        BParam::Query(BQuery {
            components: vec![A::register()],
            constraints: Vec::new(),
        })
    }
}

impl<A, B> Param for Query<(A, B)>
where
    A: Component,
    B: Component,
{
    fn parse_param(data: ParamData) -> Self {
        let ParamData::Query(query_data) = data;

        let mut items = Vec::with_capacity(query_data.len());
        for q_item in &query_data {
            let xa = A::byte_len();

            let a = A::from_bytes(&q_item[..xa]);
            let b = B::from_bytes(&q_item[xa..]);

            items.push((a, b));
        }

        Self { items }
    }
    fn register_param() -> BParam {
        BParam::Query(BQuery {
            components: vec![A::register(), B::register()],
            constraints: Vec::new(),
        })
    }
}

impl<A, B, C> Param for Query<(A, B, C)>
where
    A: Component,
    B: Component,
    C: Component,
{
    fn parse_param(data: ParamData) -> Self {
        let ParamData::Query(query_data) = data;

        let mut items = Vec::with_capacity(query_data.len());
        for q_item in &query_data {
            let xa = A::byte_len();
            let xb = xa + B::byte_len();

            let a = A::from_bytes(&q_item[..xa]);
            let b = B::from_bytes(&q_item[xa..xb]);
            let c = C::from_bytes(&q_item[xb..]);

            items.push((a, b, c));
        }

        Self { items }
    }
    fn register_param() -> BParam {
        BParam::Query(BQuery {
            components: vec![A::register(), B::register(), C::register()],
            constraints: Vec::new(),
        })
    }
}
