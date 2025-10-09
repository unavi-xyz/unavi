use crate::{
    ParamState,
    types::{Param as WParam, ParamData, Query as WQuery},
};

use super::{Param, ParamMeta, component_group::ComponentGroup};

pub struct Res<'d, T>
where
    &'d T: ComponentGroup,
{
    owned: <&'d T as ComponentGroup>::Owned,
}
pub struct ResMut<'d, T>
where
    &'d mut T: ComponentGroup,
{
    owned: <&'d mut T as ComponentGroup>::Owned,
}

impl<'d, T> AsRef<T> for Res<'d, T>
where
    &'d T: ComponentGroup,
    <&'d T as ComponentGroup>::Owned: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.owned.as_ref()
    }
}
impl<'d, T> AsRef<T> for ResMut<'d, T>
where
    &'d mut T: ComponentGroup,
    <&'d mut T as ComponentGroup>::Owned: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.owned.as_ref()
    }
}

impl<'d, T> AsMut<T> for ResMut<'d, T>
where
    &'d mut T: ComponentGroup,
    <&'d mut T as ComponentGroup>::Owned: AsMut<T>,
{
    fn as_mut(&mut self) -> &mut T {
        self.owned.as_mut()
    }
}

impl<'d, T> Param for Res<'d, T>
where
    &'d T: ComponentGroup,
{
    fn register_param(_: &mut Vec<ParamState>) -> Option<WParam> {
        Some(WParam::Query(WQuery {
            components: <&T>::register_components(),
            constraints: Vec::new(),
        }))
    }
    fn mutability() -> bool {
        false
    }
    fn meta() -> Option<ParamMeta> {
        Some(ParamMeta::Query {
            component_mut: <&T>::mutability(),
            constraints: Vec::new(),
        })
    }
    fn parse_param(
        _: &mut std::slice::IterMut<ParamState>,
        data: &mut std::vec::IntoIter<ParamData>,
    ) -> Self {
        let ParamData::Query(q) = data.next().unwrap();
        let owned = <&T>::from_data(q.into_iter().next().unwrap());
        Self { owned }
    }
}
impl<'d, T> Param for ResMut<'d, T>
where
    &'d mut T: ComponentGroup,
{
    fn register_param(_: &mut Vec<ParamState>) -> Option<WParam> {
        Some(WParam::Query(WQuery {
            components: <&mut T>::register_components(),
            constraints: Vec::new(),
        }))
    }
    fn mutability() -> bool {
        false
    }
    fn meta() -> Option<ParamMeta> {
        Some(ParamMeta::Query {
            component_mut: <&mut T>::mutability(),
            constraints: Vec::new(),
        })
    }
    fn parse_param(
        _: &mut std::slice::IterMut<ParamState>,
        data: &mut std::vec::IntoIter<ParamData>,
    ) -> Self {
        let ParamData::Query(q) = data.next().unwrap();
        let owned = <&mut T>::from_data(q.into_iter().next().unwrap());
        Self { owned }
    }
}
