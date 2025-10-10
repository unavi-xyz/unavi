use std::{any::TypeId, cell::RefCell, collections::HashMap};

use crate::{
    Component,
    host_api::{SystemOrder, insert_component, order_systems, spawn},
    param::ParamGroup,
    system::{
        System,
        function_system::FunctionSystem,
        register_system::{RegisterSystem, SystemCache},
    },
    types::{ParamData, Schedule, SystemId},
};

#[derive(Default)]
pub struct ParamState {
    pub raw: Vec<u8>,
}

#[derive(Default)]
pub struct SystemState {
    pub param_state: Vec<ParamState>,
}

#[derive(Default)]
pub struct AppState {
    pub system_state: HashMap<SystemId, SystemState>,
}

#[derive(Default)]
pub struct App {
    system_ids: HashMap<TypeId, SystemId>,
    systems: HashMap<SystemId, SystemCache>,
    state: RefCell<AppState>,
}

impl App {
    pub fn insert_resource<T>(&mut self, value: T) -> &mut Self
    where
        T: Component,
    {
        let component = T::register();
        let entity = spawn().expect("failed to spawn entity");
        let data = value.to_bytes();
        insert_component(entity, component, &data).expect("failed to insert resource component");
        self
    }

    pub fn add_system<F, In>(&mut self, schedule: Schedule, f: F) -> &mut Self
    where
        F: 'static,
        In: ParamGroup + 'static,
        FunctionSystem<F, In>: System<In = In>,
    {
        let state = self.state.get_mut();

        let f = FunctionSystem::new(f);
        let (id, sys) = f.register_system(state, schedule);

        self.systems.insert(id, sys);

        let ty = std::any::TypeId::of::<F>();
        self.system_ids.insert(ty, id);

        self
    }

    #[allow(unused_variables)]
    pub fn order_systems<A, B>(&mut self, a: A, order: SystemOrder, b: B) -> &mut Self
    where
        A: 'static,
        B: 'static,
    {
        let a_ty = std::any::TypeId::of::<A>();
        let b_ty = std::any::TypeId::of::<B>();

        let Some(a_id) = self.system_ids.get(&a_ty) else {
            panic!("System {a_ty:?} not registered")
        };
        let Some(b_id) = self.system_ids.get(&b_ty) else {
            panic!("System {b_ty:?} not registered")
        };

        if let Err(e) = order_systems(*a_id, order, *b_id) {
            panic!("Failed to order systems: {e}")
        }

        self
    }

    pub fn exec_system(&self, id: u32, data: Vec<ParamData>) {
        let cache = &self.systems[&id];
        let mut app_state = self.state.borrow_mut();
        let sys_state = app_state.system_state.entry(id).or_default();
        cache.system.run_blind(sys_state, data);
    }
}
