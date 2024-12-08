#[allow(dead_code)]
pub mod wired {
    #[allow(dead_code)]
    pub mod dwn {
        #[allow(dead_code, clippy::all)]
        pub mod types {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            #[derive(Clone)]
            pub struct EncryptedData {
                pub alg: _rt::String,
                pub ciphertext: _rt::String,
                pub iv: _rt::String,
                pub recipients: _rt::Vec<_rt::String>,
                pub tag: _rt::String,
            }
            impl ::core::fmt::Debug for EncryptedData {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("EncryptedData")
                        .field("alg", &self.alg)
                        .field("ciphertext", &self.ciphertext)
                        .field("iv", &self.iv)
                        .field("recipients", &self.recipients)
                        .field("tag", &self.tag)
                        .finish()
                }
            }
            #[derive(Clone)]
            pub enum Data {
                Base64(_rt::String),
                Encrypted(EncryptedData),
            }
            impl ::core::fmt::Debug for Data {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    match self {
                        Data::Base64(e) => {
                            f.debug_tuple("Data::Base64").field(e).finish()
                        }
                        Data::Encrypted(e) => {
                            f.debug_tuple("Data::Encrypted").field(e).finish()
                        }
                    }
                }
            }
            #[derive(Clone)]
            pub struct Message {
                pub record_id: _rt::String,
                pub data: Option<Data>,
            }
            impl ::core::fmt::Debug for Message {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("Message")
                        .field("record-id", &self.record_id)
                        .field("data", &self.data)
                        .finish()
                }
            }
            #[derive(Clone)]
            pub struct Status {
                pub code: u16,
                pub detail: Option<_rt::String>,
            }
            impl ::core::fmt::Debug for Status {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("Status")
                        .field("code", &self.code)
                        .field("detail", &self.detail)
                        .finish()
                }
            }
        }
        #[allow(dead_code, clippy::all)]
        pub mod records_query {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            pub type Message = super::super::super::wired::dwn::types::Message;
            pub type Status = super::super::super::wired::dwn::types::Status;
            #[derive(Clone)]
            pub struct RecordsQueryReply {
                pub entries: _rt::Vec<Message>,
                pub status: Status,
            }
            impl ::core::fmt::Debug for RecordsQueryReply {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("RecordsQueryReply")
                        .field("entries", &self.entries)
                        .field("status", &self.status)
                        .finish()
                }
            }
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct RecordsQuery {
                handle: _rt::Resource<RecordsQuery>,
            }
            impl RecordsQuery {
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
            unsafe impl _rt::WasmResource for RecordsQuery {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "wired:dwn/records-query")]
                        extern "C" {
                            #[link_name = "[resource-drop]records-query"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct RecordsQueryBuilder {
                handle: _rt::Resource<RecordsQueryBuilder>,
            }
            impl RecordsQueryBuilder {
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
            unsafe impl _rt::WasmResource for RecordsQueryBuilder {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "wired:dwn/records-query")]
                        extern "C" {
                            #[link_name = "[resource-drop]records-query-builder"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            impl RecordsQuery {
                #[allow(unused_unsafe, clippy::all)]
                pub fn poll(&self) -> Option<RecordsQueryReply> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 28]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 28],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:dwn/records-query")]
                        extern "C" {
                            #[link_name = "[method]records-query.poll"]
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
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let base31 = l2;
                                    let len31 = l3;
                                    let mut result31 = _rt::Vec::with_capacity(len31);
                                    for i in 0..len31 {
                                        let base = base31.add(i * 56);
                                        let e31 = {
                                            let l4 = *base.add(0).cast::<*mut u8>();
                                            let l5 = *base.add(4).cast::<usize>();
                                            let len6 = l5;
                                            let bytes6 = _rt::Vec::from_raw_parts(
                                                l4.cast(),
                                                len6,
                                                len6,
                                            );
                                            let l7 = i32::from(*base.add(8).cast::<u8>());
                                            super::super::super::wired::dwn::types::Message {
                                                record_id: _rt::string_lift(bytes6),
                                                data: match l7 {
                                                    0 => None,
                                                    1 => {
                                                        let e = {
                                                            let l8 = i32::from(*base.add(12).cast::<u8>());
                                                            use super::super::super::wired::dwn::types::Data as V30;
                                                            let v30 = match l8 {
                                                                0 => {
                                                                    let e30 = {
                                                                        let l9 = *base.add(16).cast::<*mut u8>();
                                                                        let l10 = *base.add(20).cast::<usize>();
                                                                        let len11 = l10;
                                                                        let bytes11 = _rt::Vec::from_raw_parts(
                                                                            l9.cast(),
                                                                            len11,
                                                                            len11,
                                                                        );
                                                                        _rt::string_lift(bytes11)
                                                                    };
                                                                    V30::Base64(e30)
                                                                }
                                                                n => {
                                                                    debug_assert_eq!(n, 1, "invalid enum discriminant");
                                                                    let e30 = {
                                                                        let l12 = *base.add(16).cast::<*mut u8>();
                                                                        let l13 = *base.add(20).cast::<usize>();
                                                                        let len14 = l13;
                                                                        let bytes14 = _rt::Vec::from_raw_parts(
                                                                            l12.cast(),
                                                                            len14,
                                                                            len14,
                                                                        );
                                                                        let l15 = *base.add(24).cast::<*mut u8>();
                                                                        let l16 = *base.add(28).cast::<usize>();
                                                                        let len17 = l16;
                                                                        let bytes17 = _rt::Vec::from_raw_parts(
                                                                            l15.cast(),
                                                                            len17,
                                                                            len17,
                                                                        );
                                                                        let l18 = *base.add(32).cast::<*mut u8>();
                                                                        let l19 = *base.add(36).cast::<usize>();
                                                                        let len20 = l19;
                                                                        let bytes20 = _rt::Vec::from_raw_parts(
                                                                            l18.cast(),
                                                                            len20,
                                                                            len20,
                                                                        );
                                                                        let l21 = *base.add(40).cast::<*mut u8>();
                                                                        let l22 = *base.add(44).cast::<usize>();
                                                                        let base26 = l21;
                                                                        let len26 = l22;
                                                                        let mut result26 = _rt::Vec::with_capacity(len26);
                                                                        for i in 0..len26 {
                                                                            let base = base26.add(i * 8);
                                                                            let e26 = {
                                                                                let l23 = *base.add(0).cast::<*mut u8>();
                                                                                let l24 = *base.add(4).cast::<usize>();
                                                                                let len25 = l24;
                                                                                let bytes25 = _rt::Vec::from_raw_parts(
                                                                                    l23.cast(),
                                                                                    len25,
                                                                                    len25,
                                                                                );
                                                                                _rt::string_lift(bytes25)
                                                                            };
                                                                            result26.push(e26);
                                                                        }
                                                                        _rt::cabi_dealloc(base26, len26 * 8, 4);
                                                                        let l27 = *base.add(48).cast::<*mut u8>();
                                                                        let l28 = *base.add(52).cast::<usize>();
                                                                        let len29 = l28;
                                                                        let bytes29 = _rt::Vec::from_raw_parts(
                                                                            l27.cast(),
                                                                            len29,
                                                                            len29,
                                                                        );
                                                                        super::super::super::wired::dwn::types::EncryptedData {
                                                                            alg: _rt::string_lift(bytes14),
                                                                            ciphertext: _rt::string_lift(bytes17),
                                                                            iv: _rt::string_lift(bytes20),
                                                                            recipients: result26,
                                                                            tag: _rt::string_lift(bytes29),
                                                                        }
                                                                    };
                                                                    V30::Encrypted(e30)
                                                                }
                                                            };
                                                            v30
                                                        };
                                                        Some(e)
                                                    }
                                                    _ => _rt::invalid_enum_discriminant(),
                                                },
                                            }
                                        };
                                        result31.push(e31);
                                    }
                                    _rt::cabi_dealloc(base31, len31 * 56, 4);
                                    let l32 = i32::from(*ptr0.add(12).cast::<u16>());
                                    let l33 = i32::from(*ptr0.add(16).cast::<u8>());
                                    RecordsQueryReply {
                                        entries: result31,
                                        status: super::super::super::wired::dwn::types::Status {
                                            code: l32 as u16,
                                            detail: match l33 {
                                                0 => None,
                                                1 => {
                                                    let e = {
                                                        let l34 = *ptr0.add(20).cast::<*mut u8>();
                                                        let l35 = *ptr0.add(24).cast::<usize>();
                                                        let len36 = l35;
                                                        let bytes36 = _rt::Vec::from_raw_parts(
                                                            l34.cast(),
                                                            len36,
                                                            len36,
                                                        );
                                                        _rt::string_lift(bytes36)
                                                    };
                                                    Some(e)
                                                }
                                                _ => _rt::invalid_enum_discriminant(),
                                            },
                                        },
                                    }
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl RecordsQuery {
                #[allow(unused_unsafe, clippy::all)]
                pub fn finished(&self) -> bool {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:dwn/records-query")]
                        extern "C" {
                            #[link_name = "[method]records-query.finished"]
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
            impl RecordsQueryBuilder {
                #[allow(unused_unsafe, clippy::all)]
                pub fn protocol(&self) -> Option<_rt::String> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:dwn/records-query")]
                        extern "C" {
                            #[link_name = "[method]records-query-builder.protocol"]
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
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let len4 = l3;
                                    let bytes4 = _rt::Vec::from_raw_parts(
                                        l2.cast(),
                                        len4,
                                        len4,
                                    );
                                    _rt::string_lift(bytes4)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl RecordsQueryBuilder {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_protocol(&self, value: Option<&str>) {
                    unsafe {
                        let (result1_0, result1_1, result1_2) = match value {
                            Some(e) => {
                                let vec0 = e;
                                let ptr0 = vec0.as_ptr().cast::<u8>();
                                let len0 = vec0.len();
                                (1i32, ptr0.cast_mut(), len0)
                            }
                            None => (0i32, ::core::ptr::null_mut(), 0usize),
                        };
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:dwn/records-query")]
                        extern "C" {
                            #[link_name = "[method]records-query-builder.set-protocol"]
                            fn wit_import(_: i32, _: i32, _: *mut u8, _: usize);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32, _: *mut u8, _: usize) {
                            unreachable!()
                        }
                        wit_import(
                            (self).handle() as i32,
                            result1_0,
                            result1_1,
                            result1_2,
                        );
                    }
                }
            }
            impl RecordsQueryBuilder {
                #[allow(unused_unsafe, clippy::all)]
                pub fn record_id(&self) -> Option<_rt::String> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:dwn/records-query")]
                        extern "C" {
                            #[link_name = "[method]records-query-builder.record-id"]
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
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let len4 = l3;
                                    let bytes4 = _rt::Vec::from_raw_parts(
                                        l2.cast(),
                                        len4,
                                        len4,
                                    );
                                    _rt::string_lift(bytes4)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl RecordsQueryBuilder {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_record_id(&self, value: Option<&str>) {
                    unsafe {
                        let (result1_0, result1_1, result1_2) = match value {
                            Some(e) => {
                                let vec0 = e;
                                let ptr0 = vec0.as_ptr().cast::<u8>();
                                let len0 = vec0.len();
                                (1i32, ptr0.cast_mut(), len0)
                            }
                            None => (0i32, ::core::ptr::null_mut(), 0usize),
                        };
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:dwn/records-query")]
                        extern "C" {
                            #[link_name = "[method]records-query-builder.set-record-id"]
                            fn wit_import(_: i32, _: i32, _: *mut u8, _: usize);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32, _: *mut u8, _: usize) {
                            unreachable!()
                        }
                        wit_import(
                            (self).handle() as i32,
                            result1_0,
                            result1_1,
                            result1_2,
                        );
                    }
                }
            }
            impl RecordsQueryBuilder {
                #[allow(unused_unsafe, clippy::all)]
                pub fn schema(&self) -> Option<_rt::String> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:dwn/records-query")]
                        extern "C" {
                            #[link_name = "[method]records-query-builder.schema"]
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
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let len4 = l3;
                                    let bytes4 = _rt::Vec::from_raw_parts(
                                        l2.cast(),
                                        len4,
                                        len4,
                                    );
                                    _rt::string_lift(bytes4)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl RecordsQueryBuilder {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_schema(&self, value: Option<&str>) {
                    unsafe {
                        let (result1_0, result1_1, result1_2) = match value {
                            Some(e) => {
                                let vec0 = e;
                                let ptr0 = vec0.as_ptr().cast::<u8>();
                                let len0 = vec0.len();
                                (1i32, ptr0.cast_mut(), len0)
                            }
                            None => (0i32, ::core::ptr::null_mut(), 0usize),
                        };
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:dwn/records-query")]
                        extern "C" {
                            #[link_name = "[method]records-query-builder.set-schema"]
                            fn wit_import(_: i32, _: i32, _: *mut u8, _: usize);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32, _: *mut u8, _: usize) {
                            unreachable!()
                        }
                        wit_import(
                            (self).handle() as i32,
                            result1_0,
                            result1_1,
                            result1_2,
                        );
                    }
                }
            }
            impl RecordsQueryBuilder {
                #[allow(unused_unsafe, clippy::all)]
                pub fn run(&self) -> RecordsQuery {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:dwn/records-query")]
                        extern "C" {
                            #[link_name = "[method]records-query-builder.run"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        RecordsQuery::from_handle(ret as u32)
                    }
                }
            }
        }
        #[allow(dead_code, clippy::all)]
        pub mod records_write {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            pub type Status = super::super::super::wired::dwn::types::Status;
            #[derive(Clone)]
            pub struct RecordsWriteReply {
                pub record_id: _rt::String,
                pub status: Status,
            }
            impl ::core::fmt::Debug for RecordsWriteReply {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("RecordsWriteReply")
                        .field("record-id", &self.record_id)
                        .field("status", &self.status)
                        .finish()
                }
            }
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct RecordsWrite {
                handle: _rt::Resource<RecordsWrite>,
            }
            impl RecordsWrite {
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
            unsafe impl _rt::WasmResource for RecordsWrite {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "wired:dwn/records-write")]
                        extern "C" {
                            #[link_name = "[resource-drop]records-write"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct RecordsWriteBuilder {
                handle: _rt::Resource<RecordsWriteBuilder>,
            }
            impl RecordsWriteBuilder {
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
            unsafe impl _rt::WasmResource for RecordsWriteBuilder {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "wired:dwn/records-write")]
                        extern "C" {
                            #[link_name = "[resource-drop]records-write-builder"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            impl RecordsWrite {
                #[allow(unused_unsafe, clippy::all)]
                pub fn poll(&self) -> Option<RecordsWriteReply> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 28]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 28],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:dwn/records-write")]
                        extern "C" {
                            #[link_name = "[method]records-write.poll"]
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
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let len4 = l3;
                                    let bytes4 = _rt::Vec::from_raw_parts(
                                        l2.cast(),
                                        len4,
                                        len4,
                                    );
                                    let l5 = i32::from(*ptr0.add(12).cast::<u16>());
                                    let l6 = i32::from(*ptr0.add(16).cast::<u8>());
                                    RecordsWriteReply {
                                        record_id: _rt::string_lift(bytes4),
                                        status: super::super::super::wired::dwn::types::Status {
                                            code: l5 as u16,
                                            detail: match l6 {
                                                0 => None,
                                                1 => {
                                                    let e = {
                                                        let l7 = *ptr0.add(20).cast::<*mut u8>();
                                                        let l8 = *ptr0.add(24).cast::<usize>();
                                                        let len9 = l8;
                                                        let bytes9 = _rt::Vec::from_raw_parts(
                                                            l7.cast(),
                                                            len9,
                                                            len9,
                                                        );
                                                        _rt::string_lift(bytes9)
                                                    };
                                                    Some(e)
                                                }
                                                _ => _rt::invalid_enum_discriminant(),
                                            },
                                        },
                                    }
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl RecordsWrite {
                #[allow(unused_unsafe, clippy::all)]
                pub fn finished(&self) -> bool {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:dwn/records-write")]
                        extern "C" {
                            #[link_name = "[method]records-write.finished"]
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
            impl RecordsWriteBuilder {
                #[allow(unused_unsafe, clippy::all)]
                pub fn protocol(&self) -> Option<_rt::String> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:dwn/records-write")]
                        extern "C" {
                            #[link_name = "[method]records-write-builder.protocol"]
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
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let len4 = l3;
                                    let bytes4 = _rt::Vec::from_raw_parts(
                                        l2.cast(),
                                        len4,
                                        len4,
                                    );
                                    _rt::string_lift(bytes4)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl RecordsWriteBuilder {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_protocol(&self, value: Option<&str>) {
                    unsafe {
                        let (result1_0, result1_1, result1_2) = match value {
                            Some(e) => {
                                let vec0 = e;
                                let ptr0 = vec0.as_ptr().cast::<u8>();
                                let len0 = vec0.len();
                                (1i32, ptr0.cast_mut(), len0)
                            }
                            None => (0i32, ::core::ptr::null_mut(), 0usize),
                        };
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:dwn/records-write")]
                        extern "C" {
                            #[link_name = "[method]records-write-builder.set-protocol"]
                            fn wit_import(_: i32, _: i32, _: *mut u8, _: usize);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32, _: *mut u8, _: usize) {
                            unreachable!()
                        }
                        wit_import(
                            (self).handle() as i32,
                            result1_0,
                            result1_1,
                            result1_2,
                        );
                    }
                }
            }
            impl RecordsWriteBuilder {
                #[allow(unused_unsafe, clippy::all)]
                pub fn record_id(&self) -> Option<_rt::String> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:dwn/records-write")]
                        extern "C" {
                            #[link_name = "[method]records-write-builder.record-id"]
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
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let len4 = l3;
                                    let bytes4 = _rt::Vec::from_raw_parts(
                                        l2.cast(),
                                        len4,
                                        len4,
                                    );
                                    _rt::string_lift(bytes4)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl RecordsWriteBuilder {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_record_id(&self, value: Option<&str>) {
                    unsafe {
                        let (result1_0, result1_1, result1_2) = match value {
                            Some(e) => {
                                let vec0 = e;
                                let ptr0 = vec0.as_ptr().cast::<u8>();
                                let len0 = vec0.len();
                                (1i32, ptr0.cast_mut(), len0)
                            }
                            None => (0i32, ::core::ptr::null_mut(), 0usize),
                        };
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:dwn/records-write")]
                        extern "C" {
                            #[link_name = "[method]records-write-builder.set-record-id"]
                            fn wit_import(_: i32, _: i32, _: *mut u8, _: usize);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32, _: *mut u8, _: usize) {
                            unreachable!()
                        }
                        wit_import(
                            (self).handle() as i32,
                            result1_0,
                            result1_1,
                            result1_2,
                        );
                    }
                }
            }
            impl RecordsWriteBuilder {
                #[allow(unused_unsafe, clippy::all)]
                pub fn schema(&self) -> Option<_rt::String> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:dwn/records-write")]
                        extern "C" {
                            #[link_name = "[method]records-write-builder.schema"]
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
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let len4 = l3;
                                    let bytes4 = _rt::Vec::from_raw_parts(
                                        l2.cast(),
                                        len4,
                                        len4,
                                    );
                                    _rt::string_lift(bytes4)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl RecordsWriteBuilder {
                #[allow(unused_unsafe, clippy::all)]
                pub fn set_schema(&self, value: Option<&str>) {
                    unsafe {
                        let (result1_0, result1_1, result1_2) = match value {
                            Some(e) => {
                                let vec0 = e;
                                let ptr0 = vec0.as_ptr().cast::<u8>();
                                let len0 = vec0.len();
                                (1i32, ptr0.cast_mut(), len0)
                            }
                            None => (0i32, ::core::ptr::null_mut(), 0usize),
                        };
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:dwn/records-write")]
                        extern "C" {
                            #[link_name = "[method]records-write-builder.set-schema"]
                            fn wit_import(_: i32, _: i32, _: *mut u8, _: usize);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32, _: *mut u8, _: usize) {
                            unreachable!()
                        }
                        wit_import(
                            (self).handle() as i32,
                            result1_0,
                            result1_1,
                            result1_2,
                        );
                    }
                }
            }
            impl RecordsWriteBuilder {
                #[allow(unused_unsafe, clippy::all)]
                pub fn run(&self) -> RecordsWrite {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:dwn/records-write")]
                        extern "C" {
                            #[link_name = "[method]records-write-builder.run"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        RecordsWrite::from_handle(ret as u32)
                    }
                }
            }
        }
        #[allow(dead_code, clippy::all)]
        pub mod dwn {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            pub type RecordsQueryBuilder = super::super::super::wired::dwn::records_query::RecordsQueryBuilder;
            pub type RecordsWriteBuilder = super::super::super::wired::dwn::records_write::RecordsWriteBuilder;
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct Dwn {
                handle: _rt::Resource<Dwn>,
            }
            impl Dwn {
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
            unsafe impl _rt::WasmResource for Dwn {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "wired:dwn/dwn")]
                        extern "C" {
                            #[link_name = "[resource-drop]dwn"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            impl Dwn {
                #[allow(unused_unsafe, clippy::all)]
                pub fn records_query(&self) -> RecordsQueryBuilder {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:dwn/dwn")]
                        extern "C" {
                            #[link_name = "[method]dwn.records-query"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::dwn::records_query::RecordsQueryBuilder::from_handle(
                            ret as u32,
                        )
                    }
                }
            }
            impl Dwn {
                #[allow(unused_unsafe, clippy::all)]
                pub fn records_write(&self) -> RecordsWriteBuilder {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "wired:dwn/dwn")]
                        extern "C" {
                            #[link_name = "[method]dwn.records-write"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        super::super::super::wired::dwn::records_write::RecordsWriteBuilder::from_handle(
                            ret as u32,
                        )
                    }
                }
            }
        }
        #[allow(dead_code, clippy::all)]
        pub mod api {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            pub type Dwn = super::super::super::wired::dwn::dwn::Dwn;
            #[allow(unused_unsafe, clippy::all)]
            /// Get the local user's DWN.
            pub fn user_dwn() -> Dwn {
                unsafe {
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "wired:dwn/api")]
                    extern "C" {
                        #[link_name = "user-dwn"]
                        fn wit_import() -> i32;
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import() -> i32 {
                        unreachable!()
                    }
                    let ret = wit_import();
                    super::super::super::wired::dwn::dwn::Dwn::from_handle(ret as u32)
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// Get the local user's default world host DWN.
            pub fn world_host_dwn() -> Dwn {
                unsafe {
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "wired:dwn/api")]
                    extern "C" {
                        #[link_name = "world-host-dwn"]
                        fn wit_import() -> i32;
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import() -> i32 {
                        unreachable!()
                    }
                    let ret = wit_import();
                    super::super::super::wired::dwn::dwn::Dwn::from_handle(ret as u32)
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
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            #[repr(u8)]
            #[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
            pub enum LogLevel {
                Debug,
                Info,
                Warn,
                Error,
            }
            impl ::core::fmt::Debug for LogLevel {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
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
                static __FORCE_SECTION_REF: fn() = super::super::super::super::__link_custom_section_describing_imports;
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
                        let ptr: *mut _ScriptRep<T> = _rt::Box::into_raw(
                            _rt::Box::new(val),
                        );
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
                            assert!(! cfg!(target_feature = "atomics"));
                            let id = TypeId::of::<T>();
                            match LAST_TYPE {
                                Some(ty) => {
                                    assert!(
                                        ty == id, "cannot use two types with this resource type"
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
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let result0 = Script::new(T::new());
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_script_update_cabi<T: GuestScript>(
                    arg0: *mut u8,
                    arg1: f32,
                ) {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
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
    pub use alloc_crate::string::String;
    pub use alloc_crate::vec::Vec;
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
            f.debug_struct("Resource").field("handle", &self.handle).finish()
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
    pub unsafe fn string_lift(bytes: Vec<u8>) -> String {
        if cfg!(debug_assertions) {
            String::from_utf8(bytes).unwrap()
        } else {
            String::from_utf8_unchecked(bytes)
        }
    }
    pub unsafe fn cabi_dealloc(ptr: *mut u8, size: usize, align: usize) {
        if size == 0 {
            return;
        }
        let layout = alloc::Layout::from_size_align_unchecked(size, align);
        alloc::dealloc(ptr, layout);
    }
    pub unsafe fn invalid_enum_discriminant<T>() -> T {
        if cfg!(debug_assertions) {
            panic!("invalid enum discriminant")
        } else {
            core::hint::unreachable_unchecked()
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
#[link_section = "component-type:wit-bindgen:0.35.0:test:wired-dwn:script:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 2109] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\xc0\x0f\x01A\x02\x01\
A\x13\x01B\x0b\x01ps\x01r\x05\x03algs\x0aciphertexts\x02ivs\x0arecipients\0\x03t\
ags\x04\0\x0eencrypted-data\x03\0\x01\x01q\x02\x06base64\x01s\0\x09encrypted\x01\
\x02\0\x04\0\x04data\x03\0\x03\x01k\x04\x01r\x02\x09record-ids\x04data\x05\x04\0\
\x07message\x03\0\x06\x01ks\x01r\x02\x04code{\x06detail\x08\x04\0\x06status\x03\0\
\x09\x03\0\x0fwired:dwn/types\x05\0\x02\x03\0\0\x07message\x02\x03\0\0\x06status\
\x01B\x1c\x02\x03\x02\x01\x01\x04\0\x07message\x03\0\0\x02\x03\x02\x01\x02\x04\0\
\x06status\x03\0\x02\x01p\x01\x01r\x02\x07entries\x04\x06status\x03\x04\0\x13rec\
ords-query-reply\x03\0\x05\x04\0\x0drecords-query\x03\x01\x04\0\x15records-query\
-builder\x03\x01\x01h\x07\x01k\x06\x01@\x01\x04self\x09\0\x0a\x04\0\x1a[method]r\
ecords-query.poll\x01\x0b\x01@\x01\x04self\x09\0\x7f\x04\0\x1e[method]records-qu\
ery.finished\x01\x0c\x01h\x08\x01ks\x01@\x01\x04self\x0d\0\x0e\x04\0&[method]rec\
ords-query-builder.protocol\x01\x0f\x01@\x02\x04self\x0d\x05value\x0e\x01\0\x04\0\
*[method]records-query-builder.set-protocol\x01\x10\x04\0'[method]records-query-\
builder.record-id\x01\x0f\x04\0+[method]records-query-builder.set-record-id\x01\x10\
\x04\0$[method]records-query-builder.schema\x01\x0f\x04\0([method]records-query-\
builder.set-schema\x01\x10\x01i\x07\x01@\x01\x04self\x0d\0\x11\x04\0![method]rec\
ords-query-builder.run\x01\x12\x03\0\x17wired:dwn/records-query\x05\x03\x01B\x1b\
\x02\x03\x02\x01\x01\x04\0\x07message\x03\0\0\x02\x03\x02\x01\x02\x04\0\x06statu\
s\x03\0\x02\x01r\x02\x09record-ids\x06status\x03\x04\0\x13records-write-reply\x03\
\0\x04\x04\0\x0drecords-write\x03\x01\x04\0\x15records-write-builder\x03\x01\x01\
h\x06\x01k\x05\x01@\x01\x04self\x08\0\x09\x04\0\x1a[method]records-write.poll\x01\
\x0a\x01@\x01\x04self\x08\0\x7f\x04\0\x1e[method]records-write.finished\x01\x0b\x01\
h\x07\x01ks\x01@\x01\x04self\x0c\0\x0d\x04\0&[method]records-write-builder.proto\
col\x01\x0e\x01@\x02\x04self\x0c\x05value\x0d\x01\0\x04\0*[method]records-write-\
builder.set-protocol\x01\x0f\x04\0'[method]records-write-builder.record-id\x01\x0e\
\x04\0+[method]records-write-builder.set-record-id\x01\x0f\x04\0$[method]records\
-write-builder.schema\x01\x0e\x04\0([method]records-write-builder.set-schema\x01\
\x0f\x01i\x06\x01@\x01\x04self\x0c\0\x10\x04\0![method]records-write-builder.run\
\x01\x11\x03\0\x17wired:dwn/records-write\x05\x04\x02\x03\0\x01\x15records-query\
-builder\x02\x03\0\x02\x15records-write-builder\x01B\x0c\x02\x03\x02\x01\x05\x04\
\0\x15records-query-builder\x03\0\0\x02\x03\x02\x01\x06\x04\0\x15records-write-b\
uilder\x03\0\x02\x04\0\x03dwn\x03\x01\x01h\x04\x01i\x01\x01@\x01\x04self\x05\0\x06\
\x04\0\x19[method]dwn.records-query\x01\x07\x01i\x03\x01@\x01\x04self\x05\0\x08\x04\
\0\x19[method]dwn.records-write\x01\x09\x03\0\x0dwired:dwn/dwn\x05\x07\x02\x03\0\
\x03\x03dwn\x01B\x06\x02\x03\x02\x01\x08\x04\0\x03dwn\x03\0\0\x01i\x01\x01@\0\0\x02\
\x04\0\x08user-dwn\x01\x03\x04\0\x0eworld-host-dwn\x01\x03\x03\0\x0dwired:dwn/ap\
i\x05\x09\x01B\x04\x01m\x04\x05debug\x04info\x04warn\x05error\x04\0\x09log-level\
\x03\0\0\x01@\x02\x05level\x01\x07messages\x01\0\x04\0\x03log\x01\x02\x03\0\x0dw\
ired:log/api\x05\x0a\x01B\x07\x04\0\x06script\x03\x01\x01i\0\x01@\0\0\x01\x04\0\x13\
[constructor]script\x01\x02\x01h\0\x01@\x02\x04self\x03\x05deltav\x01\0\x04\0\x15\
[method]script.update\x01\x04\x04\0\x12wired:script/types\x05\x0b\x04\0\x15test:\
wired-dwn/script\x04\0\x0b\x0c\x01\0\x06script\x03\0\0\0G\x09producers\x01\x0cpr\
ocessed-by\x02\x0dwit-component\x070.220.0\x10wit-bindgen-rust\x060.35.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
