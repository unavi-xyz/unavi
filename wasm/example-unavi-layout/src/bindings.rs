#[allow(dead_code)]
pub mod unavi {
    #[allow(dead_code)]
    pub mod layout {
        #[allow(dead_code, clippy::all)]
        pub mod container {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() =
                super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            pub type Vec3 = super::super::super::wired::math::types::Vec3;
            pub type Node = super::super::super::wired::scene::node::Node;
            #[repr(u8)]
            #[derive(Clone, Copy, Eq, PartialEq)]
            pub enum Alignment {
                Center,
                End,
                Start,
            }
            impl ::core::fmt::Debug for Alignment {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        Alignment::Center => f.debug_tuple("Alignment::Center").finish(),
                        Alignment::End => f.debug_tuple("Alignment::End").finish(),
                        Alignment::Start => f.debug_tuple("Alignment::Start").finish(),
                    }
                }
            }
            impl Alignment {
                #[doc(hidden)]
                pub unsafe fn _lift(val: u8) -> Alignment {
                    if !cfg!(debug_assertions) {
                        return ::core::mem::transmute(val);
                    }
                    match val {
                        0 => Alignment::Center,
                        1 => Alignment::End,
                        2 => Alignment::Start,
                        _ => panic!("invalid enum discriminant"),
                    }
                }
            }
            /// A 3D area of space.
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct Container {
                handle: _rt::Resource<Container>,
            }
            impl Container {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for Container {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "unavi:layout/container")]
                        extern "C" {
                            #[link_name = "[resource-drop]container"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            impl Container {
                #[allow(unused_unsafe, clippy::all)]
                pub fn new(size: Vec3) -> Self {
                    unsafe {
                        let super::super::super::wired::math::types::Vec3 {
                            x: x0,
                            y: y0,
                            z: z0,
                        } = size;
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:layout/container")]
                        extern "C" {
                            #[link_name = "[constructor]container"]
                            fn wit_import(_: f32, _: f32, _: f32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: f32, _: f32, _: f32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(_rt::as_f32(x0), _rt::as_f32(y0), _rt::as_f32(z0));
                        Container::from_handle(ret as u32)
                    }
                }
            }
            impl Container {
                #[allow(unused_unsafe, clippy::all)]
                /// Returns another reference to the same resource.
                pub fn ref_(&self) -> Container {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:layout/container")]
                        extern "C" {
                            #[link_name = "[method]container.ref"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        Container::from_handle(ret as u32)
                    }
                }
            }
            impl Container {
                #[allow(unused_unsafe, clippy::all)]
                /// The `root` node.
                /// Positioned in the center of the container.
                pub fn root(&self) -> Node {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:layout/container")]
                        extern "C" {
                            #[link_name = "[method]container.root"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::scene::node::Node::from_handle(ret as u32)
                    }
                }
            }
            impl Container {
                #[allow(unused_unsafe, clippy::all)]
                /// The `inner` node, child of the `root` node.
                /// Positioned according to the `alignment` of the container.
                pub fn inner(&self) -> Node {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:layout/container")]
                        extern "C" {
                            #[link_name = "[method]container.inner"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::scene::node::Node::from_handle(ret as u32)
                    }
                }
            }
            impl Container {
                #[allow(unused_unsafe, clippy::all)]
                pub fn list_children(&self) -> _rt::Vec<Container> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:layout/container")]
                        extern "C" {
                            #[link_name = "[method]container.list-children"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<*mut u8>();
                        let l2 = *ptr0.add(4).cast::<usize>();
                        let base4 = l1;
                        let len4 = l2;
                        let mut result4 = _rt::Vec::with_capacity(len4);
                        for i in 0..len4 {
                            let base = base4.add(i * 4);
                            let e4 = {
                                let l3 = *base.add(0).cast::<i32>();
                                Container::from_handle(l3 as u32)
                            };
                            result4.push(e4);
                        }
                        _rt::cabi_dealloc(base4, len4 * 4, 4);
                        result4
                    }
                }
            }
            impl Container {
                #[allow(unused_unsafe, clippy::all)]
                pub fn add_child(&self, child: &Container) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:layout/container")]
                        extern "C" {
                            #[link_name = "[method]container.add-child"]
                            fn wit_import(_: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, (child).handle() as i32);
                    }
                }
            }
            impl Container {
                #[allow(unused_unsafe, clippy::all)]
                pub fn remove_child(&self, child: &Container) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:layout/container")]
                        extern "C" {
                            #[link_name = "[method]container.remove-child"]
                            fn wit_import(_: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, (child).handle() as i32);
                    }
                }
            }
            impl Container {
                #[allow(unused_unsafe, clippy::all)]
                pub fn size(&self) -> Vec3 {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 12]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:layout/container")]
                        extern "C" {
                            #[link_name = "[method]container.size"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<f32>();
                        let l2 = *ptr0.add(4).cast::<f32>();
                        let l3 = *ptr0.add(8).cast::<f32>();
                        super::super::super::wired::math::types::Vec3 {
                            x: l1,
                            y: l2,
                            z: l3,
                        }
                    }
                }
            }
            impl Container {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_size(&self, value: Vec3) {
                    unsafe {
                        let super::super::super::wired::math::types::Vec3 {
                            x: x0,
                            y: y0,
                            z: z0,
                        } = value;
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:layout/container")]
                        extern "C" {
                            #[link_name = "[method]container.set-size"]
                            fn wit_import(_: i32, _: f32, _: f32, _: f32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: f32, _: f32, _: f32) {
                            unreachable!()
                        }
                        wit_import(
                            (self).handle() as i32,
                            _rt::as_f32(x0),
                            _rt::as_f32(y0),
                            _rt::as_f32(z0),
                        );
                    }
                }
            }
            impl Container {
                #[allow(unused_unsafe, clippy::all)]
                pub fn align_x(&self) -> Alignment {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:layout/container")]
                        extern "C" {
                            #[link_name = "[method]container.align-x"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        Alignment::_lift(ret as u8)
                    }
                }
            }
            impl Container {
                #[allow(unused_unsafe, clippy::all)]
                pub fn align_y(&self) -> Alignment {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:layout/container")]
                        extern "C" {
                            #[link_name = "[method]container.align-y"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        Alignment::_lift(ret as u8)
                    }
                }
            }
            impl Container {
                #[allow(unused_unsafe, clippy::all)]
                pub fn align_z(&self) -> Alignment {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:layout/container")]
                        extern "C" {
                            #[link_name = "[method]container.align-z"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        Alignment::_lift(ret as u8)
                    }
                }
            }
            impl Container {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_align_x(&self, value: Alignment) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:layout/container")]
                        extern "C" {
                            #[link_name = "[method]container.set-align-x"]
                            fn wit_import(_: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, value.clone() as i32);
                    }
                }
            }
            impl Container {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_align_y(&self, value: Alignment) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:layout/container")]
                        extern "C" {
                            #[link_name = "[method]container.set-align-y"]
                            fn wit_import(_: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, value.clone() as i32);
                    }
                }
            }
            impl Container {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_align_z(&self, value: Alignment) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:layout/container")]
                        extern "C" {
                            #[link_name = "[method]container.set-align-z"]
                            fn wit_import(_: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, value.clone() as i32);
                    }
                }
            }
        }
        #[allow(dead_code, clippy::all)]
        pub mod grid {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() =
                super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            pub type Container = super::super::super::unavi::layout::container::Container;
            pub type Vec3 = super::super::super::wired::math::types::Vec3;
            /// Creates a 3D grid of `container`s, with a set number of `rows` in each direction.
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct Grid {
                handle: _rt::Resource<Grid>,
            }
            impl Grid {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for Grid {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "unavi:layout/grid")]
                        extern "C" {
                            #[link_name = "[resource-drop]grid"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            impl Grid {
                #[allow(unused_unsafe, clippy::all)]
                pub fn new(size: Vec3, rows: Vec3) -> Self {
                    unsafe {
                        let super::super::super::wired::math::types::Vec3 {
                            x: x0,
                            y: y0,
                            z: z0,
                        } = size;
                        let super::super::super::wired::math::types::Vec3 {
                            x: x1,
                            y: y1,
                            z: z1,
                        } = rows;
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:layout/grid")]
                        extern "C" {
                            #[link_name = "[constructor]grid"]
                            fn wit_import(_: f32, _: f32, _: f32, _: f32, _: f32, _: f32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: f32, _: f32, _: f32, _: f32, _: f32, _: f32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(
                            _rt::as_f32(x0),
                            _rt::as_f32(y0),
                            _rt::as_f32(z0),
                            _rt::as_f32(x1),
                            _rt::as_f32(y1),
                            _rt::as_f32(z1),
                        );
                        Grid::from_handle(ret as u32)
                    }
                }
            }
            impl Grid {
                #[allow(unused_unsafe, clippy::all)]
                /// The `root` container.
                pub fn root(&self) -> Container {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:layout/grid")]
                        extern "C" {
                            #[link_name = "[method]grid.root"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::unavi::layout::container::Container::from_handle(
                            ret as u32,
                        )
                    }
                }
            }
            impl Grid {
                #[allow(unused_unsafe, clippy::all)]
                /// List of all child containers.
                pub fn cells(&self) -> _rt::Vec<Container> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:layout/grid")]
                        extern "C" {
                            #[link_name = "[method]grid.cells"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<*mut u8>();
                        let l2 = *ptr0.add(4).cast::<usize>();
                        let base4 = l1;
                        let len4 = l2;
                        let mut result4 = _rt::Vec::with_capacity(len4);
                        for i in 0..len4 {
                            let base = base4.add(i * 4);
                            let e4 = {
                                let l3 = *base.add(0).cast::<i32>();
                                super::super::super::unavi::layout::container::Container::from_handle(
                                    l3 as u32,
                                )
                            };
                            result4.push(e4);
                        }
                        _rt::cabi_dealloc(base4, len4 * 4, 4);
                        result4
                    }
                }
            }
            impl Grid {
                #[allow(unused_unsafe, clippy::all)]
                /// Gets a specific cell from the grid.
                /// Returns `none` if out of bounds.
                pub fn cell(&self, x: u32, y: u32, z: u32) -> Option<Container> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:layout/grid")]
                        extern "C" {
                            #[link_name = "[method]grid.cell"]
                            fn wit_import(_: i32, _: i32, _: i32, _: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32, _: i32, _: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import(
                            (self).handle() as i32,
                            _rt::as_i32(&x),
                            _rt::as_i32(&y),
                            _rt::as_i32(&z),
                            ptr0,
                        );
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<i32>();
                                    super::super::super::unavi::layout::container::Container::from_handle(
                                        l2 as u32,
                                    )
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Grid {
                #[allow(unused_unsafe, clippy::all)]
                pub fn rows(&self) -> Vec3 {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 12]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:layout/grid")]
                        extern "C" {
                            #[link_name = "[method]grid.rows"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<f32>();
                        let l2 = *ptr0.add(4).cast::<f32>();
                        let l3 = *ptr0.add(8).cast::<f32>();
                        super::super::super::wired::math::types::Vec3 {
                            x: l1,
                            y: l2,
                            z: l3,
                        }
                    }
                }
            }
            impl Grid {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_rows(&self, value: Vec3) {
                    unsafe {
                        let super::super::super::wired::math::types::Vec3 {
                            x: x0,
                            y: y0,
                            z: z0,
                        } = value;
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:layout/grid")]
                        extern "C" {
                            #[link_name = "[method]grid.set-rows"]
                            fn wit_import(_: i32, _: f32, _: f32, _: f32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: f32, _: f32, _: f32) {
                            unreachable!()
                        }
                        wit_import(
                            (self).handle() as i32,
                            _rt::as_f32(x0),
                            _rt::as_f32(y0),
                            _rt::as_f32(z0),
                        );
                    }
                }
            }
        }
    }
    #[allow(dead_code)]
    pub mod shapes {
        #[allow(dead_code, clippy::all)]
        pub mod api {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() =
                super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            pub type Vec2 = super::super::super::wired::math::types::Vec2;
            pub type Vec3 = super::super::super::wired::math::types::Vec3;
            pub type Mesh = super::super::super::wired::scene::mesh::Mesh;
            pub type Node = super::super::super::wired::scene::node::Node;
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct Rectangle {
                handle: _rt::Resource<Rectangle>,
            }
            impl Rectangle {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for Rectangle {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[resource-drop]rectangle"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct Circle {
                handle: _rt::Resource<Circle>,
            }
            impl Circle {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for Circle {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[resource-drop]circle"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct Ellipse {
                handle: _rt::Resource<Ellipse>,
            }
            impl Ellipse {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for Ellipse {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[resource-drop]ellipse"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct Cylinder {
                handle: _rt::Resource<Cylinder>,
            }
            impl Cylinder {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for Cylinder {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[resource-drop]cylinder"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct Cuboid {
                handle: _rt::Resource<Cuboid>,
            }
            impl Cuboid {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for Cuboid {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[resource-drop]cuboid"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            #[repr(C)]
            #[derive(Clone, Copy)]
            pub struct SphereIco {
                pub subdivisions: u8,
            }
            impl ::core::fmt::Debug for SphereIco {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.debug_struct("SphereIco")
                        .field("subdivisions", &self.subdivisions)
                        .finish()
                }
            }
            #[repr(C)]
            #[derive(Clone, Copy)]
            pub struct SphereUv {
                pub sectors: u8,
                pub stacks: u8,
            }
            impl ::core::fmt::Debug for SphereUv {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.debug_struct("SphereUv")
                        .field("sectors", &self.sectors)
                        .field("stacks", &self.stacks)
                        .finish()
                }
            }
            #[derive(Clone, Copy)]
            pub enum SphereKind {
                /// An icosphere, a spherical mesh that consists of similar sized triangles.
                Ico(SphereIco),
                /// A UV sphere, a spherical mesh that consists of quadrilaterals
                Uv(SphereUv),
            }
            impl ::core::fmt::Debug for SphereKind {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        SphereKind::Ico(e) => f.debug_tuple("SphereKind::Ico").field(e).finish(),
                        SphereKind::Uv(e) => f.debug_tuple("SphereKind::Uv").field(e).finish(),
                    }
                }
            }
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct Sphere {
                handle: _rt::Resource<Sphere>,
            }
            impl Sphere {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for Sphere {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[resource-drop]sphere"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct Axes {
                handle: _rt::Resource<Axes>,
            }
            impl Axes {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for Axes {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[resource-drop]axes"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            impl Rectangle {
                #[allow(unused_unsafe, clippy::all)]
                pub fn new(size: Vec2) -> Self {
                    unsafe {
                        let super::super::super::wired::math::types::Vec2 { x: x0, y: y0 } = size;
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[constructor]rectangle"]
                            fn wit_import(_: f32, _: f32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: f32, _: f32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(_rt::as_f32(x0), _rt::as_f32(y0));
                        Rectangle::from_handle(ret as u32)
                    }
                }
            }
            impl Rectangle {
                #[allow(unused_unsafe, clippy::all)]
                pub fn size(&self) -> Vec2 {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]rectangle.size"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<f32>();
                        let l2 = *ptr0.add(4).cast::<f32>();
                        super::super::super::wired::math::types::Vec2 { x: l1, y: l2 }
                    }
                }
            }
            impl Rectangle {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_size(&self, value: Vec2) {
                    unsafe {
                        let super::super::super::wired::math::types::Vec2 { x: x0, y: y0 } = value;
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]rectangle.set-size"]
                            fn wit_import(_: i32, _: f32, _: f32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: f32, _: f32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, _rt::as_f32(x0), _rt::as_f32(y0));
                    }
                }
            }
            impl Rectangle {
                #[allow(unused_unsafe, clippy::all)]
                /// Creates a mesh of this shape.
                pub fn to_mesh(&self) -> Mesh {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]rectangle.to-mesh"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::scene::mesh::Mesh::from_handle(ret as u32)
                    }
                }
            }
            impl Rectangle {
                #[allow(unused_unsafe, clippy::all)]
                /// Creates a node with a mesh of this shape.
                pub fn to_node(&self) -> Node {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]rectangle.to-node"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::scene::node::Node::from_handle(ret as u32)
                    }
                }
            }
            impl Rectangle {
                #[allow(unused_unsafe, clippy::all)]
                /// Creates a node with a mesh and physics collider of this shape.
                pub fn to_physics_node(&self) -> Node {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]rectangle.to-physics-node"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::scene::node::Node::from_handle(ret as u32)
                    }
                }
            }
            impl Circle {
                #[allow(unused_unsafe, clippy::all)]
                pub fn new(radius: f32) -> Self {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[constructor]circle"]
                            fn wit_import(_: f32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: f32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(_rt::as_f32(&radius));
                        Circle::from_handle(ret as u32)
                    }
                }
            }
            impl Circle {
                #[allow(unused_unsafe, clippy::all)]
                pub fn radius(&self) -> f32 {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]circle.radius"]
                            fn wit_import(_: i32) -> f32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> f32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        ret
                    }
                }
            }
            impl Circle {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_radius(&self, value: f32) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]circle.set-radius"]
                            fn wit_import(_: i32, _: f32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: f32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, _rt::as_f32(&value));
                    }
                }
            }
            impl Circle {
                #[allow(unused_unsafe, clippy::all)]
                /// The number of vertices used for the mesh.
                pub fn resolution(&self) -> u16 {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]circle.resolution"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        ret as u16
                    }
                }
            }
            impl Circle {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_resolution(&self, value: u16) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]circle.set-resolution"]
                            fn wit_import(_: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, _rt::as_i32(&value));
                    }
                }
            }
            impl Circle {
                #[allow(unused_unsafe, clippy::all)]
                /// Creates a mesh of this shape.
                pub fn to_mesh(&self) -> Mesh {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]circle.to-mesh"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::scene::mesh::Mesh::from_handle(ret as u32)
                    }
                }
            }
            impl Circle {
                #[allow(unused_unsafe, clippy::all)]
                /// Creates a node with a mesh of this shape.
                pub fn to_node(&self) -> Node {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]circle.to-node"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::scene::node::Node::from_handle(ret as u32)
                    }
                }
            }
            impl Circle {
                #[allow(unused_unsafe, clippy::all)]
                /// Creates a node with a mesh and physics collider of this shape.
                pub fn to_physics_node(&self) -> Node {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]circle.to-physics-node"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::scene::node::Node::from_handle(ret as u32)
                    }
                }
            }
            impl Ellipse {
                #[allow(unused_unsafe, clippy::all)]
                pub fn new(half_size: Vec2) -> Self {
                    unsafe {
                        let super::super::super::wired::math::types::Vec2 { x: x0, y: y0 } =
                            half_size;
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[constructor]ellipse"]
                            fn wit_import(_: f32, _: f32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: f32, _: f32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(_rt::as_f32(x0), _rt::as_f32(y0));
                        Ellipse::from_handle(ret as u32)
                    }
                }
            }
            impl Ellipse {
                #[allow(unused_unsafe, clippy::all)]
                pub fn half_size(&self) -> Vec2 {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]ellipse.half-size"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<f32>();
                        let l2 = *ptr0.add(4).cast::<f32>();
                        super::super::super::wired::math::types::Vec2 { x: l1, y: l2 }
                    }
                }
            }
            impl Ellipse {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_half_size(&self, value: Vec2) {
                    unsafe {
                        let super::super::super::wired::math::types::Vec2 { x: x0, y: y0 } = value;
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]ellipse.set-half-size"]
                            fn wit_import(_: i32, _: f32, _: f32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: f32, _: f32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, _rt::as_f32(x0), _rt::as_f32(y0));
                    }
                }
            }
            impl Ellipse {
                #[allow(unused_unsafe, clippy::all)]
                /// The number of vertices used for the mesh.
                pub fn resolution(&self) -> u16 {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]ellipse.resolution"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        ret as u16
                    }
                }
            }
            impl Ellipse {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_resolution(&self, value: u16) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]ellipse.set-resolution"]
                            fn wit_import(_: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, _rt::as_i32(&value));
                    }
                }
            }
            impl Ellipse {
                #[allow(unused_unsafe, clippy::all)]
                /// Creates a mesh of this shape.
                pub fn to_mesh(&self) -> Mesh {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]ellipse.to-mesh"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::scene::mesh::Mesh::from_handle(ret as u32)
                    }
                }
            }
            impl Ellipse {
                #[allow(unused_unsafe, clippy::all)]
                /// Creates a node with a mesh of this shape.
                pub fn to_node(&self) -> Node {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]ellipse.to-node"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::scene::node::Node::from_handle(ret as u32)
                    }
                }
            }
            impl Ellipse {
                #[allow(unused_unsafe, clippy::all)]
                /// Creates a node with a mesh and physics collider of this shape.
                pub fn to_physics_node(&self) -> Node {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]ellipse.to-physics-node"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::scene::node::Node::from_handle(ret as u32)
                    }
                }
            }
            impl Cylinder {
                #[allow(unused_unsafe, clippy::all)]
                pub fn new(radius: f32, height: f32) -> Self {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[constructor]cylinder"]
                            fn wit_import(_: f32, _: f32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: f32, _: f32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(_rt::as_f32(&radius), _rt::as_f32(&height));
                        Cylinder::from_handle(ret as u32)
                    }
                }
            }
            impl Cylinder {
                #[allow(unused_unsafe, clippy::all)]
                /// Whether to cap the ends of the cylinder.
                pub fn cap(&self) -> bool {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]cylinder.cap"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        _rt::bool_lift(ret as u8)
                    }
                }
            }
            impl Cylinder {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_cap(&self, value: bool) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]cylinder.set-cap"]
                            fn wit_import(_: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import(
                            (self).handle() as i32,
                            match &value {
                                true => 1,
                                false => 0,
                            },
                        );
                    }
                }
            }
            impl Cylinder {
                #[allow(unused_unsafe, clippy::all)]
                pub fn height(&self) -> f32 {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]cylinder.height"]
                            fn wit_import(_: i32) -> f32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> f32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        ret
                    }
                }
            }
            impl Cylinder {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_height(&self, value: f32) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]cylinder.set-height"]
                            fn wit_import(_: i32, _: f32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: f32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, _rt::as_f32(&value));
                    }
                }
            }
            impl Cylinder {
                #[allow(unused_unsafe, clippy::all)]
                pub fn radius(&self) -> f32 {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]cylinder.radius"]
                            fn wit_import(_: i32) -> f32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> f32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        ret
                    }
                }
            }
            impl Cylinder {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_radius(&self, value: f32) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]cylinder.set-radius"]
                            fn wit_import(_: i32, _: f32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: f32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, _rt::as_f32(&value));
                    }
                }
            }
            impl Cylinder {
                #[allow(unused_unsafe, clippy::all)]
                /// The number of vertices used for the top and bottom of the cylinder.
                pub fn resolution(&self) -> u8 {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]cylinder.resolution"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        ret as u8
                    }
                }
            }
            impl Cylinder {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_resolution(&self, value: u8) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]cylinder.set-resolution"]
                            fn wit_import(_: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, _rt::as_i32(&value));
                    }
                }
            }
            impl Cylinder {
                #[allow(unused_unsafe, clippy::all)]
                /// The number of segments along the height of the cylinder.
                pub fn segments(&self) -> u8 {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]cylinder.segments"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        ret as u8
                    }
                }
            }
            impl Cylinder {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_segments(&self, value: u8) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]cylinder.set-segments"]
                            fn wit_import(_: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, _rt::as_i32(&value));
                    }
                }
            }
            impl Cylinder {
                #[allow(unused_unsafe, clippy::all)]
                /// Creates a mesh of this shape.
                pub fn to_mesh(&self) -> Mesh {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]cylinder.to-mesh"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::scene::mesh::Mesh::from_handle(ret as u32)
                    }
                }
            }
            impl Cylinder {
                #[allow(unused_unsafe, clippy::all)]
                /// Creates a node with a mesh of this shape.
                pub fn to_node(&self) -> Node {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]cylinder.to-node"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::scene::node::Node::from_handle(ret as u32)
                    }
                }
            }
            impl Cylinder {
                #[allow(unused_unsafe, clippy::all)]
                /// Creates a node with a mesh and physics collider of this shape.
                pub fn to_physics_node(&self) -> Node {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]cylinder.to-physics-node"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::scene::node::Node::from_handle(ret as u32)
                    }
                }
            }
            impl Cuboid {
                #[allow(unused_unsafe, clippy::all)]
                pub fn new(size: Vec3) -> Self {
                    unsafe {
                        let super::super::super::wired::math::types::Vec3 {
                            x: x0,
                            y: y0,
                            z: z0,
                        } = size;
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[constructor]cuboid"]
                            fn wit_import(_: f32, _: f32, _: f32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: f32, _: f32, _: f32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(_rt::as_f32(x0), _rt::as_f32(y0), _rt::as_f32(z0));
                        Cuboid::from_handle(ret as u32)
                    }
                }
            }
            impl Cuboid {
                #[allow(unused_unsafe, clippy::all)]
                pub fn size(&self) -> Vec3 {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 12]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]cuboid.size"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<f32>();
                        let l2 = *ptr0.add(4).cast::<f32>();
                        let l3 = *ptr0.add(8).cast::<f32>();
                        super::super::super::wired::math::types::Vec3 {
                            x: l1,
                            y: l2,
                            z: l3,
                        }
                    }
                }
            }
            impl Cuboid {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_size(&self, value: Vec3) {
                    unsafe {
                        let super::super::super::wired::math::types::Vec3 {
                            x: x0,
                            y: y0,
                            z: z0,
                        } = value;
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]cuboid.set-size"]
                            fn wit_import(_: i32, _: f32, _: f32, _: f32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: f32, _: f32, _: f32) {
                            unreachable!()
                        }
                        wit_import(
                            (self).handle() as i32,
                            _rt::as_f32(x0),
                            _rt::as_f32(y0),
                            _rt::as_f32(z0),
                        );
                    }
                }
            }
            impl Cuboid {
                #[allow(unused_unsafe, clippy::all)]
                /// Creates a mesh of this shape.
                pub fn to_mesh(&self) -> Mesh {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]cuboid.to-mesh"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::scene::mesh::Mesh::from_handle(ret as u32)
                    }
                }
            }
            impl Cuboid {
                #[allow(unused_unsafe, clippy::all)]
                /// Creates a node with a mesh of this shape.
                pub fn to_node(&self) -> Node {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]cuboid.to-node"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::scene::node::Node::from_handle(ret as u32)
                    }
                }
            }
            impl Cuboid {
                #[allow(unused_unsafe, clippy::all)]
                /// Creates a node with a mesh and physics collider of this shape.
                pub fn to_physics_node(&self) -> Node {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]cuboid.to-physics-node"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::scene::node::Node::from_handle(ret as u32)
                    }
                }
            }
            impl Sphere {
                #[allow(unused_unsafe, clippy::all)]
                pub fn new_ico(radius: f32) -> Sphere {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[static]sphere.new-ico"]
                            fn wit_import(_: f32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: f32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(_rt::as_f32(&radius));
                        Sphere::from_handle(ret as u32)
                    }
                }
            }
            impl Sphere {
                #[allow(unused_unsafe, clippy::all)]
                pub fn new_uv(radius: f32) -> Sphere {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[static]sphere.new-uv"]
                            fn wit_import(_: f32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: f32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(_rt::as_f32(&radius));
                        Sphere::from_handle(ret as u32)
                    }
                }
            }
            impl Sphere {
                #[allow(unused_unsafe, clippy::all)]
                pub fn radius(&self) -> f32 {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]sphere.radius"]
                            fn wit_import(_: i32) -> f32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> f32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        ret
                    }
                }
            }
            impl Sphere {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_radius(&self, value: f32) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]sphere.set-radius"]
                            fn wit_import(_: i32, _: f32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: f32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, _rt::as_f32(&value));
                    }
                }
            }
            impl Sphere {
                #[allow(unused_unsafe, clippy::all)]
                pub fn kind(&self) -> SphereKind {
                    unsafe {
                        #[repr(align(1))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 3]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 3]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]sphere.kind"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        let v5 = match l1 {
                            0 => {
                                let e5 = {
                                    let l2 = i32::from(*ptr0.add(1).cast::<u8>());
                                    SphereIco {
                                        subdivisions: l2 as u8,
                                    }
                                };
                                SphereKind::Ico(e5)
                            }
                            n => {
                                debug_assert_eq!(n, 1, "invalid enum discriminant");
                                let e5 = {
                                    let l3 = i32::from(*ptr0.add(1).cast::<u8>());
                                    let l4 = i32::from(*ptr0.add(2).cast::<u8>());
                                    SphereUv {
                                        sectors: l3 as u8,
                                        stacks: l4 as u8,
                                    }
                                };
                                SphereKind::Uv(e5)
                            }
                        };
                        v5
                    }
                }
            }
            impl Sphere {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_kind(&self, value: SphereKind) {
                    unsafe {
                        let (result2_0, result2_1, result2_2) = match value {
                            SphereKind::Ico(e) => {
                                let SphereIco {
                                    subdivisions: subdivisions0,
                                } = e;
                                (0i32, _rt::as_i32(subdivisions0), 0i32)
                            }
                            SphereKind::Uv(e) => {
                                let SphereUv {
                                    sectors: sectors1,
                                    stacks: stacks1,
                                } = e;
                                (1i32, _rt::as_i32(sectors1), _rt::as_i32(stacks1))
                            }
                        };
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]sphere.set-kind"]
                            fn wit_import(_: i32, _: i32, _: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32, _: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, result2_0, result2_1, result2_2);
                    }
                }
            }
            impl Sphere {
                #[allow(unused_unsafe, clippy::all)]
                /// Creates a mesh of this shape.
                pub fn to_mesh(&self) -> Mesh {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]sphere.to-mesh"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::scene::mesh::Mesh::from_handle(ret as u32)
                    }
                }
            }
            impl Sphere {
                #[allow(unused_unsafe, clippy::all)]
                /// Creates a node with a mesh of this shape.
                pub fn to_node(&self) -> Node {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]sphere.to-node"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::scene::node::Node::from_handle(ret as u32)
                    }
                }
            }
            impl Sphere {
                #[allow(unused_unsafe, clippy::all)]
                /// Creates a node with a mesh and physics collider of this shape.
                pub fn to_physics_node(&self) -> Node {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]sphere.to-physics-node"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::scene::node::Node::from_handle(ret as u32)
                    }
                }
            }
            impl Axes {
                #[allow(unused_unsafe, clippy::all)]
                pub fn new() -> Self {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[constructor]axes"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        Axes::from_handle(ret as u32)
                    }
                }
            }
            impl Axes {
                #[allow(unused_unsafe, clippy::all)]
                pub fn size(&self) -> f32 {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]axes.size"]
                            fn wit_import(_: i32) -> f32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> f32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        ret
                    }
                }
            }
            impl Axes {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_size(&self, value: f32) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]axes.set-size"]
                            fn wit_import(_: i32, _: f32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: f32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, _rt::as_f32(&value));
                    }
                }
            }
            impl Axes {
                #[allow(unused_unsafe, clippy::all)]
                /// Creates a node with a mesh of this shape.
                pub fn to_node(&self) -> Node {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "unavi:shapes/api")]
                        extern "C" {
                            #[link_name = "[method]axes.to-node"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::scene::node::Node::from_handle(ret as u32)
                    }
                }
            }
        }
    }
}
#[allow(dead_code)]
pub mod wired {
    #[allow(dead_code)]
    pub mod input {
        #[allow(dead_code, clippy::all)]
        pub mod types {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() =
                super::super::super::__link_custom_section_describing_imports;
            pub type Vec3 = super::super::super::wired::math::types::Vec3;
            pub type Quat = super::super::super::wired::math::types::Quat;
            #[repr(u8)]
            #[derive(Clone, Copy, Eq, PartialEq)]
            pub enum HandSide {
                Left,
                Right,
            }
            impl ::core::fmt::Debug for HandSide {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        HandSide::Left => f.debug_tuple("HandSide::Left").finish(),
                        HandSide::Right => f.debug_tuple("HandSide::Right").finish(),
                    }
                }
            }
            impl HandSide {
                #[doc(hidden)]
                pub unsafe fn _lift(val: u8) -> HandSide {
                    if !cfg!(debug_assertions) {
                        return ::core::mem::transmute(val);
                    }
                    match val {
                        0 => HandSide::Left,
                        1 => HandSide::Right,
                        _ => panic!("invalid enum discriminant"),
                    }
                }
            }
            #[repr(C)]
            #[derive(Clone, Copy)]
            pub struct Joint {
                pub translation: Vec3,
                pub rotation: Quat,
                pub radius: f32,
            }
            impl ::core::fmt::Debug for Joint {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.debug_struct("Joint")
                        .field("translation", &self.translation)
                        .field("rotation", &self.rotation)
                        .field("radius", &self.radius)
                        .finish()
                }
            }
            #[repr(C)]
            #[derive(Clone, Copy)]
            pub struct Finger {
                pub tip: Joint,
                pub distal: Joint,
                pub proximal: Joint,
                pub metacarpal: Joint,
            }
            impl ::core::fmt::Debug for Finger {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.debug_struct("Finger")
                        .field("tip", &self.tip)
                        .field("distal", &self.distal)
                        .field("proximal", &self.proximal)
                        .field("metacarpal", &self.metacarpal)
                        .finish()
                }
            }
            /// Hand tracking data.
            #[repr(C)]
            #[derive(Clone, Copy)]
            pub struct Hand {
                pub side: HandSide,
                pub thumb: Finger,
                pub index: Finger,
                pub middle: Finger,
                pub ring: Finger,
                pub little: Finger,
                pub palm: Joint,
                pub wrist: Joint,
                pub elbow: Option<Joint>,
            }
            impl ::core::fmt::Debug for Hand {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.debug_struct("Hand")
                        .field("side", &self.side)
                        .field("thumb", &self.thumb)
                        .field("index", &self.index)
                        .field("middle", &self.middle)
                        .field("ring", &self.ring)
                        .field("little", &self.little)
                        .field("palm", &self.palm)
                        .field("wrist", &self.wrist)
                        .field("elbow", &self.elbow)
                        .finish()
                }
            }
            /// A line with an origin and a direction.
            #[repr(C)]
            #[derive(Clone, Copy)]
            pub struct Ray {
                pub orientation: Quat,
                pub origin: Vec3,
            }
            impl ::core::fmt::Debug for Ray {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.debug_struct("Ray")
                        .field("orientation", &self.orientation)
                        .field("origin", &self.origin)
                        .finish()
                }
            }
            #[derive(Clone, Copy)]
            pub enum InputData {
                Hand(Hand),
                Ray(Ray),
            }
            impl ::core::fmt::Debug for InputData {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        InputData::Hand(e) => f.debug_tuple("InputData::Hand").field(e).finish(),
                        InputData::Ray(e) => f.debug_tuple("InputData::Ray").field(e).finish(),
                    }
                }
            }
            #[derive(Clone, Copy)]
            pub enum InputAction {
                Collision,
                Hover,
            }
            impl ::core::fmt::Debug for InputAction {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        InputAction::Collision => f.debug_tuple("InputAction::Collision").finish(),
                        InputAction::Hover => f.debug_tuple("InputAction::Hover").finish(),
                    }
                }
            }
            #[repr(C)]
            #[derive(Clone, Copy)]
            pub struct InputEvent {
                /// Unique id for the event.
                pub id: u64,
                /// The action that created the event.
                pub action: InputAction,
                /// Spatial input data.
                pub data: InputData,
            }
            impl ::core::fmt::Debug for InputEvent {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.debug_struct("InputEvent")
                        .field("id", &self.id)
                        .field("action", &self.action)
                        .field("data", &self.data)
                        .finish()
                }
            }
        }
        #[allow(dead_code, clippy::all)]
        pub mod handler {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() =
                super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            pub type InputEvent = super::super::super::wired::input::types::InputEvent;
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct InputHandler {
                handle: _rt::Resource<InputHandler>,
            }
            impl InputHandler {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for InputHandler {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "wired:input/handler")]
                        extern "C" {
                            #[link_name = "[resource-drop]input-handler"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            impl InputHandler {
                #[allow(unused_unsafe, clippy::all)]
                pub fn new() -> Self {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:input/handler")]
                        extern "C" {
                            #[link_name = "[constructor]input-handler"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        InputHandler::from_handle(ret as u32)
                    }
                }
            }
            impl InputHandler {
                #[allow(unused_unsafe, clippy::all)]
                /// Handle the next recieved input event.
                /// Events only last for one tick.
                pub fn next(&self) -> Option<InputEvent> {
                    unsafe {
                        #[repr(align(8))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 768]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 768]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:input/handler")]
                        extern "C" {
                            #[link_name = "[method]input-handler.next"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(8).cast::<i64>();
                                    let l3 = i32::from(*ptr0.add(16).cast::<u8>());
                                    use super::super::super::wired::input::types::InputAction as V4;
                                    let v4 = match l3 {
                                        0 => V4::Collision,
                                        n => {
                                            debug_assert_eq!(n, 1, "invalid enum discriminant");
                                            V4::Hover
                                        }
                                    };
                                    let l5 = i32::from(*ptr0.add(20).cast::<u8>());
                                    use super::super::super::wired::input::types::InputData as V199;
                                    let v199 = match l5 {
                                        0 => {
                                            let e199 = {
                                                let l6 = i32::from(*ptr0.add(24).cast::<u8>());
                                                let l7 = *ptr0.add(28).cast::<f32>();
                                                let l8 = *ptr0.add(32).cast::<f32>();
                                                let l9 = *ptr0.add(36).cast::<f32>();
                                                let l10 = *ptr0.add(40).cast::<f32>();
                                                let l11 = *ptr0.add(44).cast::<f32>();
                                                let l12 = *ptr0.add(48).cast::<f32>();
                                                let l13 = *ptr0.add(52).cast::<f32>();
                                                let l14 = *ptr0.add(56).cast::<f32>();
                                                let l15 = *ptr0.add(60).cast::<f32>();
                                                let l16 = *ptr0.add(64).cast::<f32>();
                                                let l17 = *ptr0.add(68).cast::<f32>();
                                                let l18 = *ptr0.add(72).cast::<f32>();
                                                let l19 = *ptr0.add(76).cast::<f32>();
                                                let l20 = *ptr0.add(80).cast::<f32>();
                                                let l21 = *ptr0.add(84).cast::<f32>();
                                                let l22 = *ptr0.add(88).cast::<f32>();
                                                let l23 = *ptr0.add(92).cast::<f32>();
                                                let l24 = *ptr0.add(96).cast::<f32>();
                                                let l25 = *ptr0.add(100).cast::<f32>();
                                                let l26 = *ptr0.add(104).cast::<f32>();
                                                let l27 = *ptr0.add(108).cast::<f32>();
                                                let l28 = *ptr0.add(112).cast::<f32>();
                                                let l29 = *ptr0.add(116).cast::<f32>();
                                                let l30 = *ptr0.add(120).cast::<f32>();
                                                let l31 = *ptr0.add(124).cast::<f32>();
                                                let l32 = *ptr0.add(128).cast::<f32>();
                                                let l33 = *ptr0.add(132).cast::<f32>();
                                                let l34 = *ptr0.add(136).cast::<f32>();
                                                let l35 = *ptr0.add(140).cast::<f32>();
                                                let l36 = *ptr0.add(144).cast::<f32>();
                                                let l37 = *ptr0.add(148).cast::<f32>();
                                                let l38 = *ptr0.add(152).cast::<f32>();
                                                let l39 = *ptr0.add(156).cast::<f32>();
                                                let l40 = *ptr0.add(160).cast::<f32>();
                                                let l41 = *ptr0.add(164).cast::<f32>();
                                                let l42 = *ptr0.add(168).cast::<f32>();
                                                let l43 = *ptr0.add(172).cast::<f32>();
                                                let l44 = *ptr0.add(176).cast::<f32>();
                                                let l45 = *ptr0.add(180).cast::<f32>();
                                                let l46 = *ptr0.add(184).cast::<f32>();
                                                let l47 = *ptr0.add(188).cast::<f32>();
                                                let l48 = *ptr0.add(192).cast::<f32>();
                                                let l49 = *ptr0.add(196).cast::<f32>();
                                                let l50 = *ptr0.add(200).cast::<f32>();
                                                let l51 = *ptr0.add(204).cast::<f32>();
                                                let l52 = *ptr0.add(208).cast::<f32>();
                                                let l53 = *ptr0.add(212).cast::<f32>();
                                                let l54 = *ptr0.add(216).cast::<f32>();
                                                let l55 = *ptr0.add(220).cast::<f32>();
                                                let l56 = *ptr0.add(224).cast::<f32>();
                                                let l57 = *ptr0.add(228).cast::<f32>();
                                                let l58 = *ptr0.add(232).cast::<f32>();
                                                let l59 = *ptr0.add(236).cast::<f32>();
                                                let l60 = *ptr0.add(240).cast::<f32>();
                                                let l61 = *ptr0.add(244).cast::<f32>();
                                                let l62 = *ptr0.add(248).cast::<f32>();
                                                let l63 = *ptr0.add(252).cast::<f32>();
                                                let l64 = *ptr0.add(256).cast::<f32>();
                                                let l65 = *ptr0.add(260).cast::<f32>();
                                                let l66 = *ptr0.add(264).cast::<f32>();
                                                let l67 = *ptr0.add(268).cast::<f32>();
                                                let l68 = *ptr0.add(272).cast::<f32>();
                                                let l69 = *ptr0.add(276).cast::<f32>();
                                                let l70 = *ptr0.add(280).cast::<f32>();
                                                let l71 = *ptr0.add(284).cast::<f32>();
                                                let l72 = *ptr0.add(288).cast::<f32>();
                                                let l73 = *ptr0.add(292).cast::<f32>();
                                                let l74 = *ptr0.add(296).cast::<f32>();
                                                let l75 = *ptr0.add(300).cast::<f32>();
                                                let l76 = *ptr0.add(304).cast::<f32>();
                                                let l77 = *ptr0.add(308).cast::<f32>();
                                                let l78 = *ptr0.add(312).cast::<f32>();
                                                let l79 = *ptr0.add(316).cast::<f32>();
                                                let l80 = *ptr0.add(320).cast::<f32>();
                                                let l81 = *ptr0.add(324).cast::<f32>();
                                                let l82 = *ptr0.add(328).cast::<f32>();
                                                let l83 = *ptr0.add(332).cast::<f32>();
                                                let l84 = *ptr0.add(336).cast::<f32>();
                                                let l85 = *ptr0.add(340).cast::<f32>();
                                                let l86 = *ptr0.add(344).cast::<f32>();
                                                let l87 = *ptr0.add(348).cast::<f32>();
                                                let l88 = *ptr0.add(352).cast::<f32>();
                                                let l89 = *ptr0.add(356).cast::<f32>();
                                                let l90 = *ptr0.add(360).cast::<f32>();
                                                let l91 = *ptr0.add(364).cast::<f32>();
                                                let l92 = *ptr0.add(368).cast::<f32>();
                                                let l93 = *ptr0.add(372).cast::<f32>();
                                                let l94 = *ptr0.add(376).cast::<f32>();
                                                let l95 = *ptr0.add(380).cast::<f32>();
                                                let l96 = *ptr0.add(384).cast::<f32>();
                                                let l97 = *ptr0.add(388).cast::<f32>();
                                                let l98 = *ptr0.add(392).cast::<f32>();
                                                let l99 = *ptr0.add(396).cast::<f32>();
                                                let l100 = *ptr0.add(400).cast::<f32>();
                                                let l101 = *ptr0.add(404).cast::<f32>();
                                                let l102 = *ptr0.add(408).cast::<f32>();
                                                let l103 = *ptr0.add(412).cast::<f32>();
                                                let l104 = *ptr0.add(416).cast::<f32>();
                                                let l105 = *ptr0.add(420).cast::<f32>();
                                                let l106 = *ptr0.add(424).cast::<f32>();
                                                let l107 = *ptr0.add(428).cast::<f32>();
                                                let l108 = *ptr0.add(432).cast::<f32>();
                                                let l109 = *ptr0.add(436).cast::<f32>();
                                                let l110 = *ptr0.add(440).cast::<f32>();
                                                let l111 = *ptr0.add(444).cast::<f32>();
                                                let l112 = *ptr0.add(448).cast::<f32>();
                                                let l113 = *ptr0.add(452).cast::<f32>();
                                                let l114 = *ptr0.add(456).cast::<f32>();
                                                let l115 = *ptr0.add(460).cast::<f32>();
                                                let l116 = *ptr0.add(464).cast::<f32>();
                                                let l117 = *ptr0.add(468).cast::<f32>();
                                                let l118 = *ptr0.add(472).cast::<f32>();
                                                let l119 = *ptr0.add(476).cast::<f32>();
                                                let l120 = *ptr0.add(480).cast::<f32>();
                                                let l121 = *ptr0.add(484).cast::<f32>();
                                                let l122 = *ptr0.add(488).cast::<f32>();
                                                let l123 = *ptr0.add(492).cast::<f32>();
                                                let l124 = *ptr0.add(496).cast::<f32>();
                                                let l125 = *ptr0.add(500).cast::<f32>();
                                                let l126 = *ptr0.add(504).cast::<f32>();
                                                let l127 = *ptr0.add(508).cast::<f32>();
                                                let l128 = *ptr0.add(512).cast::<f32>();
                                                let l129 = *ptr0.add(516).cast::<f32>();
                                                let l130 = *ptr0.add(520).cast::<f32>();
                                                let l131 = *ptr0.add(524).cast::<f32>();
                                                let l132 = *ptr0.add(528).cast::<f32>();
                                                let l133 = *ptr0.add(532).cast::<f32>();
                                                let l134 = *ptr0.add(536).cast::<f32>();
                                                let l135 = *ptr0.add(540).cast::<f32>();
                                                let l136 = *ptr0.add(544).cast::<f32>();
                                                let l137 = *ptr0.add(548).cast::<f32>();
                                                let l138 = *ptr0.add(552).cast::<f32>();
                                                let l139 = *ptr0.add(556).cast::<f32>();
                                                let l140 = *ptr0.add(560).cast::<f32>();
                                                let l141 = *ptr0.add(564).cast::<f32>();
                                                let l142 = *ptr0.add(568).cast::<f32>();
                                                let l143 = *ptr0.add(572).cast::<f32>();
                                                let l144 = *ptr0.add(576).cast::<f32>();
                                                let l145 = *ptr0.add(580).cast::<f32>();
                                                let l146 = *ptr0.add(584).cast::<f32>();
                                                let l147 = *ptr0.add(588).cast::<f32>();
                                                let l148 = *ptr0.add(592).cast::<f32>();
                                                let l149 = *ptr0.add(596).cast::<f32>();
                                                let l150 = *ptr0.add(600).cast::<f32>();
                                                let l151 = *ptr0.add(604).cast::<f32>();
                                                let l152 = *ptr0.add(608).cast::<f32>();
                                                let l153 = *ptr0.add(612).cast::<f32>();
                                                let l154 = *ptr0.add(616).cast::<f32>();
                                                let l155 = *ptr0.add(620).cast::<f32>();
                                                let l156 = *ptr0.add(624).cast::<f32>();
                                                let l157 = *ptr0.add(628).cast::<f32>();
                                                let l158 = *ptr0.add(632).cast::<f32>();
                                                let l159 = *ptr0.add(636).cast::<f32>();
                                                let l160 = *ptr0.add(640).cast::<f32>();
                                                let l161 = *ptr0.add(644).cast::<f32>();
                                                let l162 = *ptr0.add(648).cast::<f32>();
                                                let l163 = *ptr0.add(652).cast::<f32>();
                                                let l164 = *ptr0.add(656).cast::<f32>();
                                                let l165 = *ptr0.add(660).cast::<f32>();
                                                let l166 = *ptr0.add(664).cast::<f32>();
                                                let l167 = *ptr0.add(668).cast::<f32>();
                                                let l168 = *ptr0.add(672).cast::<f32>();
                                                let l169 = *ptr0.add(676).cast::<f32>();
                                                let l170 = *ptr0.add(680).cast::<f32>();
                                                let l171 = *ptr0.add(684).cast::<f32>();
                                                let l172 = *ptr0.add(688).cast::<f32>();
                                                let l173 = *ptr0.add(692).cast::<f32>();
                                                let l174 = *ptr0.add(696).cast::<f32>();
                                                let l175 = *ptr0.add(700).cast::<f32>();
                                                let l176 = *ptr0.add(704).cast::<f32>();
                                                let l177 = *ptr0.add(708).cast::<f32>();
                                                let l178 = *ptr0.add(712).cast::<f32>();
                                                let l179 = *ptr0.add(716).cast::<f32>();
                                                let l180 = *ptr0.add(720).cast::<f32>();
                                                let l181 = *ptr0.add(724).cast::<f32>();
                                                let l182 = *ptr0.add(728).cast::<f32>();
                                                let l183 = i32::from(*ptr0.add(732).cast::<u8>());
                                                super::super::super::wired::input::types::Hand {
                                                    side: super::super::super::wired::input::types::HandSide::_lift(
                                                        l6 as u8,
                                                    ),
                                                    thumb: super::super::super::wired::input::types::Finger {
                                                        tip: super::super::super::wired::input::types::Joint {
                                                            translation: super::super::super::wired::math::types::Vec3 {
                                                                x: l7,
                                                                y: l8,
                                                                z: l9,
                                                            },
                                                            rotation: super::super::super::wired::math::types::Quat {
                                                                x: l10,
                                                                y: l11,
                                                                z: l12,
                                                                w: l13,
                                                            },
                                                            radius: l14,
                                                        },
                                                        distal: super::super::super::wired::input::types::Joint {
                                                            translation: super::super::super::wired::math::types::Vec3 {
                                                                x: l15,
                                                                y: l16,
                                                                z: l17,
                                                            },
                                                            rotation: super::super::super::wired::math::types::Quat {
                                                                x: l18,
                                                                y: l19,
                                                                z: l20,
                                                                w: l21,
                                                            },
                                                            radius: l22,
                                                        },
                                                        proximal: super::super::super::wired::input::types::Joint {
                                                            translation: super::super::super::wired::math::types::Vec3 {
                                                                x: l23,
                                                                y: l24,
                                                                z: l25,
                                                            },
                                                            rotation: super::super::super::wired::math::types::Quat {
                                                                x: l26,
                                                                y: l27,
                                                                z: l28,
                                                                w: l29,
                                                            },
                                                            radius: l30,
                                                        },
                                                        metacarpal: super::super::super::wired::input::types::Joint {
                                                            translation: super::super::super::wired::math::types::Vec3 {
                                                                x: l31,
                                                                y: l32,
                                                                z: l33,
                                                            },
                                                            rotation: super::super::super::wired::math::types::Quat {
                                                                x: l34,
                                                                y: l35,
                                                                z: l36,
                                                                w: l37,
                                                            },
                                                            radius: l38,
                                                        },
                                                    },
                                                    index: super::super::super::wired::input::types::Finger {
                                                        tip: super::super::super::wired::input::types::Joint {
                                                            translation: super::super::super::wired::math::types::Vec3 {
                                                                x: l39,
                                                                y: l40,
                                                                z: l41,
                                                            },
                                                            rotation: super::super::super::wired::math::types::Quat {
                                                                x: l42,
                                                                y: l43,
                                                                z: l44,
                                                                w: l45,
                                                            },
                                                            radius: l46,
                                                        },
                                                        distal: super::super::super::wired::input::types::Joint {
                                                            translation: super::super::super::wired::math::types::Vec3 {
                                                                x: l47,
                                                                y: l48,
                                                                z: l49,
                                                            },
                                                            rotation: super::super::super::wired::math::types::Quat {
                                                                x: l50,
                                                                y: l51,
                                                                z: l52,
                                                                w: l53,
                                                            },
                                                            radius: l54,
                                                        },
                                                        proximal: super::super::super::wired::input::types::Joint {
                                                            translation: super::super::super::wired::math::types::Vec3 {
                                                                x: l55,
                                                                y: l56,
                                                                z: l57,
                                                            },
                                                            rotation: super::super::super::wired::math::types::Quat {
                                                                x: l58,
                                                                y: l59,
                                                                z: l60,
                                                                w: l61,
                                                            },
                                                            radius: l62,
                                                        },
                                                        metacarpal: super::super::super::wired::input::types::Joint {
                                                            translation: super::super::super::wired::math::types::Vec3 {
                                                                x: l63,
                                                                y: l64,
                                                                z: l65,
                                                            },
                                                            rotation: super::super::super::wired::math::types::Quat {
                                                                x: l66,
                                                                y: l67,
                                                                z: l68,
                                                                w: l69,
                                                            },
                                                            radius: l70,
                                                        },
                                                    },
                                                    middle: super::super::super::wired::input::types::Finger {
                                                        tip: super::super::super::wired::input::types::Joint {
                                                            translation: super::super::super::wired::math::types::Vec3 {
                                                                x: l71,
                                                                y: l72,
                                                                z: l73,
                                                            },
                                                            rotation: super::super::super::wired::math::types::Quat {
                                                                x: l74,
                                                                y: l75,
                                                                z: l76,
                                                                w: l77,
                                                            },
                                                            radius: l78,
                                                        },
                                                        distal: super::super::super::wired::input::types::Joint {
                                                            translation: super::super::super::wired::math::types::Vec3 {
                                                                x: l79,
                                                                y: l80,
                                                                z: l81,
                                                            },
                                                            rotation: super::super::super::wired::math::types::Quat {
                                                                x: l82,
                                                                y: l83,
                                                                z: l84,
                                                                w: l85,
                                                            },
                                                            radius: l86,
                                                        },
                                                        proximal: super::super::super::wired::input::types::Joint {
                                                            translation: super::super::super::wired::math::types::Vec3 {
                                                                x: l87,
                                                                y: l88,
                                                                z: l89,
                                                            },
                                                            rotation: super::super::super::wired::math::types::Quat {
                                                                x: l90,
                                                                y: l91,
                                                                z: l92,
                                                                w: l93,
                                                            },
                                                            radius: l94,
                                                        },
                                                        metacarpal: super::super::super::wired::input::types::Joint {
                                                            translation: super::super::super::wired::math::types::Vec3 {
                                                                x: l95,
                                                                y: l96,
                                                                z: l97,
                                                            },
                                                            rotation: super::super::super::wired::math::types::Quat {
                                                                x: l98,
                                                                y: l99,
                                                                z: l100,
                                                                w: l101,
                                                            },
                                                            radius: l102,
                                                        },
                                                    },
                                                    ring: super::super::super::wired::input::types::Finger {
                                                        tip: super::super::super::wired::input::types::Joint {
                                                            translation: super::super::super::wired::math::types::Vec3 {
                                                                x: l103,
                                                                y: l104,
                                                                z: l105,
                                                            },
                                                            rotation: super::super::super::wired::math::types::Quat {
                                                                x: l106,
                                                                y: l107,
                                                                z: l108,
                                                                w: l109,
                                                            },
                                                            radius: l110,
                                                        },
                                                        distal: super::super::super::wired::input::types::Joint {
                                                            translation: super::super::super::wired::math::types::Vec3 {
                                                                x: l111,
                                                                y: l112,
                                                                z: l113,
                                                            },
                                                            rotation: super::super::super::wired::math::types::Quat {
                                                                x: l114,
                                                                y: l115,
                                                                z: l116,
                                                                w: l117,
                                                            },
                                                            radius: l118,
                                                        },
                                                        proximal: super::super::super::wired::input::types::Joint {
                                                            translation: super::super::super::wired::math::types::Vec3 {
                                                                x: l119,
                                                                y: l120,
                                                                z: l121,
                                                            },
                                                            rotation: super::super::super::wired::math::types::Quat {
                                                                x: l122,
                                                                y: l123,
                                                                z: l124,
                                                                w: l125,
                                                            },
                                                            radius: l126,
                                                        },
                                                        metacarpal: super::super::super::wired::input::types::Joint {
                                                            translation: super::super::super::wired::math::types::Vec3 {
                                                                x: l127,
                                                                y: l128,
                                                                z: l129,
                                                            },
                                                            rotation: super::super::super::wired::math::types::Quat {
                                                                x: l130,
                                                                y: l131,
                                                                z: l132,
                                                                w: l133,
                                                            },
                                                            radius: l134,
                                                        },
                                                    },
                                                    little: super::super::super::wired::input::types::Finger {
                                                        tip: super::super::super::wired::input::types::Joint {
                                                            translation: super::super::super::wired::math::types::Vec3 {
                                                                x: l135,
                                                                y: l136,
                                                                z: l137,
                                                            },
                                                            rotation: super::super::super::wired::math::types::Quat {
                                                                x: l138,
                                                                y: l139,
                                                                z: l140,
                                                                w: l141,
                                                            },
                                                            radius: l142,
                                                        },
                                                        distal: super::super::super::wired::input::types::Joint {
                                                            translation: super::super::super::wired::math::types::Vec3 {
                                                                x: l143,
                                                                y: l144,
                                                                z: l145,
                                                            },
                                                            rotation: super::super::super::wired::math::types::Quat {
                                                                x: l146,
                                                                y: l147,
                                                                z: l148,
                                                                w: l149,
                                                            },
                                                            radius: l150,
                                                        },
                                                        proximal: super::super::super::wired::input::types::Joint {
                                                            translation: super::super::super::wired::math::types::Vec3 {
                                                                x: l151,
                                                                y: l152,
                                                                z: l153,
                                                            },
                                                            rotation: super::super::super::wired::math::types::Quat {
                                                                x: l154,
                                                                y: l155,
                                                                z: l156,
                                                                w: l157,
                                                            },
                                                            radius: l158,
                                                        },
                                                        metacarpal: super::super::super::wired::input::types::Joint {
                                                            translation: super::super::super::wired::math::types::Vec3 {
                                                                x: l159,
                                                                y: l160,
                                                                z: l161,
                                                            },
                                                            rotation: super::super::super::wired::math::types::Quat {
                                                                x: l162,
                                                                y: l163,
                                                                z: l164,
                                                                w: l165,
                                                            },
                                                            radius: l166,
                                                        },
                                                    },
                                                    palm: super::super::super::wired::input::types::Joint {
                                                        translation: super::super::super::wired::math::types::Vec3 {
                                                            x: l167,
                                                            y: l168,
                                                            z: l169,
                                                        },
                                                        rotation: super::super::super::wired::math::types::Quat {
                                                            x: l170,
                                                            y: l171,
                                                            z: l172,
                                                            w: l173,
                                                        },
                                                        radius: l174,
                                                    },
                                                    wrist: super::super::super::wired::input::types::Joint {
                                                        translation: super::super::super::wired::math::types::Vec3 {
                                                            x: l175,
                                                            y: l176,
                                                            z: l177,
                                                        },
                                                        rotation: super::super::super::wired::math::types::Quat {
                                                            x: l178,
                                                            y: l179,
                                                            z: l180,
                                                            w: l181,
                                                        },
                                                        radius: l182,
                                                    },
                                                    elbow: match l183 {
                                                        0 => None,
                                                        1 => {
                                                            let e = {
                                                                let l184 = *ptr0.add(736).cast::<f32>();
                                                                let l185 = *ptr0.add(740).cast::<f32>();
                                                                let l186 = *ptr0.add(744).cast::<f32>();
                                                                let l187 = *ptr0.add(748).cast::<f32>();
                                                                let l188 = *ptr0.add(752).cast::<f32>();
                                                                let l189 = *ptr0.add(756).cast::<f32>();
                                                                let l190 = *ptr0.add(760).cast::<f32>();
                                                                let l191 = *ptr0.add(764).cast::<f32>();
                                                                super::super::super::wired::input::types::Joint {
                                                                    translation: super::super::super::wired::math::types::Vec3 {
                                                                        x: l184,
                                                                        y: l185,
                                                                        z: l186,
                                                                    },
                                                                    rotation: super::super::super::wired::math::types::Quat {
                                                                        x: l187,
                                                                        y: l188,
                                                                        z: l189,
                                                                        w: l190,
                                                                    },
                                                                    radius: l191,
                                                                }
                                                            };
                                                            Some(e)
                                                        }
                                                        _ => _rt::invalid_enum_discriminant(),
                                                    },
                                                }
                                            };
                                            V199::Hand(e199)
                                        }
                                        n => {
                                            debug_assert_eq!(n, 1, "invalid enum discriminant");
                                            let e199 = {
                                                let l192 = *ptr0.add(24).cast::<f32>();
                                                let l193 = *ptr0.add(28).cast::<f32>();
                                                let l194 = *ptr0.add(32).cast::<f32>();
                                                let l195 = *ptr0.add(36).cast::<f32>();
                                                let l196 = *ptr0.add(40).cast::<f32>();
                                                let l197 = *ptr0.add(44).cast::<f32>();
                                                let l198 = *ptr0.add(48).cast::<f32>();
                                                super::super::super::wired::input::types::Ray {
                                                    orientation: super::super::super::wired::math::types::Quat {
                                                        x: l192,
                                                        y: l193,
                                                        z: l194,
                                                        w: l195,
                                                    },
                                                    origin: super::super::super::wired::math::types::Vec3 {
                                                        x: l196,
                                                        y: l197,
                                                        z: l198,
                                                    },
                                                }
                                            };
                                            V199::Ray(e199)
                                        }
                                    };
                                    super::super::super::wired::input::types::InputEvent {
                                        id: l2 as u64,
                                        action: v4,
                                        data: v199,
                                    }
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
        }
    }
    #[allow(dead_code)]
    pub mod log {
        #[allow(dead_code, clippy::all)]
        pub mod api {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() =
                super::super::super::__link_custom_section_describing_imports;
            #[repr(u8)]
            #[derive(Clone, Copy, Eq, PartialEq)]
            pub enum LogLevel {
                Debug,
                Info,
                Warn,
                Error,
            }
            impl ::core::fmt::Debug for LogLevel {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        LogLevel::Debug => f.debug_tuple("LogLevel::Debug").finish(),
                        LogLevel::Info => f.debug_tuple("LogLevel::Info").finish(),
                        LogLevel::Warn => f.debug_tuple("LogLevel::Warn").finish(),
                        LogLevel::Error => f.debug_tuple("LogLevel::Error").finish(),
                    }
                }
            }
            impl LogLevel {
                #[doc(hidden)]
                pub unsafe fn _lift(val: u8) -> LogLevel {
                    if !cfg!(debug_assertions) {
                        return ::core::mem::transmute(val);
                    }
                    match val {
                        0 => LogLevel::Debug,
                        1 => LogLevel::Info,
                        2 => LogLevel::Warn,
                        3 => LogLevel::Error,
                        _ => panic!("invalid enum discriminant"),
                    }
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            pub fn log(level: LogLevel, message: &str) {
                unsafe {
                    let vec0 = message;
                    let ptr0 = vec0.as_ptr().cast::<u8>();
                    let len0 = vec0.len();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "wired:log/api")]
                    extern "C" {
                        #[link_name = "log"]
                        fn wit_import(_: i32, _: *mut u8, _: usize);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: i32, _: *mut u8, _: usize) {
                        unreachable!()
                    }
                    wit_import(level.clone() as i32, ptr0.cast_mut(), len0);
                }
            }
        }
    }
    #[allow(dead_code)]
    pub mod math {
        #[allow(dead_code, clippy::all)]
        pub mod types {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() =
                super::super::super::__link_custom_section_describing_imports;
            #[repr(C)]
            #[derive(Clone, Copy)]
            pub struct Vec2 {
                pub x: f32,
                pub y: f32,
            }
            impl ::core::fmt::Debug for Vec2 {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.debug_struct("Vec2")
                        .field("x", &self.x)
                        .field("y", &self.y)
                        .finish()
                }
            }
            #[repr(C)]
            #[derive(Clone, Copy)]
            pub struct Vec3 {
                pub x: f32,
                pub y: f32,
                pub z: f32,
            }
            impl ::core::fmt::Debug for Vec3 {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.debug_struct("Vec3")
                        .field("x", &self.x)
                        .field("y", &self.y)
                        .field("z", &self.z)
                        .finish()
                }
            }
            #[repr(C)]
            #[derive(Clone, Copy)]
            pub struct Quat {
                pub x: f32,
                pub y: f32,
                pub z: f32,
                pub w: f32,
            }
            impl ::core::fmt::Debug for Quat {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.debug_struct("Quat")
                        .field("x", &self.x)
                        .field("y", &self.y)
                        .field("z", &self.z)
                        .field("w", &self.w)
                        .finish()
                }
            }
            #[repr(C)]
            #[derive(Clone, Copy)]
            pub struct Transform {
                pub rotation: Quat,
                pub scale: Vec3,
                pub translation: Vec3,
            }
            impl ::core::fmt::Debug for Transform {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.debug_struct("Transform")
                        .field("rotation", &self.rotation)
                        .field("scale", &self.scale)
                        .field("translation", &self.translation)
                        .finish()
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// Codegen doesn't always include types, these function force its inclusion.
            pub fn fake_fn_a() -> Vec2 {
                unsafe {
                    #[repr(align(4))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                    let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "wired:math/types")]
                    extern "C" {
                        #[link_name = "fake-fn-a"]
                        fn wit_import(_: *mut u8);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: *mut u8) {
                        unreachable!()
                    }
                    wit_import(ptr0);
                    let l1 = *ptr0.add(0).cast::<f32>();
                    let l2 = *ptr0.add(4).cast::<f32>();
                    Vec2 { x: l1, y: l2 }
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            pub fn fake_fn_b() -> Vec3 {
                unsafe {
                    #[repr(align(4))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 12]);
                    let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "wired:math/types")]
                    extern "C" {
                        #[link_name = "fake-fn-b"]
                        fn wit_import(_: *mut u8);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: *mut u8) {
                        unreachable!()
                    }
                    wit_import(ptr0);
                    let l1 = *ptr0.add(0).cast::<f32>();
                    let l2 = *ptr0.add(4).cast::<f32>();
                    let l3 = *ptr0.add(8).cast::<f32>();
                    Vec3 {
                        x: l1,
                        y: l2,
                        z: l3,
                    }
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            pub fn fake_fn_c() -> Quat {
                unsafe {
                    #[repr(align(4))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 16]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 16]);
                    let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "wired:math/types")]
                    extern "C" {
                        #[link_name = "fake-fn-c"]
                        fn wit_import(_: *mut u8);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: *mut u8) {
                        unreachable!()
                    }
                    wit_import(ptr0);
                    let l1 = *ptr0.add(0).cast::<f32>();
                    let l2 = *ptr0.add(4).cast::<f32>();
                    let l3 = *ptr0.add(8).cast::<f32>();
                    let l4 = *ptr0.add(12).cast::<f32>();
                    Quat {
                        x: l1,
                        y: l2,
                        z: l3,
                        w: l4,
                    }
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            pub fn fake_fn_d() -> Transform {
                unsafe {
                    #[repr(align(4))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 40]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 40]);
                    let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "wired:math/types")]
                    extern "C" {
                        #[link_name = "fake-fn-d"]
                        fn wit_import(_: *mut u8);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: *mut u8) {
                        unreachable!()
                    }
                    wit_import(ptr0);
                    let l1 = *ptr0.add(0).cast::<f32>();
                    let l2 = *ptr0.add(4).cast::<f32>();
                    let l3 = *ptr0.add(8).cast::<f32>();
                    let l4 = *ptr0.add(12).cast::<f32>();
                    let l5 = *ptr0.add(16).cast::<f32>();
                    let l6 = *ptr0.add(20).cast::<f32>();
                    let l7 = *ptr0.add(24).cast::<f32>();
                    let l8 = *ptr0.add(28).cast::<f32>();
                    let l9 = *ptr0.add(32).cast::<f32>();
                    let l10 = *ptr0.add(36).cast::<f32>();
                    Transform {
                        rotation: Quat {
                            x: l1,
                            y: l2,
                            z: l3,
                            w: l4,
                        },
                        scale: Vec3 {
                            x: l5,
                            y: l6,
                            z: l7,
                        },
                        translation: Vec3 {
                            x: l8,
                            y: l9,
                            z: l10,
                        },
                    }
                }
            }
        }
    }
    #[allow(dead_code)]
    pub mod physics {
        #[allow(dead_code, clippy::all)]
        pub mod types {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() =
                super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            pub type Vec3 = super::super::super::wired::math::types::Vec3;
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct Collider {
                handle: _rt::Resource<Collider>,
            }
            impl Collider {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for Collider {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "wired:physics/types")]
                        extern "C" {
                            #[link_name = "[resource-drop]collider"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            #[repr(C)]
            #[derive(Clone, Copy)]
            pub struct ShapeCylinder {
                pub height: f32,
                pub radius: f32,
            }
            impl ::core::fmt::Debug for ShapeCylinder {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.debug_struct("ShapeCylinder")
                        .field("height", &self.height)
                        .field("radius", &self.radius)
                        .finish()
                }
            }
            #[derive(Clone, Copy)]
            pub enum Shape {
                Cuboid(Vec3),
                Cylinder(ShapeCylinder),
                Sphere(f32),
            }
            impl ::core::fmt::Debug for Shape {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        Shape::Cuboid(e) => f.debug_tuple("Shape::Cuboid").field(e).finish(),
                        Shape::Cylinder(e) => f.debug_tuple("Shape::Cylinder").field(e).finish(),
                        Shape::Sphere(e) => f.debug_tuple("Shape::Sphere").field(e).finish(),
                    }
                }
            }
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct RigidBody {
                handle: _rt::Resource<RigidBody>,
            }
            impl RigidBody {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for RigidBody {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "wired:physics/types")]
                        extern "C" {
                            #[link_name = "[resource-drop]rigid-body"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            #[repr(u8)]
            #[derive(Clone, Copy, Eq, PartialEq)]
            pub enum RigidBodyType {
                Dynamic,
                Fixed,
                Kinematic,
            }
            impl ::core::fmt::Debug for RigidBodyType {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        RigidBodyType::Dynamic => f.debug_tuple("RigidBodyType::Dynamic").finish(),
                        RigidBodyType::Fixed => f.debug_tuple("RigidBodyType::Fixed").finish(),
                        RigidBodyType::Kinematic => {
                            f.debug_tuple("RigidBodyType::Kinematic").finish()
                        }
                    }
                }
            }
            impl RigidBodyType {
                #[doc(hidden)]
                pub unsafe fn _lift(val: u8) -> RigidBodyType {
                    if !cfg!(debug_assertions) {
                        return ::core::mem::transmute(val);
                    }
                    match val {
                        0 => RigidBodyType::Dynamic,
                        1 => RigidBodyType::Fixed,
                        2 => RigidBodyType::Kinematic,
                        _ => panic!("invalid enum discriminant"),
                    }
                }
            }
            impl Collider {
                #[allow(unused_unsafe, clippy::all)]
                pub fn new(shape: Shape) -> Self {
                    unsafe {
                        let (result2_0, result2_1, result2_2, result2_3) = match shape {
                            Shape::Cuboid(e) => {
                                let super::super::super::wired::math::types::Vec3 {
                                    x: x0,
                                    y: y0,
                                    z: z0,
                                } = e;
                                (0i32, _rt::as_f32(x0), _rt::as_f32(y0), _rt::as_f32(z0))
                            }
                            Shape::Cylinder(e) => {
                                let ShapeCylinder {
                                    height: height1,
                                    radius: radius1,
                                } = e;
                                (1i32, _rt::as_f32(height1), _rt::as_f32(radius1), 0.0f32)
                            }
                            Shape::Sphere(e) => (2i32, _rt::as_f32(e), 0.0f32, 0.0f32),
                        };
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:physics/types")]
                        extern "C" {
                            #[link_name = "[constructor]collider"]
                            fn wit_import(_: i32, _: f32, _: f32, _: f32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: f32, _: f32, _: f32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(result2_0, result2_1, result2_2, result2_3);
                        Collider::from_handle(ret as u32)
                    }
                }
            }
            impl Collider {
                #[allow(unused_unsafe, clippy::all)]
                pub fn density(&self) -> f32 {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:physics/types")]
                        extern "C" {
                            #[link_name = "[method]collider.density"]
                            fn wit_import(_: i32) -> f32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> f32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        ret
                    }
                }
            }
            impl Collider {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_density(&self, value: f32) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:physics/types")]
                        extern "C" {
                            #[link_name = "[method]collider.set-density"]
                            fn wit_import(_: i32, _: f32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: f32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, _rt::as_f32(&value));
                    }
                }
            }
            impl RigidBody {
                #[allow(unused_unsafe, clippy::all)]
                pub fn new(rigid_body_type: RigidBodyType) -> Self {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:physics/types")]
                        extern "C" {
                            #[link_name = "[constructor]rigid-body"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(rigid_body_type.clone() as i32);
                        RigidBody::from_handle(ret as u32)
                    }
                }
            }
            impl RigidBody {
                #[allow(unused_unsafe, clippy::all)]
                pub fn angvel(&self) -> Vec3 {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 12]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:physics/types")]
                        extern "C" {
                            #[link_name = "[method]rigid-body.angvel"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<f32>();
                        let l2 = *ptr0.add(4).cast::<f32>();
                        let l3 = *ptr0.add(8).cast::<f32>();
                        super::super::super::wired::math::types::Vec3 {
                            x: l1,
                            y: l2,
                            z: l3,
                        }
                    }
                }
            }
            impl RigidBody {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_angvel(&self, value: Vec3) {
                    unsafe {
                        let super::super::super::wired::math::types::Vec3 {
                            x: x0,
                            y: y0,
                            z: z0,
                        } = value;
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:physics/types")]
                        extern "C" {
                            #[link_name = "[method]rigid-body.set-angvel"]
                            fn wit_import(_: i32, _: f32, _: f32, _: f32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: f32, _: f32, _: f32) {
                            unreachable!()
                        }
                        wit_import(
                            (self).handle() as i32,
                            _rt::as_f32(x0),
                            _rt::as_f32(y0),
                            _rt::as_f32(z0),
                        );
                    }
                }
            }
            impl RigidBody {
                #[allow(unused_unsafe, clippy::all)]
                pub fn linvel(&self) -> Vec3 {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 12]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:physics/types")]
                        extern "C" {
                            #[link_name = "[method]rigid-body.linvel"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<f32>();
                        let l2 = *ptr0.add(4).cast::<f32>();
                        let l3 = *ptr0.add(8).cast::<f32>();
                        super::super::super::wired::math::types::Vec3 {
                            x: l1,
                            y: l2,
                            z: l3,
                        }
                    }
                }
            }
            impl RigidBody {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_linvel(&self, value: Vec3) {
                    unsafe {
                        let super::super::super::wired::math::types::Vec3 {
                            x: x0,
                            y: y0,
                            z: z0,
                        } = value;
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:physics/types")]
                        extern "C" {
                            #[link_name = "[method]rigid-body.set-linvel"]
                            fn wit_import(_: i32, _: f32, _: f32, _: f32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: f32, _: f32, _: f32) {
                            unreachable!()
                        }
                        wit_import(
                            (self).handle() as i32,
                            _rt::as_f32(x0),
                            _rt::as_f32(y0),
                            _rt::as_f32(z0),
                        );
                    }
                }
            }
        }
    }
    #[allow(dead_code)]
    pub mod player {
        #[allow(dead_code, clippy::all)]
        pub mod api {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() =
                super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            pub type Node = super::super::super::wired::scene::node::Node;
            pub struct Skeleton {
                pub hips: Node,
                pub spine: Node,
                pub chest: Node,
                pub upper_chest: Node,
                pub neck: Node,
                pub head: Node,
                pub left_shoulder: Node,
                pub left_upper_arm: Node,
                pub left_lower_arm: Node,
                pub left_hand: Node,
                pub right_shoulder: Node,
                pub right_upper_arm: Node,
                pub right_lower_arm: Node,
                pub right_hand: Node,
                pub left_upper_leg: Node,
                pub left_lower_leg: Node,
                pub left_foot: Node,
                pub right_upper_leg: Node,
                pub right_lower_leg: Node,
                pub right_foot: Node,
            }
            impl ::core::fmt::Debug for Skeleton {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.debug_struct("Skeleton")
                        .field("hips", &self.hips)
                        .field("spine", &self.spine)
                        .field("chest", &self.chest)
                        .field("upper-chest", &self.upper_chest)
                        .field("neck", &self.neck)
                        .field("head", &self.head)
                        .field("left-shoulder", &self.left_shoulder)
                        .field("left-upper-arm", &self.left_upper_arm)
                        .field("left-lower-arm", &self.left_lower_arm)
                        .field("left-hand", &self.left_hand)
                        .field("right-shoulder", &self.right_shoulder)
                        .field("right-upper-arm", &self.right_upper_arm)
                        .field("right-lower-arm", &self.right_lower_arm)
                        .field("right-hand", &self.right_hand)
                        .field("left-upper-leg", &self.left_upper_leg)
                        .field("left-lower-leg", &self.left_lower_leg)
                        .field("left-foot", &self.left_foot)
                        .field("right-upper-leg", &self.right_upper_leg)
                        .field("right-lower-leg", &self.right_lower_leg)
                        .field("right-foot", &self.right_foot)
                        .finish()
                }
            }
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct Player {
                handle: _rt::Resource<Player>,
            }
            impl Player {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for Player {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "wired:player/api")]
                        extern "C" {
                            #[link_name = "[resource-drop]player"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            impl Player {
                #[allow(unused_unsafe, clippy::all)]
                pub fn root(&self) -> Node {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:player/api")]
                        extern "C" {
                            #[link_name = "[method]player.root"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::scene::node::Node::from_handle(ret as u32)
                    }
                }
            }
            impl Player {
                #[allow(unused_unsafe, clippy::all)]
                pub fn skeleton(&self) -> Skeleton {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 80]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 80]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:player/api")]
                        extern "C" {
                            #[link_name = "[method]player.skeleton"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<i32>();
                        let l2 = *ptr0.add(4).cast::<i32>();
                        let l3 = *ptr0.add(8).cast::<i32>();
                        let l4 = *ptr0.add(12).cast::<i32>();
                        let l5 = *ptr0.add(16).cast::<i32>();
                        let l6 = *ptr0.add(20).cast::<i32>();
                        let l7 = *ptr0.add(24).cast::<i32>();
                        let l8 = *ptr0.add(28).cast::<i32>();
                        let l9 = *ptr0.add(32).cast::<i32>();
                        let l10 = *ptr0.add(36).cast::<i32>();
                        let l11 = *ptr0.add(40).cast::<i32>();
                        let l12 = *ptr0.add(44).cast::<i32>();
                        let l13 = *ptr0.add(48).cast::<i32>();
                        let l14 = *ptr0.add(52).cast::<i32>();
                        let l15 = *ptr0.add(56).cast::<i32>();
                        let l16 = *ptr0.add(60).cast::<i32>();
                        let l17 = *ptr0.add(64).cast::<i32>();
                        let l18 = *ptr0.add(68).cast::<i32>();
                        let l19 = *ptr0.add(72).cast::<i32>();
                        let l20 = *ptr0.add(76).cast::<i32>();
                        Skeleton {
                            hips: super::super::super::wired::scene::node::Node::from_handle(
                                l1 as u32,
                            ),
                            spine: super::super::super::wired::scene::node::Node::from_handle(
                                l2 as u32,
                            ),
                            chest: super::super::super::wired::scene::node::Node::from_handle(
                                l3 as u32,
                            ),
                            upper_chest: super::super::super::wired::scene::node::Node::from_handle(
                                l4 as u32,
                            ),
                            neck: super::super::super::wired::scene::node::Node::from_handle(
                                l5 as u32,
                            ),
                            head: super::super::super::wired::scene::node::Node::from_handle(
                                l6 as u32,
                            ),
                            left_shoulder:
                                super::super::super::wired::scene::node::Node::from_handle(
                                    l7 as u32,
                                ),
                            left_upper_arm:
                                super::super::super::wired::scene::node::Node::from_handle(
                                    l8 as u32,
                                ),
                            left_lower_arm:
                                super::super::super::wired::scene::node::Node::from_handle(
                                    l9 as u32,
                                ),
                            left_hand: super::super::super::wired::scene::node::Node::from_handle(
                                l10 as u32,
                            ),
                            right_shoulder:
                                super::super::super::wired::scene::node::Node::from_handle(
                                    l11 as u32,
                                ),
                            right_upper_arm:
                                super::super::super::wired::scene::node::Node::from_handle(
                                    l12 as u32,
                                ),
                            right_lower_arm:
                                super::super::super::wired::scene::node::Node::from_handle(
                                    l13 as u32,
                                ),
                            right_hand: super::super::super::wired::scene::node::Node::from_handle(
                                l14 as u32,
                            ),
                            left_upper_leg:
                                super::super::super::wired::scene::node::Node::from_handle(
                                    l15 as u32,
                                ),
                            left_lower_leg:
                                super::super::super::wired::scene::node::Node::from_handle(
                                    l16 as u32,
                                ),
                            left_foot: super::super::super::wired::scene::node::Node::from_handle(
                                l17 as u32,
                            ),
                            right_upper_leg:
                                super::super::super::wired::scene::node::Node::from_handle(
                                    l18 as u32,
                                ),
                            right_lower_leg:
                                super::super::super::wired::scene::node::Node::from_handle(
                                    l19 as u32,
                                ),
                            right_foot: super::super::super::wired::scene::node::Node::from_handle(
                                l20 as u32,
                            ),
                        }
                    }
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            pub fn list_players() -> _rt::Vec<Player> {
                unsafe {
                    #[repr(align(4))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                    let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "wired:player/api")]
                    extern "C" {
                        #[link_name = "list-players"]
                        fn wit_import(_: *mut u8);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: *mut u8) {
                        unreachable!()
                    }
                    wit_import(ptr0);
                    let l1 = *ptr0.add(0).cast::<*mut u8>();
                    let l2 = *ptr0.add(4).cast::<usize>();
                    let base4 = l1;
                    let len4 = l2;
                    let mut result4 = _rt::Vec::with_capacity(len4);
                    for i in 0..len4 {
                        let base = base4.add(i * 4);
                        let e4 = {
                            let l3 = *base.add(0).cast::<i32>();
                            Player::from_handle(l3 as u32)
                        };
                        result4.push(e4);
                    }
                    _rt::cabi_dealloc(base4, len4 * 4, 4);
                    result4
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            pub fn local_player() -> Player {
                unsafe {
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "wired:player/api")]
                    extern "C" {
                        #[link_name = "local-player"]
                        fn wit_import() -> i32;
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import() -> i32 {
                        unreachable!()
                    }
                    let ret = wit_import();
                    Player::from_handle(ret as u32)
                }
            }
        }
    }
    #[allow(dead_code)]
    pub mod scene {
        #[allow(dead_code, clippy::all)]
        pub mod material {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() =
                super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            #[repr(C)]
            #[derive(Clone, Copy)]
            pub struct Color {
                pub r: f32,
                pub g: f32,
                pub b: f32,
                pub a: f32,
            }
            impl ::core::fmt::Debug for Color {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.debug_struct("Color")
                        .field("r", &self.r)
                        .field("g", &self.g)
                        .field("b", &self.b)
                        .field("a", &self.a)
                        .finish()
                }
            }
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct Material {
                handle: _rt::Resource<Material>,
            }
            impl Material {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for Material {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "wired:scene/material")]
                        extern "C" {
                            #[link_name = "[resource-drop]material"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            impl Material {
                #[allow(unused_unsafe, clippy::all)]
                pub fn new() -> Self {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/material")]
                        extern "C" {
                            #[link_name = "[constructor]material"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        Material::from_handle(ret as u32)
                    }
                }
            }
            impl Material {
                #[allow(unused_unsafe, clippy::all)]
                pub fn id(&self) -> u32 {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/material")]
                        extern "C" {
                            #[link_name = "[method]material.id"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        ret as u32
                    }
                }
            }
            impl Material {
                #[allow(unused_unsafe, clippy::all)]
                /// Returns another reference to the same resource.
                pub fn ref_(&self) -> Material {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/material")]
                        extern "C" {
                            #[link_name = "[method]material.ref"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        Material::from_handle(ret as u32)
                    }
                }
            }
            impl Material {
                #[allow(unused_unsafe, clippy::all)]
                pub fn name(&self) -> _rt::String {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/material")]
                        extern "C" {
                            #[link_name = "[method]material.name"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<*mut u8>();
                        let l2 = *ptr0.add(4).cast::<usize>();
                        let len3 = l2;
                        let bytes3 = _rt::Vec::from_raw_parts(l1.cast(), len3, len3);
                        _rt::string_lift(bytes3)
                    }
                }
            }
            impl Material {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_name(&self, value: &str) {
                    unsafe {
                        let vec0 = value;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/material")]
                        extern "C" {
                            #[link_name = "[method]material.set-name"]
                            fn wit_import(_: i32, _: *mut u8, _: usize);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8, _: usize) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0.cast_mut(), len0);
                    }
                }
            }
            impl Material {
                #[allow(unused_unsafe, clippy::all)]
                pub fn color(&self) -> Color {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 16]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 16]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/material")]
                        extern "C" {
                            #[link_name = "[method]material.color"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<f32>();
                        let l2 = *ptr0.add(4).cast::<f32>();
                        let l3 = *ptr0.add(8).cast::<f32>();
                        let l4 = *ptr0.add(12).cast::<f32>();
                        Color {
                            r: l1,
                            g: l2,
                            b: l3,
                            a: l4,
                        }
                    }
                }
            }
            impl Material {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_color(&self, value: Color) {
                    unsafe {
                        let Color {
                            r: r0,
                            g: g0,
                            b: b0,
                            a: a0,
                        } = value;
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/material")]
                        extern "C" {
                            #[link_name = "[method]material.set-color"]
                            fn wit_import(_: i32, _: f32, _: f32, _: f32, _: f32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: f32, _: f32, _: f32, _: f32) {
                            unreachable!()
                        }
                        wit_import(
                            (self).handle() as i32,
                            _rt::as_f32(r0),
                            _rt::as_f32(g0),
                            _rt::as_f32(b0),
                            _rt::as_f32(a0),
                        );
                    }
                }
            }
        }
        #[allow(dead_code, clippy::all)]
        pub mod mesh {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() =
                super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            pub type Material = super::super::super::wired::scene::material::Material;
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct Primitive {
                handle: _rt::Resource<Primitive>,
            }
            impl Primitive {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for Primitive {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "wired:scene/mesh")]
                        extern "C" {
                            #[link_name = "[resource-drop]primitive"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct Mesh {
                handle: _rt::Resource<Mesh>,
            }
            impl Mesh {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for Mesh {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "wired:scene/mesh")]
                        extern "C" {
                            #[link_name = "[resource-drop]mesh"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            impl Primitive {
                #[allow(unused_unsafe, clippy::all)]
                pub fn id(&self) -> u32 {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/mesh")]
                        extern "C" {
                            #[link_name = "[method]primitive.id"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        ret as u32
                    }
                }
            }
            impl Primitive {
                #[allow(unused_unsafe, clippy::all)]
                pub fn material(&self) -> Option<Material> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/mesh")]
                        extern "C" {
                            #[link_name = "[method]primitive.material"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<i32>();
                                    super::super::super::wired::scene::material::Material::from_handle(
                                        l2 as u32,
                                    )
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Primitive {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_material(&self, value: Option<&Material>) {
                    unsafe {
                        let (result0_0, result0_1) = match value {
                            Some(e) => (1i32, (e).handle() as i32),
                            None => (0i32, 0i32),
                        };
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/mesh")]
                        extern "C" {
                            #[link_name = "[method]primitive.set-material"]
                            fn wit_import(_: i32, _: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, result0_0, result0_1);
                    }
                }
            }
            impl Primitive {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_indices(&self, value: &[u32]) {
                    unsafe {
                        let vec0 = value;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/mesh")]
                        extern "C" {
                            #[link_name = "[method]primitive.set-indices"]
                            fn wit_import(_: i32, _: *mut u8, _: usize);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8, _: usize) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0.cast_mut(), len0);
                    }
                }
            }
            impl Primitive {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_normals(&self, value: &[f32]) {
                    unsafe {
                        let vec0 = value;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/mesh")]
                        extern "C" {
                            #[link_name = "[method]primitive.set-normals"]
                            fn wit_import(_: i32, _: *mut u8, _: usize);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8, _: usize) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0.cast_mut(), len0);
                    }
                }
            }
            impl Primitive {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_positions(&self, value: &[f32]) {
                    unsafe {
                        let vec0 = value;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/mesh")]
                        extern "C" {
                            #[link_name = "[method]primitive.set-positions"]
                            fn wit_import(_: i32, _: *mut u8, _: usize);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8, _: usize) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0.cast_mut(), len0);
                    }
                }
            }
            impl Primitive {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_uvs(&self, value: &[f32]) {
                    unsafe {
                        let vec0 = value;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/mesh")]
                        extern "C" {
                            #[link_name = "[method]primitive.set-uvs"]
                            fn wit_import(_: i32, _: *mut u8, _: usize);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8, _: usize) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0.cast_mut(), len0);
                    }
                }
            }
            impl Mesh {
                #[allow(unused_unsafe, clippy::all)]
                pub fn new() -> Self {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/mesh")]
                        extern "C" {
                            #[link_name = "[constructor]mesh"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        Mesh::from_handle(ret as u32)
                    }
                }
            }
            impl Mesh {
                #[allow(unused_unsafe, clippy::all)]
                pub fn id(&self) -> u32 {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/mesh")]
                        extern "C" {
                            #[link_name = "[method]mesh.id"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        ret as u32
                    }
                }
            }
            impl Mesh {
                #[allow(unused_unsafe, clippy::all)]
                /// Returns another reference to the same resource.
                pub fn ref_(&self) -> Mesh {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/mesh")]
                        extern "C" {
                            #[link_name = "[method]mesh.ref"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        Mesh::from_handle(ret as u32)
                    }
                }
            }
            impl Mesh {
                #[allow(unused_unsafe, clippy::all)]
                pub fn name(&self) -> _rt::String {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/mesh")]
                        extern "C" {
                            #[link_name = "[method]mesh.name"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<*mut u8>();
                        let l2 = *ptr0.add(4).cast::<usize>();
                        let len3 = l2;
                        let bytes3 = _rt::Vec::from_raw_parts(l1.cast(), len3, len3);
                        _rt::string_lift(bytes3)
                    }
                }
            }
            impl Mesh {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_name(&self, value: &str) {
                    unsafe {
                        let vec0 = value;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/mesh")]
                        extern "C" {
                            #[link_name = "[method]mesh.set-name"]
                            fn wit_import(_: i32, _: *mut u8, _: usize);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8, _: usize) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0.cast_mut(), len0);
                    }
                }
            }
            impl Mesh {
                #[allow(unused_unsafe, clippy::all)]
                pub fn list_primitives(&self) -> _rt::Vec<Primitive> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/mesh")]
                        extern "C" {
                            #[link_name = "[method]mesh.list-primitives"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<*mut u8>();
                        let l2 = *ptr0.add(4).cast::<usize>();
                        let base4 = l1;
                        let len4 = l2;
                        let mut result4 = _rt::Vec::with_capacity(len4);
                        for i in 0..len4 {
                            let base = base4.add(i * 4);
                            let e4 = {
                                let l3 = *base.add(0).cast::<i32>();
                                Primitive::from_handle(l3 as u32)
                            };
                            result4.push(e4);
                        }
                        _rt::cabi_dealloc(base4, len4 * 4, 4);
                        result4
                    }
                }
            }
            impl Mesh {
                #[allow(unused_unsafe, clippy::all)]
                pub fn create_primitive(&self) -> Primitive {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/mesh")]
                        extern "C" {
                            #[link_name = "[method]mesh.create-primitive"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        Primitive::from_handle(ret as u32)
                    }
                }
            }
            impl Mesh {
                #[allow(unused_unsafe, clippy::all)]
                pub fn remove_primitive(&self, value: Primitive) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/mesh")]
                        extern "C" {
                            #[link_name = "[method]mesh.remove-primitive"]
                            fn wit_import(_: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, (&value).take_handle() as i32);
                    }
                }
            }
        }
        #[allow(dead_code, clippy::all)]
        pub mod node {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() =
                super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            pub type Mesh = super::super::super::wired::scene::mesh::Mesh;
            pub type InputHandler = super::super::super::wired::input::handler::InputHandler;
            pub type Transform = super::super::super::wired::math::types::Transform;
            pub type Collider = super::super::super::wired::physics::types::Collider;
            pub type RigidBody = super::super::super::wired::physics::types::RigidBody;
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct Node {
                handle: _rt::Resource<Node>,
            }
            impl Node {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for Node {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "wired:scene/node")]
                        extern "C" {
                            #[link_name = "[resource-drop]node"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            impl Node {
                #[allow(unused_unsafe, clippy::all)]
                pub fn new() -> Self {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/node")]
                        extern "C" {
                            #[link_name = "[constructor]node"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        Node::from_handle(ret as u32)
                    }
                }
            }
            impl Node {
                #[allow(unused_unsafe, clippy::all)]
                pub fn id(&self) -> u32 {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/node")]
                        extern "C" {
                            #[link_name = "[method]node.id"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        ret as u32
                    }
                }
            }
            impl Node {
                #[allow(unused_unsafe, clippy::all)]
                /// Returns another reference to the same resource.
                pub fn ref_(&self) -> Node {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/node")]
                        extern "C" {
                            #[link_name = "[method]node.ref"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        Node::from_handle(ret as u32)
                    }
                }
            }
            impl Node {
                #[allow(unused_unsafe, clippy::all)]
                pub fn name(&self) -> _rt::String {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/node")]
                        extern "C" {
                            #[link_name = "[method]node.name"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<*mut u8>();
                        let l2 = *ptr0.add(4).cast::<usize>();
                        let len3 = l2;
                        let bytes3 = _rt::Vec::from_raw_parts(l1.cast(), len3, len3);
                        _rt::string_lift(bytes3)
                    }
                }
            }
            impl Node {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_name(&self, value: &str) {
                    unsafe {
                        let vec0 = value;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/node")]
                        extern "C" {
                            #[link_name = "[method]node.set-name"]
                            fn wit_import(_: i32, _: *mut u8, _: usize);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8, _: usize) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0.cast_mut(), len0);
                    }
                }
            }
            impl Node {
                #[allow(unused_unsafe, clippy::all)]
                pub fn parent(&self) -> Option<Node> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/node")]
                        extern "C" {
                            #[link_name = "[method]node.parent"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<i32>();
                                    Node::from_handle(l2 as u32)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Node {
                #[allow(unused_unsafe, clippy::all)]
                pub fn children(&self) -> _rt::Vec<Node> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/node")]
                        extern "C" {
                            #[link_name = "[method]node.children"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<*mut u8>();
                        let l2 = *ptr0.add(4).cast::<usize>();
                        let base4 = l1;
                        let len4 = l2;
                        let mut result4 = _rt::Vec::with_capacity(len4);
                        for i in 0..len4 {
                            let base = base4.add(i * 4);
                            let e4 = {
                                let l3 = *base.add(0).cast::<i32>();
                                Node::from_handle(l3 as u32)
                            };
                            result4.push(e4);
                        }
                        _rt::cabi_dealloc(base4, len4 * 4, 4);
                        result4
                    }
                }
            }
            impl Node {
                #[allow(unused_unsafe, clippy::all)]
                pub fn add_child(&self, value: &Node) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/node")]
                        extern "C" {
                            #[link_name = "[method]node.add-child"]
                            fn wit_import(_: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, (value).handle() as i32);
                    }
                }
            }
            impl Node {
                #[allow(unused_unsafe, clippy::all)]
                pub fn remove_child(&self, value: &Node) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/node")]
                        extern "C" {
                            #[link_name = "[method]node.remove-child"]
                            fn wit_import(_: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, (value).handle() as i32);
                    }
                }
            }
            impl Node {
                #[allow(unused_unsafe, clippy::all)]
                pub fn global_transform(&self) -> Transform {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 40]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 40]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/node")]
                        extern "C" {
                            #[link_name = "[method]node.global-transform"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<f32>();
                        let l2 = *ptr0.add(4).cast::<f32>();
                        let l3 = *ptr0.add(8).cast::<f32>();
                        let l4 = *ptr0.add(12).cast::<f32>();
                        let l5 = *ptr0.add(16).cast::<f32>();
                        let l6 = *ptr0.add(20).cast::<f32>();
                        let l7 = *ptr0.add(24).cast::<f32>();
                        let l8 = *ptr0.add(28).cast::<f32>();
                        let l9 = *ptr0.add(32).cast::<f32>();
                        let l10 = *ptr0.add(36).cast::<f32>();
                        super::super::super::wired::math::types::Transform {
                            rotation: super::super::super::wired::math::types::Quat {
                                x: l1,
                                y: l2,
                                z: l3,
                                w: l4,
                            },
                            scale: super::super::super::wired::math::types::Vec3 {
                                x: l5,
                                y: l6,
                                z: l7,
                            },
                            translation: super::super::super::wired::math::types::Vec3 {
                                x: l8,
                                y: l9,
                                z: l10,
                            },
                        }
                    }
                }
            }
            impl Node {
                #[allow(unused_unsafe, clippy::all)]
                pub fn transform(&self) -> Transform {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 40]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 40]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/node")]
                        extern "C" {
                            #[link_name = "[method]node.transform"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<f32>();
                        let l2 = *ptr0.add(4).cast::<f32>();
                        let l3 = *ptr0.add(8).cast::<f32>();
                        let l4 = *ptr0.add(12).cast::<f32>();
                        let l5 = *ptr0.add(16).cast::<f32>();
                        let l6 = *ptr0.add(20).cast::<f32>();
                        let l7 = *ptr0.add(24).cast::<f32>();
                        let l8 = *ptr0.add(28).cast::<f32>();
                        let l9 = *ptr0.add(32).cast::<f32>();
                        let l10 = *ptr0.add(36).cast::<f32>();
                        super::super::super::wired::math::types::Transform {
                            rotation: super::super::super::wired::math::types::Quat {
                                x: l1,
                                y: l2,
                                z: l3,
                                w: l4,
                            },
                            scale: super::super::super::wired::math::types::Vec3 {
                                x: l5,
                                y: l6,
                                z: l7,
                            },
                            translation: super::super::super::wired::math::types::Vec3 {
                                x: l8,
                                y: l9,
                                z: l10,
                            },
                        }
                    }
                }
            }
            impl Node {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_transform(&self, value: Transform) {
                    unsafe {
                        let super::super::super::wired::math::types::Transform {
                            rotation: rotation0,
                            scale: scale0,
                            translation: translation0,
                        } = value;
                        let super::super::super::wired::math::types::Quat {
                            x: x1,
                            y: y1,
                            z: z1,
                            w: w1,
                        } = rotation0;
                        let super::super::super::wired::math::types::Vec3 {
                            x: x2,
                            y: y2,
                            z: z2,
                        } = scale0;
                        let super::super::super::wired::math::types::Vec3 {
                            x: x3,
                            y: y3,
                            z: z3,
                        } = translation0;
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/node")]
                        extern "C" {
                            #[link_name = "[method]node.set-transform"]
                            fn wit_import(
                                _: i32,
                                _: f32,
                                _: f32,
                                _: f32,
                                _: f32,
                                _: f32,
                                _: f32,
                                _: f32,
                                _: f32,
                                _: f32,
                                _: f32,
                            );
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(
                            _: i32,
                            _: f32,
                            _: f32,
                            _: f32,
                            _: f32,
                            _: f32,
                            _: f32,
                            _: f32,
                            _: f32,
                            _: f32,
                            _: f32,
                        ) {
                            unreachable!()
                        }
                        wit_import(
                            (self).handle() as i32,
                            _rt::as_f32(x1),
                            _rt::as_f32(y1),
                            _rt::as_f32(z1),
                            _rt::as_f32(w1),
                            _rt::as_f32(x2),
                            _rt::as_f32(y2),
                            _rt::as_f32(z2),
                            _rt::as_f32(x3),
                            _rt::as_f32(y3),
                            _rt::as_f32(z3),
                        );
                    }
                }
            }
            impl Node {
                #[allow(unused_unsafe, clippy::all)]
                pub fn mesh(&self) -> Option<Mesh> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/node")]
                        extern "C" {
                            #[link_name = "[method]node.mesh"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<i32>();
                                    super::super::super::wired::scene::mesh::Mesh::from_handle(
                                        l2 as u32,
                                    )
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Node {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_mesh(&self, value: Option<&Mesh>) {
                    unsafe {
                        let (result0_0, result0_1) = match value {
                            Some(e) => (1i32, (e).handle() as i32),
                            None => (0i32, 0i32),
                        };
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/node")]
                        extern "C" {
                            #[link_name = "[method]node.set-mesh"]
                            fn wit_import(_: i32, _: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, result0_0, result0_1);
                    }
                }
            }
            impl Node {
                #[allow(unused_unsafe, clippy::all)]
                pub fn collider(&self) -> Option<Collider> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/node")]
                        extern "C" {
                            #[link_name = "[method]node.collider"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<i32>();
                                    super::super::super::wired::physics::types::Collider::from_handle(
                                        l2 as u32,
                                    )
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Node {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_collider(&self, value: Option<&Collider>) {
                    unsafe {
                        let (result0_0, result0_1) = match value {
                            Some(e) => (1i32, (e).handle() as i32),
                            None => (0i32, 0i32),
                        };
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/node")]
                        extern "C" {
                            #[link_name = "[method]node.set-collider"]
                            fn wit_import(_: i32, _: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, result0_0, result0_1);
                    }
                }
            }
            impl Node {
                #[allow(unused_unsafe, clippy::all)]
                pub fn rigid_body(&self) -> Option<RigidBody> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/node")]
                        extern "C" {
                            #[link_name = "[method]node.rigid-body"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<i32>();
                                    super::super::super::wired::physics::types::RigidBody::from_handle(
                                        l2 as u32,
                                    )
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Node {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_rigid_body(&self, value: Option<&RigidBody>) {
                    unsafe {
                        let (result0_0, result0_1) = match value {
                            Some(e) => (1i32, (e).handle() as i32),
                            None => (0i32, 0i32),
                        };
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/node")]
                        extern "C" {
                            #[link_name = "[method]node.set-rigid-body"]
                            fn wit_import(_: i32, _: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, result0_0, result0_1);
                    }
                }
            }
            impl Node {
                #[allow(unused_unsafe, clippy::all)]
                pub fn input_handler(&self) -> Option<InputHandler> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/node")]
                        extern "C" {
                            #[link_name = "[method]node.input-handler"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<i32>();
                                    super::super::super::wired::input::handler::InputHandler::from_handle(
                                        l2 as u32,
                                    )
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Node {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_input_handler(&self, value: Option<&InputHandler>) {
                    unsafe {
                        let (result0_0, result0_1) = match value {
                            Some(e) => (1i32, (e).handle() as i32),
                            None => (0i32, 0i32),
                        };
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/node")]
                        extern "C" {
                            #[link_name = "[method]node.set-input-handler"]
                            fn wit_import(_: i32, _: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, result0_0, result0_1);
                    }
                }
            }
        }
        #[allow(dead_code, clippy::all)]
        pub mod scene {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() =
                super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            pub type Node = super::super::super::wired::scene::node::Node;
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct Scene {
                handle: _rt::Resource<Scene>,
            }
            impl Scene {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for Scene {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "wired:scene/scene")]
                        extern "C" {
                            #[link_name = "[resource-drop]scene"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            impl Scene {
                #[allow(unused_unsafe, clippy::all)]
                pub fn new() -> Self {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/scene")]
                        extern "C" {
                            #[link_name = "[constructor]scene"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        Scene::from_handle(ret as u32)
                    }
                }
            }
            impl Scene {
                #[allow(unused_unsafe, clippy::all)]
                pub fn id(&self) -> u32 {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/scene")]
                        extern "C" {
                            #[link_name = "[method]scene.id"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        ret as u32
                    }
                }
            }
            impl Scene {
                #[allow(unused_unsafe, clippy::all)]
                pub fn name(&self) -> _rt::String {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/scene")]
                        extern "C" {
                            #[link_name = "[method]scene.name"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<*mut u8>();
                        let l2 = *ptr0.add(4).cast::<usize>();
                        let len3 = l2;
                        let bytes3 = _rt::Vec::from_raw_parts(l1.cast(), len3, len3);
                        _rt::string_lift(bytes3)
                    }
                }
            }
            impl Scene {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_name(&self, value: &str) {
                    unsafe {
                        let vec0 = value;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/scene")]
                        extern "C" {
                            #[link_name = "[method]scene.set-name"]
                            fn wit_import(_: i32, _: *mut u8, _: usize);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8, _: usize) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0.cast_mut(), len0);
                    }
                }
            }
            impl Scene {
                #[allow(unused_unsafe, clippy::all)]
                pub fn nodes(&self) -> _rt::Vec<Node> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/scene")]
                        extern "C" {
                            #[link_name = "[method]scene.nodes"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<*mut u8>();
                        let l2 = *ptr0.add(4).cast::<usize>();
                        let base4 = l1;
                        let len4 = l2;
                        let mut result4 = _rt::Vec::with_capacity(len4);
                        for i in 0..len4 {
                            let base = base4.add(i * 4);
                            let e4 = {
                                let l3 = *base.add(0).cast::<i32>();
                                super::super::super::wired::scene::node::Node::from_handle(
                                    l3 as u32,
                                )
                            };
                            result4.push(e4);
                        }
                        _rt::cabi_dealloc(base4, len4 * 4, 4);
                        result4
                    }
                }
            }
            impl Scene {
                #[allow(unused_unsafe, clippy::all)]
                pub fn add_node(&self, value: &Node) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/scene")]
                        extern "C" {
                            #[link_name = "[method]scene.add-node"]
                            fn wit_import(_: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, (value).handle() as i32);
                    }
                }
            }
            impl Scene {
                #[allow(unused_unsafe, clippy::all)]
                pub fn remove_node(&self, value: &Node) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/scene")]
                        extern "C" {
                            #[link_name = "[method]scene.remove-node"]
                            fn wit_import(_: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, (value).handle() as i32);
                    }
                }
            }
        }
        #[allow(dead_code, clippy::all)]
        pub mod document {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() =
                super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            pub type Scene = super::super::super::wired::scene::scene::Scene;
            /// A save-able collection of resources with an active scene.
            /// Roughly equivalent to a glTF file.
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct Document {
                handle: _rt::Resource<Document>,
            }
            impl Document {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for Document {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "wired:scene/document")]
                        extern "C" {
                            #[link_name = "[resource-drop]document"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            impl Document {
                #[allow(unused_unsafe, clippy::all)]
                pub fn new() -> Self {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/document")]
                        extern "C" {
                            #[link_name = "[constructor]document"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        Document::from_handle(ret as u32)
                    }
                }
            }
            impl Document {
                #[allow(unused_unsafe, clippy::all)]
                pub fn active_scene(&self) -> Option<Scene> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/document")]
                        extern "C" {
                            #[link_name = "[method]document.active-scene"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<i32>();
                                    super::super::super::wired::scene::scene::Scene::from_handle(
                                        l2 as u32,
                                    )
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Document {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_active_scene(&self, value: Option<&Scene>) {
                    unsafe {
                        let (result0_0, result0_1) = match value {
                            Some(e) => (1i32, (e).handle() as i32),
                            None => (0i32, 0i32),
                        };
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/document")]
                        extern "C" {
                            #[link_name = "[method]document.set-active-scene"]
                            fn wit_import(_: i32, _: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, result0_0, result0_1);
                    }
                }
            }
            impl Document {
                #[allow(unused_unsafe, clippy::all)]
                /// The initial active scene, set on document load.
                /// If none is provided, the first scene will be used.
                pub fn default_scene(&self) -> Option<Scene> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/document")]
                        extern "C" {
                            #[link_name = "[method]document.default-scene"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<i32>();
                                    super::super::super::wired::scene::scene::Scene::from_handle(
                                        l2 as u32,
                                    )
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Document {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_default_scene(&self, value: Option<&Scene>) {
                    unsafe {
                        let (result0_0, result0_1) = match value {
                            Some(e) => (1i32, (e).handle() as i32),
                            None => (0i32, 0i32),
                        };
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/document")]
                        extern "C" {
                            #[link_name = "[method]document.set-default-scene"]
                            fn wit_import(_: i32, _: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, result0_0, result0_1);
                    }
                }
            }
            impl Document {
                #[allow(unused_unsafe, clippy::all)]
                pub fn scenes(&self) -> _rt::Vec<Scene> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/document")]
                        extern "C" {
                            #[link_name = "[method]document.scenes"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<*mut u8>();
                        let l2 = *ptr0.add(4).cast::<usize>();
                        let base4 = l1;
                        let len4 = l2;
                        let mut result4 = _rt::Vec::with_capacity(len4);
                        for i in 0..len4 {
                            let base = base4.add(i * 4);
                            let e4 = {
                                let l3 = *base.add(0).cast::<i32>();
                                super::super::super::wired::scene::scene::Scene::from_handle(
                                    l3 as u32,
                                )
                            };
                            result4.push(e4);
                        }
                        _rt::cabi_dealloc(base4, len4 * 4, 4);
                        result4
                    }
                }
            }
            impl Document {
                #[allow(unused_unsafe, clippy::all)]
                pub fn add_scene(&self, value: &Scene) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/document")]
                        extern "C" {
                            #[link_name = "[method]document.add-scene"]
                            fn wit_import(_: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, (value).handle() as i32);
                    }
                }
            }
            impl Document {
                #[allow(unused_unsafe, clippy::all)]
                pub fn remove_scene(&self, value: &Scene) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/document")]
                        extern "C" {
                            #[link_name = "[method]document.remove-scene"]
                            fn wit_import(_: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, (value).handle() as i32);
                    }
                }
            }
        }
        #[allow(dead_code, clippy::all)]
        pub mod composition {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() =
                super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            pub type Document = super::super::super::wired::scene::document::Document;
            pub type Transform = super::super::super::wired::math::types::Transform;
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct AssetNode {
                handle: _rt::Resource<AssetNode>,
            }
            impl AssetNode {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for AssetNode {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "wired:scene/composition")]
                        extern "C" {
                            #[link_name = "[resource-drop]asset-node"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            /// A composition of assets.
            /// Roughly equivalent to a glXF file.
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct Composition {
                handle: _rt::Resource<Composition>,
            }
            impl Composition {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for Composition {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "wired:scene/composition")]
                        extern "C" {
                            #[link_name = "[resource-drop]composition"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            pub enum Asset {
                Composition(Composition),
                Document(Document),
            }
            impl ::core::fmt::Debug for Asset {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        Asset::Composition(e) => {
                            f.debug_tuple("Asset::Composition").field(e).finish()
                        }
                        Asset::Document(e) => f.debug_tuple("Asset::Document").field(e).finish(),
                    }
                }
            }
            impl AssetNode {
                #[allow(unused_unsafe, clippy::all)]
                pub fn new() -> Self {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/composition")]
                        extern "C" {
                            #[link_name = "[constructor]asset-node"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        AssetNode::from_handle(ret as u32)
                    }
                }
            }
            impl AssetNode {
                #[allow(unused_unsafe, clippy::all)]
                pub fn asset(&self) -> Option<Asset> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 12]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/composition")]
                        extern "C" {
                            #[link_name = "[method]asset-node.asset"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = i32::from(*ptr0.add(4).cast::<u8>());
                                    let v5 = match l2 {
                                        0 => {
                                            let e5 = {
                                                let l3 = *ptr0.add(8).cast::<i32>();
                                                Composition::from_handle(l3 as u32)
                                            };
                                            Asset::Composition(e5)
                                        }
                                        n => {
                                            debug_assert_eq!(n, 1, "invalid enum discriminant");
                                            let e5 = {
                                                let l4 = *ptr0.add(8).cast::<i32>();
                                                super::super::super::wired::scene::document::Document::from_handle(
                                                    l4 as u32,
                                                )
                                            };
                                            Asset::Document(e5)
                                        }
                                    };
                                    v5
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl AssetNode {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_asset(&self, value: Option<Asset>) {
                    unsafe {
                        let (result1_0, result1_1, result1_2) = match &value {
                            Some(e) => {
                                let (result0_0, result0_1) = match e {
                                    Asset::Composition(e) => (0i32, (e).take_handle() as i32),
                                    Asset::Document(e) => (1i32, (e).take_handle() as i32),
                                };
                                (1i32, result0_0, result0_1)
                            }
                            None => (0i32, 0i32, 0i32),
                        };
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/composition")]
                        extern "C" {
                            #[link_name = "[method]asset-node.set-asset"]
                            fn wit_import(_: i32, _: i32, _: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32, _: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, result1_0, result1_1, result1_2);
                    }
                }
            }
            impl AssetNode {
                #[allow(unused_unsafe, clippy::all)]
                pub fn name(&self) -> _rt::String {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/composition")]
                        extern "C" {
                            #[link_name = "[method]asset-node.name"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<*mut u8>();
                        let l2 = *ptr0.add(4).cast::<usize>();
                        let len3 = l2;
                        let bytes3 = _rt::Vec::from_raw_parts(l1.cast(), len3, len3);
                        _rt::string_lift(bytes3)
                    }
                }
            }
            impl AssetNode {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_name(&self, value: &str) {
                    unsafe {
                        let vec0 = value;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/composition")]
                        extern "C" {
                            #[link_name = "[method]asset-node.set-name"]
                            fn wit_import(_: i32, _: *mut u8, _: usize);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8, _: usize) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0.cast_mut(), len0);
                    }
                }
            }
            impl AssetNode {
                #[allow(unused_unsafe, clippy::all)]
                pub fn parent(&self) -> Option<AssetNode> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/composition")]
                        extern "C" {
                            #[link_name = "[method]asset-node.parent"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<i32>();
                                    AssetNode::from_handle(l2 as u32)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl AssetNode {
                #[allow(unused_unsafe, clippy::all)]
                pub fn children(&self) -> _rt::Vec<AssetNode> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/composition")]
                        extern "C" {
                            #[link_name = "[method]asset-node.children"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<*mut u8>();
                        let l2 = *ptr0.add(4).cast::<usize>();
                        let base4 = l1;
                        let len4 = l2;
                        let mut result4 = _rt::Vec::with_capacity(len4);
                        for i in 0..len4 {
                            let base = base4.add(i * 4);
                            let e4 = {
                                let l3 = *base.add(0).cast::<i32>();
                                AssetNode::from_handle(l3 as u32)
                            };
                            result4.push(e4);
                        }
                        _rt::cabi_dealloc(base4, len4 * 4, 4);
                        result4
                    }
                }
            }
            impl AssetNode {
                #[allow(unused_unsafe, clippy::all)]
                pub fn add_child(&self, value: &AssetNode) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/composition")]
                        extern "C" {
                            #[link_name = "[method]asset-node.add-child"]
                            fn wit_import(_: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, (value).handle() as i32);
                    }
                }
            }
            impl AssetNode {
                #[allow(unused_unsafe, clippy::all)]
                pub fn remove_child(&self, value: &AssetNode) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/composition")]
                        extern "C" {
                            #[link_name = "[method]asset-node.remove-child"]
                            fn wit_import(_: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, (value).handle() as i32);
                    }
                }
            }
            impl AssetNode {
                #[allow(unused_unsafe, clippy::all)]
                pub fn global_transform(&self) -> Transform {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 40]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 40]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/composition")]
                        extern "C" {
                            #[link_name = "[method]asset-node.global-transform"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<f32>();
                        let l2 = *ptr0.add(4).cast::<f32>();
                        let l3 = *ptr0.add(8).cast::<f32>();
                        let l4 = *ptr0.add(12).cast::<f32>();
                        let l5 = *ptr0.add(16).cast::<f32>();
                        let l6 = *ptr0.add(20).cast::<f32>();
                        let l7 = *ptr0.add(24).cast::<f32>();
                        let l8 = *ptr0.add(28).cast::<f32>();
                        let l9 = *ptr0.add(32).cast::<f32>();
                        let l10 = *ptr0.add(36).cast::<f32>();
                        super::super::super::wired::math::types::Transform {
                            rotation: super::super::super::wired::math::types::Quat {
                                x: l1,
                                y: l2,
                                z: l3,
                                w: l4,
                            },
                            scale: super::super::super::wired::math::types::Vec3 {
                                x: l5,
                                y: l6,
                                z: l7,
                            },
                            translation: super::super::super::wired::math::types::Vec3 {
                                x: l8,
                                y: l9,
                                z: l10,
                            },
                        }
                    }
                }
            }
            impl AssetNode {
                #[allow(unused_unsafe, clippy::all)]
                pub fn transform(&self) -> Transform {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 40]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 40]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/composition")]
                        extern "C" {
                            #[link_name = "[method]asset-node.transform"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<f32>();
                        let l2 = *ptr0.add(4).cast::<f32>();
                        let l3 = *ptr0.add(8).cast::<f32>();
                        let l4 = *ptr0.add(12).cast::<f32>();
                        let l5 = *ptr0.add(16).cast::<f32>();
                        let l6 = *ptr0.add(20).cast::<f32>();
                        let l7 = *ptr0.add(24).cast::<f32>();
                        let l8 = *ptr0.add(28).cast::<f32>();
                        let l9 = *ptr0.add(32).cast::<f32>();
                        let l10 = *ptr0.add(36).cast::<f32>();
                        super::super::super::wired::math::types::Transform {
                            rotation: super::super::super::wired::math::types::Quat {
                                x: l1,
                                y: l2,
                                z: l3,
                                w: l4,
                            },
                            scale: super::super::super::wired::math::types::Vec3 {
                                x: l5,
                                y: l6,
                                z: l7,
                            },
                            translation: super::super::super::wired::math::types::Vec3 {
                                x: l8,
                                y: l9,
                                z: l10,
                            },
                        }
                    }
                }
            }
            impl AssetNode {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_transform(&self, value: Transform) {
                    unsafe {
                        let super::super::super::wired::math::types::Transform {
                            rotation: rotation0,
                            scale: scale0,
                            translation: translation0,
                        } = value;
                        let super::super::super::wired::math::types::Quat {
                            x: x1,
                            y: y1,
                            z: z1,
                            w: w1,
                        } = rotation0;
                        let super::super::super::wired::math::types::Vec3 {
                            x: x2,
                            y: y2,
                            z: z2,
                        } = scale0;
                        let super::super::super::wired::math::types::Vec3 {
                            x: x3,
                            y: y3,
                            z: z3,
                        } = translation0;
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/composition")]
                        extern "C" {
                            #[link_name = "[method]asset-node.set-transform"]
                            fn wit_import(
                                _: i32,
                                _: f32,
                                _: f32,
                                _: f32,
                                _: f32,
                                _: f32,
                                _: f32,
                                _: f32,
                                _: f32,
                                _: f32,
                                _: f32,
                            );
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(
                            _: i32,
                            _: f32,
                            _: f32,
                            _: f32,
                            _: f32,
                            _: f32,
                            _: f32,
                            _: f32,
                            _: f32,
                            _: f32,
                            _: f32,
                        ) {
                            unreachable!()
                        }
                        wit_import(
                            (self).handle() as i32,
                            _rt::as_f32(x1),
                            _rt::as_f32(y1),
                            _rt::as_f32(z1),
                            _rt::as_f32(w1),
                            _rt::as_f32(x2),
                            _rt::as_f32(y2),
                            _rt::as_f32(z2),
                            _rt::as_f32(x3),
                            _rt::as_f32(y3),
                            _rt::as_f32(z3),
                        );
                    }
                }
            }
            impl Composition {
                #[allow(unused_unsafe, clippy::all)]
                pub fn new() -> Self {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/composition")]
                        extern "C" {
                            #[link_name = "[constructor]composition"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        Composition::from_handle(ret as u32)
                    }
                }
            }
            impl Composition {
                #[allow(unused_unsafe, clippy::all)]
                pub fn nodes(&self) -> _rt::Vec<AssetNode> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/composition")]
                        extern "C" {
                            #[link_name = "[method]composition.nodes"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<*mut u8>();
                        let l2 = *ptr0.add(4).cast::<usize>();
                        let base4 = l1;
                        let len4 = l2;
                        let mut result4 = _rt::Vec::with_capacity(len4);
                        for i in 0..len4 {
                            let base = base4.add(i * 4);
                            let e4 = {
                                let l3 = *base.add(0).cast::<i32>();
                                AssetNode::from_handle(l3 as u32)
                            };
                            result4.push(e4);
                        }
                        _rt::cabi_dealloc(base4, len4 * 4, 4);
                        result4
                    }
                }
            }
            impl Composition {
                #[allow(unused_unsafe, clippy::all)]
                pub fn add_node(&self, value: &AssetNode) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/composition")]
                        extern "C" {
                            #[link_name = "[method]composition.add-node"]
                            fn wit_import(_: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, (value).handle() as i32);
                    }
                }
            }
            impl Composition {
                #[allow(unused_unsafe, clippy::all)]
                pub fn remove_node(&self, value: &AssetNode) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:scene/composition")]
                        extern "C" {
                            #[link_name = "[method]composition.remove-node"]
                            fn wit_import(_: i32, _: i32);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, (value).handle() as i32);
                    }
                }
            }
        }
        #[allow(dead_code, clippy::all)]
        pub mod api {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() =
                super::super::super::__link_custom_section_describing_imports;
            pub type Composition = super::super::super::wired::scene::composition::Composition;
            #[allow(unused_unsafe, clippy::all)]
            /// The root composition that the script is attached to.
            pub fn root() -> Composition {
                unsafe {
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "wired:scene/api")]
                    extern "C" {
                        #[link_name = "root"]
                        fn wit_import() -> i32;
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import() -> i32 {
                        unreachable!()
                    }
                    let ret = wit_import();
                    super::super::super::wired::scene::composition::Composition::from_handle(
                        ret as u32,
                    )
                }
            }
        }
    }
}
#[allow(dead_code)]
pub mod exports {
    #[allow(dead_code)]
    pub mod wired {
        #[allow(dead_code)]
        pub mod script {
            #[allow(dead_code, clippy::all)]
            pub mod types {
                #[used]
                #[doc(hidden)]
                static __FORCE_SECTION_REF: fn() =
                    super::super::super::super::__link_custom_section_describing_imports;
                use super::super::super::super::_rt;
                #[derive(Debug)]
                #[repr(transparent)]
                pub struct Script {
                    handle: _rt::Resource<Script>,
                }
                type _ScriptRep<T> = Option<T>;
                impl Script {
                    /// Creates a new resource from the specified representation.
                    ///
                    /// This function will create a new resource handle by moving `val` onto
                    /// the heap and then passing that heap pointer to the component model to
                    /// create a handle. The owned handle is then returned as `Script`.
                    pub fn new<T: GuestScript>(val: T) -> Self {
                        Self::type_guard::<T>();
                        let val: _ScriptRep<T> = Some(val);
                        let ptr: *mut _ScriptRep<T> = _rt::Box::into_raw(_rt::Box::new(val));
                        unsafe { Self::from_handle(T::_resource_new(ptr.cast())) }
                    }
                    /// Gets access to the underlying `T` which represents this resource.
                    pub fn get<T: GuestScript>(&self) -> &T {
                        let ptr = unsafe { &*self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    /// Gets mutable access to the underlying `T` which represents this
                    /// resource.
                    pub fn get_mut<T: GuestScript>(&mut self) -> &mut T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_mut().unwrap()
                    }
                    /// Consumes this resource and returns the underlying `T`.
                    pub fn into_inner<T: GuestScript>(self) -> T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.take().unwrap()
                    }
                    #[doc(hidden)]
                    pub unsafe fn from_handle(handle: u32) -> Self {
                        Self {
                            handle: _rt::Resource::from_handle(handle),
                        }
                    }
                    #[doc(hidden)]
                    pub fn take_handle(&self) -> u32 {
                        _rt::Resource::take_handle(&self.handle)
                    }
                    #[doc(hidden)]
                    pub fn handle(&self) -> u32 {
                        _rt::Resource::handle(&self.handle)
                    }
                    #[doc(hidden)]
                    fn type_guard<T: 'static>() {
                        use core::any::TypeId;
                        static mut LAST_TYPE: Option<TypeId> = None;
                        unsafe {
                            assert!(!cfg!(target_feature = "atomics"));
                            let id = TypeId::of::<T>();
                            match LAST_TYPE {
                                Some(ty) => {
                                    assert!(
                                        ty == id,
                                        "cannot use two types with this resource type"
                                    )
                                }
                                None => LAST_TYPE = Some(id),
                            }
                        }
                    }
                    #[doc(hidden)]
                    pub unsafe fn dtor<T: 'static>(handle: *mut u8) {
                        Self::type_guard::<T>();
                        let _ = _rt::Box::from_raw(handle as *mut _ScriptRep<T>);
                    }
                    fn as_ptr<T: GuestScript>(&self) -> *mut _ScriptRep<T> {
                        Script::type_guard::<T>();
                        T::_resource_rep(self.handle()).cast()
                    }
                }
                /// A borrowed version of [`Script`] which represents a borrowed value
                /// with the lifetime `'a`.
                #[derive(Debug)]
                #[repr(transparent)]
                pub struct ScriptBorrow<'a> {
                    rep: *mut u8,
                    _marker: core::marker::PhantomData<&'a Script>,
                }
                impl<'a> ScriptBorrow<'a> {
                    #[doc(hidden)]
                    pub unsafe fn lift(rep: usize) -> Self {
                        Self {
                            rep: rep as *mut u8,
                            _marker: core::marker::PhantomData,
                        }
                    }
                    /// Gets access to the underlying `T` in this resource.
                    pub fn get<T: GuestScript>(&self) -> &T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    fn as_ptr<T: 'static>(&self) -> *mut _ScriptRep<T> {
                        Script::type_guard::<T>();
                        self.rep.cast()
                    }
                }
                unsafe impl _rt::WasmResource for Script {
                    #[inline]
                    unsafe fn drop(_handle: u32) {
                        #[cfg(not(target_arch = "wasm32"))]
                        unreachable!();
                        #[cfg(target_arch = "wasm32")]
                        {
                            #[link(wasm_import_module = "[export]wired:script/types")]
                            extern "C" {
                                #[link_name = "[resource-drop]script"]
                                fn drop(_: u32);
                            }
                            drop(_handle);
                        }
                    }
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_constructor_script_cabi<T: GuestScript>() -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = Script::new(T::new());
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_script_update_cabi<T: GuestScript>(
                    arg0: *mut u8,
                    arg1: f32,
                ) {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    T::update(ScriptBorrow::lift(arg0 as u32 as usize).get(), arg1);
                }
                pub trait Guest {
                    type Script: GuestScript;
                }
                pub trait GuestScript: 'static {
                    #[doc(hidden)]
                    unsafe fn _resource_new(val: *mut u8) -> u32
                    where
                        Self: Sized,
                    {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            let _ = val;
                            unreachable!();
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            #[link(wasm_import_module = "[export]wired:script/types")]
                            extern "C" {
                                #[link_name = "[resource-new]script"]
                                fn new(_: *mut u8) -> u32;
                            }
                            new(val)
                        }
                    }
                    #[doc(hidden)]
                    fn _resource_rep(handle: u32) -> *mut u8
                    where
                        Self: Sized,
                    {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            let _ = handle;
                            unreachable!();
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            #[link(wasm_import_module = "[export]wired:script/types")]
                            extern "C" {
                                #[link_name = "[resource-rep]script"]
                                fn rep(_: u32) -> *mut u8;
                            }
                            unsafe { rep(handle) }
                        }
                    }
                    fn new() -> Self;
                    /// Called every tick.
                    fn update(&self, delta: f32);
                }
                #[doc(hidden)]
                macro_rules! __export_wired_script_types_cabi {
                    ($ty:ident with_types_in $($path_to_types:tt)*) => {
                        const _ : () = { #[export_name =
                        "wired:script/types#[constructor]script"] unsafe extern "C" fn
                        export_constructor_script() -> i32 { $($path_to_types)*::
                        _export_constructor_script_cabi::<<$ty as $($path_to_types)*::
                        Guest >::Script > () } #[export_name =
                        "wired:script/types#[method]script.update"] unsafe extern "C" fn
                        export_method_script_update(arg0 : * mut u8, arg1 : f32,) {
                        $($path_to_types)*:: _export_method_script_update_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Script > (arg0, arg1) } const _ :
                        () = { #[doc(hidden)] #[export_name =
                        "wired:script/types#[dtor]script"] #[allow(non_snake_case)]
                        unsafe extern "C" fn dtor(rep : * mut u8) { $($path_to_types)*::
                        Script::dtor::< <$ty as $($path_to_types)*:: Guest >::Script >
                        (rep) } }; };
                    };
                }
                #[doc(hidden)]
                pub(crate) use __export_wired_script_types_cabi;
            }
        }
    }
}
mod _rt {
    use core::fmt;
    use core::marker;
    use core::sync::atomic::{AtomicU32, Ordering::Relaxed};
    /// A type which represents a component model resource, either imported or
    /// exported into this component.
    ///
    /// This is a low-level wrapper which handles the lifetime of the resource
    /// (namely this has a destructor). The `T` provided defines the component model
    /// intrinsics that this wrapper uses.
    ///
    /// One of the chief purposes of this type is to provide `Deref` implementations
    /// to access the underlying data when it is owned.
    ///
    /// This type is primarily used in generated code for exported and imported
    /// resources.
    #[repr(transparent)]
    pub struct Resource<T: WasmResource> {
        handle: AtomicU32,
        _marker: marker::PhantomData<T>,
    }
    /// A trait which all wasm resources implement, namely providing the ability to
    /// drop a resource.
    ///
    /// This generally is implemented by generated code, not user-facing code.
    #[allow(clippy::missing_safety_doc)]
    pub unsafe trait WasmResource {
        /// Invokes the `[resource-drop]...` intrinsic.
        unsafe fn drop(handle: u32);
    }
    impl<T: WasmResource> Resource<T> {
        #[doc(hidden)]
        pub unsafe fn from_handle(handle: u32) -> Self {
            debug_assert!(handle != u32::MAX);
            Self {
                handle: AtomicU32::new(handle),
                _marker: marker::PhantomData,
            }
        }
        /// Takes ownership of the handle owned by `resource`.
        ///
        /// Note that this ideally would be `into_handle` taking `Resource<T>` by
        /// ownership. The code generator does not enable that in all situations,
        /// unfortunately, so this is provided instead.
        ///
        /// Also note that `take_handle` is in theory only ever called on values
        /// owned by a generated function. For example a generated function might
        /// take `Resource<T>` as an argument but then call `take_handle` on a
        /// reference to that argument. In that sense the dynamic nature of
        /// `take_handle` should only be exposed internally to generated code, not
        /// to user code.
        #[doc(hidden)]
        pub fn take_handle(resource: &Resource<T>) -> u32 {
            resource.handle.swap(u32::MAX, Relaxed)
        }
        #[doc(hidden)]
        pub fn handle(resource: &Resource<T>) -> u32 {
            resource.handle.load(Relaxed)
        }
    }
    impl<T: WasmResource> fmt::Debug for Resource<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Resource")
                .field("handle", &self.handle)
                .finish()
        }
    }
    impl<T: WasmResource> Drop for Resource<T> {
        fn drop(&mut self) {
            unsafe {
                match self.handle.load(Relaxed) {
                    u32::MAX => {}
                    other => T::drop(other),
                }
            }
        }
    }
    pub use alloc_crate::string::String;
    pub use alloc_crate::vec::Vec;
    pub unsafe fn string_lift(bytes: Vec<u8>) -> String {
        if cfg!(debug_assertions) {
            String::from_utf8(bytes).unwrap()
        } else {
            String::from_utf8_unchecked(bytes)
        }
    }
    pub fn as_f32<T: AsF32>(t: T) -> f32 {
        t.as_f32()
    }
    pub trait AsF32 {
        fn as_f32(self) -> f32;
    }
    impl<'a, T: Copy + AsF32> AsF32 for &'a T {
        fn as_f32(self) -> f32 {
            (*self).as_f32()
        }
    }
    impl AsF32 for f32 {
        #[inline]
        fn as_f32(self) -> f32 {
            self as f32
        }
    }
    pub unsafe fn invalid_enum_discriminant<T>() -> T {
        if cfg!(debug_assertions) {
            panic!("invalid enum discriminant")
        } else {
            core::hint::unreachable_unchecked()
        }
    }
    pub unsafe fn cabi_dealloc(ptr: *mut u8, size: usize, align: usize) {
        if size == 0 {
            return;
        }
        let layout = alloc::Layout::from_size_align_unchecked(size, align);
        alloc::dealloc(ptr, layout);
    }
    pub fn as_i32<T: AsI32>(t: T) -> i32 {
        t.as_i32()
    }
    pub trait AsI32 {
        fn as_i32(self) -> i32;
    }
    impl<'a, T: Copy + AsI32> AsI32 for &'a T {
        fn as_i32(self) -> i32 {
            (*self).as_i32()
        }
    }
    impl AsI32 for i32 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for u32 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for i16 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for u16 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for i8 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for u8 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for char {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for usize {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    pub unsafe fn bool_lift(val: u8) -> bool {
        if cfg!(debug_assertions) {
            match val {
                0 => false,
                1 => true,
                _ => panic!("invalid bool discriminant"),
            }
        } else {
            val != 0
        }
    }
    pub use alloc_crate::boxed::Box;
    #[cfg(target_arch = "wasm32")]
    pub fn run_ctors_once() {
        wit_bindgen_rt::run_ctors_once();
    }
    extern crate alloc as alloc_crate;
    pub use alloc_crate::alloc;
}
/// Generates `#[no_mangle]` functions to export the specified type as the
/// root implementation of all generated traits.
///
/// For more information see the documentation of `wit_bindgen::generate!`.
///
/// ```rust
/// # macro_rules! export{ ($($t:tt)*) => (); }
/// # trait Guest {}
/// struct MyType;
///
/// impl Guest for MyType {
///     // ...
/// }
///
/// export!(MyType);
/// ```
#[allow(unused_macros)]
#[doc(hidden)]
macro_rules! __export_script_impl {
    ($ty:ident) => {
        self::export!($ty with_types_in self);
    };
    ($ty:ident with_types_in $($path_to_types_root:tt)*) => {
        $($path_to_types_root)*::
        exports::wired::script::types::__export_wired_script_types_cabi!($ty
        with_types_in $($path_to_types_root)*:: exports::wired::script::types);
    };
}
#[doc(inline)]
pub(crate) use __export_script_impl as export;
#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.30.0:script:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 9508] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\xa7I\x01A\x02\x01A1\x01\
B\x10\x01r\x02\x01xv\x01yv\x04\0\x04vec2\x03\0\0\x01r\x03\x01xv\x01yv\x01zv\x04\0\
\x04vec3\x03\0\x02\x01r\x04\x01xv\x01yv\x01zv\x01wv\x04\0\x04quat\x03\0\x04\x01r\
\x03\x08rotation\x05\x05scale\x03\x0btranslation\x03\x04\0\x09transform\x03\0\x06\
\x01@\0\0\x01\x04\0\x09fake-fn-a\x01\x08\x01@\0\0\x03\x04\0\x09fake-fn-b\x01\x09\
\x01@\0\0\x05\x04\0\x09fake-fn-c\x01\x0a\x01@\0\0\x07\x04\0\x09fake-fn-d\x01\x0b\
\x03\x01\x10wired:math/types\x05\0\x01B\x13\x01r\x04\x01rv\x01gv\x01bv\x01av\x04\
\0\x05color\x03\0\0\x04\0\x08material\x03\x01\x01i\x02\x01@\0\0\x03\x04\0\x15[co\
nstructor]material\x01\x04\x01h\x02\x01@\x01\x04self\x05\0y\x04\0\x13[method]mat\
erial.id\x01\x06\x01@\x01\x04self\x05\0\x03\x04\0\x14[method]material.ref\x01\x07\
\x01@\x01\x04self\x05\0s\x04\0\x15[method]material.name\x01\x08\x01@\x02\x04self\
\x05\x05values\x01\0\x04\0\x19[method]material.set-name\x01\x09\x01@\x01\x04self\
\x05\0\x01\x04\0\x16[method]material.color\x01\x0a\x01@\x02\x04self\x05\x05value\
\x01\x01\0\x04\0\x1a[method]material.set-color\x01\x0b\x03\x01\x14wired:scene/ma\
terial\x05\x01\x02\x03\0\x01\x08material\x01B+\x02\x03\x02\x01\x02\x04\0\x08mate\
rial\x03\0\0\x04\0\x09primitive\x03\x01\x04\0\x04mesh\x03\x01\x01h\x02\x01@\x01\x04\
self\x04\0y\x04\0\x14[method]primitive.id\x01\x05\x01i\x01\x01k\x06\x01@\x01\x04\
self\x04\0\x07\x04\0\x1a[method]primitive.material\x01\x08\x01h\x01\x01k\x09\x01\
@\x02\x04self\x04\x05value\x0a\x01\0\x04\0\x1e[method]primitive.set-material\x01\
\x0b\x01py\x01@\x02\x04self\x04\x05value\x0c\x01\0\x04\0\x1d[method]primitive.se\
t-indices\x01\x0d\x01pv\x01@\x02\x04self\x04\x05value\x0e\x01\0\x04\0\x1d[method\
]primitive.set-normals\x01\x0f\x04\0\x1f[method]primitive.set-positions\x01\x0f\x04\
\0\x19[method]primitive.set-uvs\x01\x0f\x01i\x03\x01@\0\0\x10\x04\0\x11[construc\
tor]mesh\x01\x11\x01h\x03\x01@\x01\x04self\x12\0y\x04\0\x0f[method]mesh.id\x01\x13\
\x01@\x01\x04self\x12\0\x10\x04\0\x10[method]mesh.ref\x01\x14\x01@\x01\x04self\x12\
\0s\x04\0\x11[method]mesh.name\x01\x15\x01@\x02\x04self\x12\x05values\x01\0\x04\0\
\x15[method]mesh.set-name\x01\x16\x01i\x02\x01p\x17\x01@\x01\x04self\x12\0\x18\x04\
\0\x1c[method]mesh.list-primitives\x01\x19\x01@\x01\x04self\x12\0\x17\x04\0\x1d[\
method]mesh.create-primitive\x01\x1a\x01@\x02\x04self\x12\x05value\x17\x01\0\x04\
\0\x1d[method]mesh.remove-primitive\x01\x1b\x03\x01\x10wired:scene/mesh\x05\x03\x02\
\x03\0\0\x04vec3\x02\x03\0\0\x04quat\x01B\x15\x02\x03\x02\x01\x04\x04\0\x04vec3\x03\
\0\0\x02\x03\x02\x01\x05\x04\0\x04quat\x03\0\x02\x01m\x02\x04left\x05right\x04\0\
\x09hand-side\x03\0\x04\x01r\x03\x0btranslation\x01\x08rotation\x03\x06radiusv\x04\
\0\x05joint\x03\0\x06\x01r\x04\x03tip\x07\x06distal\x07\x08proximal\x07\x0ametac\
arpal\x07\x04\0\x06finger\x03\0\x08\x01k\x07\x01r\x09\x04side\x05\x05thumb\x09\x05\
index\x09\x06middle\x09\x04ring\x09\x06little\x09\x04palm\x07\x05wrist\x07\x05el\
bow\x0a\x04\0\x04hand\x03\0\x0b\x01r\x02\x0borientation\x03\x06origin\x01\x04\0\x03\
ray\x03\0\x0d\x01q\x02\x04hand\x01\x0c\0\x03ray\x01\x0e\0\x04\0\x0ainput-data\x03\
\0\x0f\x01q\x02\x09collision\0\0\x05hover\0\0\x04\0\x0cinput-action\x03\0\x11\x01\
r\x03\x02idw\x06action\x12\x04data\x10\x04\0\x0binput-event\x03\0\x13\x03\x01\x11\
wired:input/types\x05\x06\x02\x03\0\x03\x0binput-event\x01B\x0a\x02\x03\x02\x01\x07\
\x04\0\x0binput-event\x03\0\0\x04\0\x0dinput-handler\x03\x01\x01i\x02\x01@\0\0\x03\
\x04\0\x1a[constructor]input-handler\x01\x04\x01h\x02\x01k\x01\x01@\x01\x04self\x05\
\0\x06\x04\0\x1a[method]input-handler.next\x01\x07\x03\x01\x13wired:input/handle\
r\x05\x08\x01B\x1c\x02\x03\x02\x01\x04\x04\0\x04vec3\x03\0\0\x04\0\x08collider\x03\
\x01\x01r\x02\x06heightv\x06radiusv\x04\0\x0eshape-cylinder\x03\0\x03\x01q\x03\x06\
cuboid\x01\x01\0\x08cylinder\x01\x04\0\x06sphere\x01v\0\x04\0\x05shape\x03\0\x05\
\x04\0\x0arigid-body\x03\x01\x01m\x03\x07dynamic\x05fixed\x09kinematic\x04\0\x0f\
rigid-body-type\x03\0\x08\x01i\x02\x01@\x01\x05shape\x06\0\x0a\x04\0\x15[constru\
ctor]collider\x01\x0b\x01h\x02\x01@\x01\x04self\x0c\0v\x04\0\x18[method]collider\
.density\x01\x0d\x01@\x02\x04self\x0c\x05valuev\x01\0\x04\0\x1c[method]collider.\
set-density\x01\x0e\x01i\x07\x01@\x01\x0frigid-body-type\x09\0\x0f\x04\0\x17[con\
structor]rigid-body\x01\x10\x01h\x07\x01@\x01\x04self\x11\0\x01\x04\0\x19[method\
]rigid-body.angvel\x01\x12\x01@\x02\x04self\x11\x05value\x01\x01\0\x04\0\x1d[met\
hod]rigid-body.set-angvel\x01\x13\x04\0\x19[method]rigid-body.linvel\x01\x12\x04\
\0\x1d[method]rigid-body.set-linvel\x01\x13\x03\x01\x13wired:physics/types\x05\x09\
\x02\x03\0\x02\x04mesh\x02\x03\0\x04\x0dinput-handler\x02\x03\0\0\x09transform\x02\
\x03\0\x05\x08collider\x02\x03\0\x05\x0arigid-body\x01BE\x02\x03\x02\x01\x0a\x04\
\0\x04mesh\x03\0\0\x02\x03\x02\x01\x0b\x04\0\x0dinput-handler\x03\0\x02\x02\x03\x02\
\x01\x0c\x04\0\x09transform\x03\0\x04\x02\x03\x02\x01\x0d\x04\0\x08collider\x03\0\
\x06\x02\x03\x02\x01\x0e\x04\0\x0arigid-body\x03\0\x08\x04\0\x04node\x03\x01\x01\
i\x0a\x01@\0\0\x0b\x04\0\x11[constructor]node\x01\x0c\x01h\x0a\x01@\x01\x04self\x0d\
\0y\x04\0\x0f[method]node.id\x01\x0e\x01@\x01\x04self\x0d\0\x0b\x04\0\x10[method\
]node.ref\x01\x0f\x01@\x01\x04self\x0d\0s\x04\0\x11[method]node.name\x01\x10\x01\
@\x02\x04self\x0d\x05values\x01\0\x04\0\x15[method]node.set-name\x01\x11\x01k\x0b\
\x01@\x01\x04self\x0d\0\x12\x04\0\x13[method]node.parent\x01\x13\x01p\x0b\x01@\x01\
\x04self\x0d\0\x14\x04\0\x15[method]node.children\x01\x15\x01@\x02\x04self\x0d\x05\
value\x0d\x01\0\x04\0\x16[method]node.add-child\x01\x16\x04\0\x19[method]node.re\
move-child\x01\x16\x01@\x01\x04self\x0d\0\x05\x04\0\x1d[method]node.global-trans\
form\x01\x17\x04\0\x16[method]node.transform\x01\x17\x01@\x02\x04self\x0d\x05val\
ue\x05\x01\0\x04\0\x1a[method]node.set-transform\x01\x18\x01i\x01\x01k\x19\x01@\x01\
\x04self\x0d\0\x1a\x04\0\x11[method]node.mesh\x01\x1b\x01h\x01\x01k\x1c\x01@\x02\
\x04self\x0d\x05value\x1d\x01\0\x04\0\x15[method]node.set-mesh\x01\x1e\x01i\x07\x01\
k\x1f\x01@\x01\x04self\x0d\0\x20\x04\0\x15[method]node.collider\x01!\x01h\x07\x01\
k\"\x01@\x02\x04self\x0d\x05value#\x01\0\x04\0\x19[method]node.set-collider\x01$\
\x01i\x09\x01k%\x01@\x01\x04self\x0d\0&\x04\0\x17[method]node.rigid-body\x01'\x01\
h\x09\x01k(\x01@\x02\x04self\x0d\x05value)\x01\0\x04\0\x1b[method]node.set-rigid\
-body\x01*\x01i\x03\x01k+\x01@\x01\x04self\x0d\0,\x04\0\x1a[method]node.input-ha\
ndler\x01-\x01h\x03\x01k.\x01@\x02\x04self\x0d\x05value/\x01\0\x04\0\x1e[method]\
node.set-input-handler\x010\x03\x01\x10wired:scene/node\x05\x0f\x02\x03\0\0\x04v\
ec2\x02\x03\0\x06\x04node\x01B\x88\x01\x02\x03\x02\x01\x10\x04\0\x04vec2\x03\0\0\
\x02\x03\x02\x01\x04\x04\0\x04vec3\x03\0\x02\x02\x03\x02\x01\x0a\x04\0\x04mesh\x03\
\0\x04\x02\x03\x02\x01\x11\x04\0\x04node\x03\0\x06\x04\0\x09rectangle\x03\x01\x04\
\0\x06circle\x03\x01\x04\0\x07ellipse\x03\x01\x04\0\x08cylinder\x03\x01\x04\0\x06\
cuboid\x03\x01\x01r\x01\x0csubdivisions}\x04\0\x0asphere-ico\x03\0\x0d\x01r\x02\x07\
sectors}\x06stacks}\x04\0\x09sphere-uv\x03\0\x0f\x01q\x02\x03ico\x01\x0e\0\x02uv\
\x01\x10\0\x04\0\x0bsphere-kind\x03\0\x11\x04\0\x06sphere\x03\x01\x04\0\x04axes\x03\
\x01\x01i\x08\x01@\x01\x04size\x01\0\x15\x04\0\x16[constructor]rectangle\x01\x16\
\x01h\x08\x01@\x01\x04self\x17\0\x01\x04\0\x16[method]rectangle.size\x01\x18\x01\
@\x02\x04self\x17\x05value\x01\x01\0\x04\0\x1a[method]rectangle.set-size\x01\x19\
\x01i\x05\x01@\x01\x04self\x17\0\x1a\x04\0\x19[method]rectangle.to-mesh\x01\x1b\x01\
i\x07\x01@\x01\x04self\x17\0\x1c\x04\0\x19[method]rectangle.to-node\x01\x1d\x04\0\
![method]rectangle.to-physics-node\x01\x1d\x01i\x09\x01@\x01\x06radiusv\0\x1e\x04\
\0\x13[constructor]circle\x01\x1f\x01h\x09\x01@\x01\x04self\x20\0v\x04\0\x15[met\
hod]circle.radius\x01!\x01@\x02\x04self\x20\x05valuev\x01\0\x04\0\x19[method]cir\
cle.set-radius\x01\"\x01@\x01\x04self\x20\0{\x04\0\x19[method]circle.resolution\x01\
#\x01@\x02\x04self\x20\x05value{\x01\0\x04\0\x1d[method]circle.set-resolution\x01\
$\x01@\x01\x04self\x20\0\x1a\x04\0\x16[method]circle.to-mesh\x01%\x01@\x01\x04se\
lf\x20\0\x1c\x04\0\x16[method]circle.to-node\x01&\x04\0\x1e[method]circle.to-phy\
sics-node\x01&\x01i\x0a\x01@\x01\x09half-size\x01\0'\x04\0\x14[constructor]ellip\
se\x01(\x01h\x0a\x01@\x01\x04self)\0\x01\x04\0\x19[method]ellipse.half-size\x01*\
\x01@\x02\x04self)\x05value\x01\x01\0\x04\0\x1d[method]ellipse.set-half-size\x01\
+\x01@\x01\x04self)\0{\x04\0\x1a[method]ellipse.resolution\x01,\x01@\x02\x04self\
)\x05value{\x01\0\x04\0\x1e[method]ellipse.set-resolution\x01-\x01@\x01\x04self)\
\0\x1a\x04\0\x17[method]ellipse.to-mesh\x01.\x01@\x01\x04self)\0\x1c\x04\0\x17[m\
ethod]ellipse.to-node\x01/\x04\0\x1f[method]ellipse.to-physics-node\x01/\x01i\x0b\
\x01@\x02\x06radiusv\x06heightv\00\x04\0\x15[constructor]cylinder\x011\x01h\x0b\x01\
@\x01\x04self2\0\x7f\x04\0\x14[method]cylinder.cap\x013\x01@\x02\x04self2\x05val\
ue\x7f\x01\0\x04\0\x18[method]cylinder.set-cap\x014\x01@\x01\x04self2\0v\x04\0\x17\
[method]cylinder.height\x015\x01@\x02\x04self2\x05valuev\x01\0\x04\0\x1b[method]\
cylinder.set-height\x016\x04\0\x17[method]cylinder.radius\x015\x04\0\x1b[method]\
cylinder.set-radius\x016\x01@\x01\x04self2\0}\x04\0\x1b[method]cylinder.resoluti\
on\x017\x01@\x02\x04self2\x05value}\x01\0\x04\0\x1f[method]cylinder.set-resoluti\
on\x018\x04\0\x19[method]cylinder.segments\x017\x04\0\x1d[method]cylinder.set-se\
gments\x018\x01@\x01\x04self2\0\x1a\x04\0\x18[method]cylinder.to-mesh\x019\x01@\x01\
\x04self2\0\x1c\x04\0\x18[method]cylinder.to-node\x01:\x04\0\x20[method]cylinder\
.to-physics-node\x01:\x01i\x0c\x01@\x01\x04size\x03\0;\x04\0\x13[constructor]cub\
oid\x01<\x01h\x0c\x01@\x01\x04self=\0\x03\x04\0\x13[method]cuboid.size\x01>\x01@\
\x02\x04self=\x05value\x03\x01\0\x04\0\x17[method]cuboid.set-size\x01?\x01@\x01\x04\
self=\0\x1a\x04\0\x16[method]cuboid.to-mesh\x01@\x01@\x01\x04self=\0\x1c\x04\0\x16\
[method]cuboid.to-node\x01A\x04\0\x1e[method]cuboid.to-physics-node\x01A\x01i\x13\
\x01@\x01\x06radiusv\0\xc2\0\x04\0\x16[static]sphere.new-ico\x01C\x04\0\x15[stat\
ic]sphere.new-uv\x01C\x01h\x13\x01@\x01\x04self\xc4\0\0v\x04\0\x15[method]sphere\
.radius\x01E\x01@\x02\x04self\xc4\0\x05valuev\x01\0\x04\0\x19[method]sphere.set-\
radius\x01F\x01@\x01\x04self\xc4\0\0\x12\x04\0\x13[method]sphere.kind\x01G\x01@\x02\
\x04self\xc4\0\x05value\x12\x01\0\x04\0\x17[method]sphere.set-kind\x01H\x01@\x01\
\x04self\xc4\0\0\x1a\x04\0\x16[method]sphere.to-mesh\x01I\x01@\x01\x04self\xc4\0\
\0\x1c\x04\0\x16[method]sphere.to-node\x01J\x04\0\x1e[method]sphere.to-physics-n\
ode\x01J\x01i\x14\x01@\0\0\xcb\0\x04\0\x11[constructor]axes\x01L\x01h\x14\x01@\x01\
\x04self\xcd\0\0v\x04\0\x11[method]axes.size\x01N\x01@\x02\x04self\xcd\0\x05valu\
ev\x01\0\x04\0\x15[method]axes.set-size\x01O\x01@\x01\x04self\xcd\0\0\x1c\x04\0\x14\
[method]axes.to-node\x01P\x03\x01\x10unavi:shapes/api\x05\x12\x01B#\x02\x03\x02\x01\
\x04\x04\0\x04vec3\x03\0\0\x02\x03\x02\x01\x11\x04\0\x04node\x03\0\x02\x01m\x03\x06\
center\x03end\x05start\x04\0\x09alignment\x03\0\x04\x04\0\x09container\x03\x01\x01\
i\x06\x01@\x01\x04size\x01\0\x07\x04\0\x16[constructor]container\x01\x08\x01h\x06\
\x01@\x01\x04self\x09\0\x07\x04\0\x15[method]container.ref\x01\x0a\x01i\x03\x01@\
\x01\x04self\x09\0\x0b\x04\0\x16[method]container.root\x01\x0c\x04\0\x17[method]\
container.inner\x01\x0c\x01p\x07\x01@\x01\x04self\x09\0\x0d\x04\0\x1f[method]con\
tainer.list-children\x01\x0e\x01@\x02\x04self\x09\x05child\x09\x01\0\x04\0\x1b[m\
ethod]container.add-child\x01\x0f\x04\0\x1e[method]container.remove-child\x01\x0f\
\x01@\x01\x04self\x09\0\x01\x04\0\x16[method]container.size\x01\x10\x01@\x02\x04\
self\x09\x05value\x01\x01\0\x04\0\x1a[method]container.set-size\x01\x11\x01@\x01\
\x04self\x09\0\x05\x04\0\x19[method]container.align-x\x01\x12\x04\0\x19[method]c\
ontainer.align-y\x01\x12\x04\0\x19[method]container.align-z\x01\x12\x01@\x02\x04\
self\x09\x05value\x05\x01\0\x04\0\x1d[method]container.set-align-x\x01\x13\x04\0\
\x1d[method]container.set-align-y\x01\x13\x04\0\x1d[method]container.set-align-z\
\x01\x13\x03\x01\x16unavi:layout/container\x05\x13\x02\x03\0\x08\x09container\x01\
B\x16\x02\x03\x02\x01\x14\x04\0\x09container\x03\0\0\x02\x03\x02\x01\x04\x04\0\x04\
vec3\x03\0\x02\x04\0\x04grid\x03\x01\x01i\x04\x01@\x02\x04size\x03\x04rows\x03\0\
\x05\x04\0\x11[constructor]grid\x01\x06\x01h\x04\x01i\x01\x01@\x01\x04self\x07\0\
\x08\x04\0\x11[method]grid.root\x01\x09\x01p\x08\x01@\x01\x04self\x07\0\x0a\x04\0\
\x12[method]grid.cells\x01\x0b\x01k\x08\x01@\x04\x04self\x07\x01xy\x01yy\x01zy\0\
\x0c\x04\0\x11[method]grid.cell\x01\x0d\x01@\x01\x04self\x07\0\x03\x04\0\x11[met\
hod]grid.rows\x01\x0e\x01@\x02\x04self\x07\x05value\x03\x01\0\x04\0\x15[method]g\
rid.set-rows\x01\x0f\x03\x01\x11unavi:layout/grid\x05\x15\x01B\x04\x01m\x04\x05d\
ebug\x04info\x04warn\x05error\x04\0\x09log-level\x03\0\0\x01@\x02\x05level\x01\x07\
messages\x01\0\x04\0\x03log\x01\x02\x03\x01\x0dwired:log/api\x05\x16\x01B\x11\x02\
\x03\x02\x01\x11\x04\0\x04node\x03\0\0\x01i\x01\x01r\x14\x04hips\x02\x05spine\x02\
\x05chest\x02\x0bupper-chest\x02\x04neck\x02\x04head\x02\x0dleft-shoulder\x02\x0e\
left-upper-arm\x02\x0eleft-lower-arm\x02\x09left-hand\x02\x0eright-shoulder\x02\x0f\
right-upper-arm\x02\x0fright-lower-arm\x02\x0aright-hand\x02\x0eleft-upper-leg\x02\
\x0eleft-lower-leg\x02\x09left-foot\x02\x0fright-upper-leg\x02\x0fright-lower-le\
g\x02\x0aright-foot\x02\x04\0\x08skeleton\x03\0\x03\x04\0\x06player\x03\x01\x01h\
\x05\x01@\x01\x04self\x06\0\x02\x04\0\x13[method]player.root\x01\x07\x01@\x01\x04\
self\x06\0\x04\x04\0\x17[method]player.skeleton\x01\x08\x01i\x05\x01p\x09\x01@\0\
\0\x0a\x04\0\x0clist-players\x01\x0b\x01@\0\0\x09\x04\0\x0clocal-player\x01\x0c\x03\
\x01\x10wired:player/api\x05\x17\x01B\x15\x02\x03\x02\x01\x11\x04\0\x04node\x03\0\
\0\x04\0\x05scene\x03\x01\x01i\x02\x01@\0\0\x03\x04\0\x12[constructor]scene\x01\x04\
\x01h\x02\x01@\x01\x04self\x05\0y\x04\0\x10[method]scene.id\x01\x06\x01@\x01\x04\
self\x05\0s\x04\0\x12[method]scene.name\x01\x07\x01@\x02\x04self\x05\x05values\x01\
\0\x04\0\x16[method]scene.set-name\x01\x08\x01i\x01\x01p\x09\x01@\x01\x04self\x05\
\0\x0a\x04\0\x13[method]scene.nodes\x01\x0b\x01h\x01\x01@\x02\x04self\x05\x05val\
ue\x0c\x01\0\x04\0\x16[method]scene.add-node\x01\x0d\x04\0\x19[method]scene.remo\
ve-node\x01\x0d\x03\x01\x11wired:scene/scene\x05\x18\x02\x03\0\x0c\x05scene\x01B\
\x17\x02\x03\x02\x01\x19\x04\0\x05scene\x03\0\0\x04\0\x08document\x03\x01\x01i\x02\
\x01@\0\0\x03\x04\0\x15[constructor]document\x01\x04\x01h\x02\x01i\x01\x01k\x06\x01\
@\x01\x04self\x05\0\x07\x04\0\x1d[method]document.active-scene\x01\x08\x01h\x01\x01\
k\x09\x01@\x02\x04self\x05\x05value\x0a\x01\0\x04\0![method]document.set-active-\
scene\x01\x0b\x04\0\x1e[method]document.default-scene\x01\x08\x04\0\"[method]doc\
ument.set-default-scene\x01\x0b\x01p\x06\x01@\x01\x04self\x05\0\x0c\x04\0\x17[me\
thod]document.scenes\x01\x0d\x01@\x02\x04self\x05\x05value\x09\x01\0\x04\0\x1a[m\
ethod]document.add-scene\x01\x0e\x04\0\x1d[method]document.remove-scene\x01\x0e\x03\
\x01\x14wired:scene/document\x05\x1a\x02\x03\0\x0d\x08document\x01B-\x02\x03\x02\
\x01\x1b\x04\0\x08document\x03\0\0\x02\x03\x02\x01\x0c\x04\0\x09transform\x03\0\x02\
\x04\0\x0aasset-node\x03\x01\x04\0\x0bcomposition\x03\x01\x01i\x05\x01i\x01\x01q\
\x02\x0bcomposition\x01\x06\0\x08document\x01\x07\0\x04\0\x05asset\x03\0\x08\x01\
i\x04\x01@\0\0\x0a\x04\0\x17[constructor]asset-node\x01\x0b\x01h\x04\x01k\x09\x01\
@\x01\x04self\x0c\0\x0d\x04\0\x18[method]asset-node.asset\x01\x0e\x01@\x02\x04se\
lf\x0c\x05value\x0d\x01\0\x04\0\x1c[method]asset-node.set-asset\x01\x0f\x01@\x01\
\x04self\x0c\0s\x04\0\x17[method]asset-node.name\x01\x10\x01@\x02\x04self\x0c\x05\
values\x01\0\x04\0\x1b[method]asset-node.set-name\x01\x11\x01k\x0a\x01@\x01\x04s\
elf\x0c\0\x12\x04\0\x19[method]asset-node.parent\x01\x13\x01p\x0a\x01@\x01\x04se\
lf\x0c\0\x14\x04\0\x1b[method]asset-node.children\x01\x15\x01@\x02\x04self\x0c\x05\
value\x0c\x01\0\x04\0\x1c[method]asset-node.add-child\x01\x16\x04\0\x1f[method]a\
sset-node.remove-child\x01\x16\x01@\x01\x04self\x0c\0\x03\x04\0#[method]asset-no\
de.global-transform\x01\x17\x04\0\x1c[method]asset-node.transform\x01\x17\x01@\x02\
\x04self\x0c\x05value\x03\x01\0\x04\0\x20[method]asset-node.set-transform\x01\x18\
\x01@\0\0\x06\x04\0\x18[constructor]composition\x01\x19\x01h\x05\x01@\x01\x04sel\
f\x1a\0\x14\x04\0\x19[method]composition.nodes\x01\x1b\x01@\x02\x04self\x1a\x05v\
alue\x0c\x01\0\x04\0\x1c[method]composition.add-node\x01\x1c\x04\0\x1f[method]co\
mposition.remove-node\x01\x1c\x03\x01\x17wired:scene/composition\x05\x1c\x02\x03\
\0\x0e\x0bcomposition\x01B\x05\x02\x03\x02\x01\x1d\x04\0\x0bcomposition\x03\0\0\x01\
i\x01\x01@\0\0\x02\x04\0\x04root\x01\x03\x03\x01\x0fwired:scene/api\x05\x1e\x01B\
\x07\x04\0\x06script\x03\x01\x01i\0\x01@\0\0\x01\x04\0\x13[constructor]script\x01\
\x02\x01h\0\x01@\x02\x04self\x03\x05deltav\x01\0\x04\0\x15[method]script.update\x01\
\x04\x04\x01\x12wired:script/types\x05\x1f\x04\x01\x1bexample:unavi-layout/scrip\
t\x04\0\x0b\x0c\x01\0\x06script\x03\0\0\0G\x09producers\x01\x0cprocessed-by\x02\x0d\
wit-component\x070.215.0\x10wit-bindgen-rust\x060.30.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
