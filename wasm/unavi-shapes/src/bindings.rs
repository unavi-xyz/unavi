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
    }
}
#[allow(dead_code)]
pub mod exports {
    #[allow(dead_code)]
    pub mod unavi {
        #[allow(dead_code)]
        pub mod shapes {
            #[allow(dead_code, clippy::all)]
            pub mod api {
                #[used]
                #[doc(hidden)]
                static __FORCE_SECTION_REF: fn() =
                    super::super::super::super::__link_custom_section_describing_imports;
                use super::super::super::super::_rt;
                pub type Vec2 = super::super::super::super::wired::math::types::Vec2;
                pub type Vec3 = super::super::super::super::wired::math::types::Vec3;
                pub type Mesh = super::super::super::super::wired::scene::mesh::Mesh;
                pub type Node = super::super::super::super::wired::scene::node::Node;
                #[derive(Debug)]
                #[repr(transparent)]
                pub struct Rectangle {
                    handle: _rt::Resource<Rectangle>,
                }
                type _RectangleRep<T> = Option<T>;
                impl Rectangle {
                    /// Creates a new resource from the specified representation.
                    ///
                    /// This function will create a new resource handle by moving `val` onto
                    /// the heap and then passing that heap pointer to the component model to
                    /// create a handle. The owned handle is then returned as `Rectangle`.
                    pub fn new<T: GuestRectangle>(val: T) -> Self {
                        Self::type_guard::<T>();
                        let val: _RectangleRep<T> = Some(val);
                        let ptr: *mut _RectangleRep<T> = _rt::Box::into_raw(_rt::Box::new(val));
                        unsafe { Self::from_handle(T::_resource_new(ptr.cast())) }
                    }
                    /// Gets access to the underlying `T` which represents this resource.
                    pub fn get<T: GuestRectangle>(&self) -> &T {
                        let ptr = unsafe { &*self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    /// Gets mutable access to the underlying `T` which represents this
                    /// resource.
                    pub fn get_mut<T: GuestRectangle>(&mut self) -> &mut T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_mut().unwrap()
                    }
                    /// Consumes this resource and returns the underlying `T`.
                    pub fn into_inner<T: GuestRectangle>(self) -> T {
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
                        let _ = _rt::Box::from_raw(handle as *mut _RectangleRep<T>);
                    }
                    fn as_ptr<T: GuestRectangle>(&self) -> *mut _RectangleRep<T> {
                        Rectangle::type_guard::<T>();
                        T::_resource_rep(self.handle()).cast()
                    }
                }
                /// A borrowed version of [`Rectangle`] which represents a borrowed value
                /// with the lifetime `'a`.
                #[derive(Debug)]
                #[repr(transparent)]
                pub struct RectangleBorrow<'a> {
                    rep: *mut u8,
                    _marker: core::marker::PhantomData<&'a Rectangle>,
                }
                impl<'a> RectangleBorrow<'a> {
                    #[doc(hidden)]
                    pub unsafe fn lift(rep: usize) -> Self {
                        Self {
                            rep: rep as *mut u8,
                            _marker: core::marker::PhantomData,
                        }
                    }
                    /// Gets access to the underlying `T` in this resource.
                    pub fn get<T: GuestRectangle>(&self) -> &T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    fn as_ptr<T: 'static>(&self) -> *mut _RectangleRep<T> {
                        Rectangle::type_guard::<T>();
                        self.rep.cast()
                    }
                }
                unsafe impl _rt::WasmResource for Rectangle {
                    #[inline]
                    unsafe fn drop(_handle: u32) {
                        #[cfg(not(target_arch = "wasm32"))]
                        unreachable!();
                        #[cfg(target_arch = "wasm32")]
                        {
                            #[link(wasm_import_module = "[export]unavi:shapes/api")]
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
                type _CircleRep<T> = Option<T>;
                impl Circle {
                    /// Creates a new resource from the specified representation.
                    ///
                    /// This function will create a new resource handle by moving `val` onto
                    /// the heap and then passing that heap pointer to the component model to
                    /// create a handle. The owned handle is then returned as `Circle`.
                    pub fn new<T: GuestCircle>(val: T) -> Self {
                        Self::type_guard::<T>();
                        let val: _CircleRep<T> = Some(val);
                        let ptr: *mut _CircleRep<T> = _rt::Box::into_raw(_rt::Box::new(val));
                        unsafe { Self::from_handle(T::_resource_new(ptr.cast())) }
                    }
                    /// Gets access to the underlying `T` which represents this resource.
                    pub fn get<T: GuestCircle>(&self) -> &T {
                        let ptr = unsafe { &*self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    /// Gets mutable access to the underlying `T` which represents this
                    /// resource.
                    pub fn get_mut<T: GuestCircle>(&mut self) -> &mut T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_mut().unwrap()
                    }
                    /// Consumes this resource and returns the underlying `T`.
                    pub fn into_inner<T: GuestCircle>(self) -> T {
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
                        let _ = _rt::Box::from_raw(handle as *mut _CircleRep<T>);
                    }
                    fn as_ptr<T: GuestCircle>(&self) -> *mut _CircleRep<T> {
                        Circle::type_guard::<T>();
                        T::_resource_rep(self.handle()).cast()
                    }
                }
                /// A borrowed version of [`Circle`] which represents a borrowed value
                /// with the lifetime `'a`.
                #[derive(Debug)]
                #[repr(transparent)]
                pub struct CircleBorrow<'a> {
                    rep: *mut u8,
                    _marker: core::marker::PhantomData<&'a Circle>,
                }
                impl<'a> CircleBorrow<'a> {
                    #[doc(hidden)]
                    pub unsafe fn lift(rep: usize) -> Self {
                        Self {
                            rep: rep as *mut u8,
                            _marker: core::marker::PhantomData,
                        }
                    }
                    /// Gets access to the underlying `T` in this resource.
                    pub fn get<T: GuestCircle>(&self) -> &T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    fn as_ptr<T: 'static>(&self) -> *mut _CircleRep<T> {
                        Circle::type_guard::<T>();
                        self.rep.cast()
                    }
                }
                unsafe impl _rt::WasmResource for Circle {
                    #[inline]
                    unsafe fn drop(_handle: u32) {
                        #[cfg(not(target_arch = "wasm32"))]
                        unreachable!();
                        #[cfg(target_arch = "wasm32")]
                        {
                            #[link(wasm_import_module = "[export]unavi:shapes/api")]
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
                type _EllipseRep<T> = Option<T>;
                impl Ellipse {
                    /// Creates a new resource from the specified representation.
                    ///
                    /// This function will create a new resource handle by moving `val` onto
                    /// the heap and then passing that heap pointer to the component model to
                    /// create a handle. The owned handle is then returned as `Ellipse`.
                    pub fn new<T: GuestEllipse>(val: T) -> Self {
                        Self::type_guard::<T>();
                        let val: _EllipseRep<T> = Some(val);
                        let ptr: *mut _EllipseRep<T> = _rt::Box::into_raw(_rt::Box::new(val));
                        unsafe { Self::from_handle(T::_resource_new(ptr.cast())) }
                    }
                    /// Gets access to the underlying `T` which represents this resource.
                    pub fn get<T: GuestEllipse>(&self) -> &T {
                        let ptr = unsafe { &*self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    /// Gets mutable access to the underlying `T` which represents this
                    /// resource.
                    pub fn get_mut<T: GuestEllipse>(&mut self) -> &mut T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_mut().unwrap()
                    }
                    /// Consumes this resource and returns the underlying `T`.
                    pub fn into_inner<T: GuestEllipse>(self) -> T {
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
                        let _ = _rt::Box::from_raw(handle as *mut _EllipseRep<T>);
                    }
                    fn as_ptr<T: GuestEllipse>(&self) -> *mut _EllipseRep<T> {
                        Ellipse::type_guard::<T>();
                        T::_resource_rep(self.handle()).cast()
                    }
                }
                /// A borrowed version of [`Ellipse`] which represents a borrowed value
                /// with the lifetime `'a`.
                #[derive(Debug)]
                #[repr(transparent)]
                pub struct EllipseBorrow<'a> {
                    rep: *mut u8,
                    _marker: core::marker::PhantomData<&'a Ellipse>,
                }
                impl<'a> EllipseBorrow<'a> {
                    #[doc(hidden)]
                    pub unsafe fn lift(rep: usize) -> Self {
                        Self {
                            rep: rep as *mut u8,
                            _marker: core::marker::PhantomData,
                        }
                    }
                    /// Gets access to the underlying `T` in this resource.
                    pub fn get<T: GuestEllipse>(&self) -> &T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    fn as_ptr<T: 'static>(&self) -> *mut _EllipseRep<T> {
                        Ellipse::type_guard::<T>();
                        self.rep.cast()
                    }
                }
                unsafe impl _rt::WasmResource for Ellipse {
                    #[inline]
                    unsafe fn drop(_handle: u32) {
                        #[cfg(not(target_arch = "wasm32"))]
                        unreachable!();
                        #[cfg(target_arch = "wasm32")]
                        {
                            #[link(wasm_import_module = "[export]unavi:shapes/api")]
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
                type _CylinderRep<T> = Option<T>;
                impl Cylinder {
                    /// Creates a new resource from the specified representation.
                    ///
                    /// This function will create a new resource handle by moving `val` onto
                    /// the heap and then passing that heap pointer to the component model to
                    /// create a handle. The owned handle is then returned as `Cylinder`.
                    pub fn new<T: GuestCylinder>(val: T) -> Self {
                        Self::type_guard::<T>();
                        let val: _CylinderRep<T> = Some(val);
                        let ptr: *mut _CylinderRep<T> = _rt::Box::into_raw(_rt::Box::new(val));
                        unsafe { Self::from_handle(T::_resource_new(ptr.cast())) }
                    }
                    /// Gets access to the underlying `T` which represents this resource.
                    pub fn get<T: GuestCylinder>(&self) -> &T {
                        let ptr = unsafe { &*self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    /// Gets mutable access to the underlying `T` which represents this
                    /// resource.
                    pub fn get_mut<T: GuestCylinder>(&mut self) -> &mut T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_mut().unwrap()
                    }
                    /// Consumes this resource and returns the underlying `T`.
                    pub fn into_inner<T: GuestCylinder>(self) -> T {
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
                        let _ = _rt::Box::from_raw(handle as *mut _CylinderRep<T>);
                    }
                    fn as_ptr<T: GuestCylinder>(&self) -> *mut _CylinderRep<T> {
                        Cylinder::type_guard::<T>();
                        T::_resource_rep(self.handle()).cast()
                    }
                }
                /// A borrowed version of [`Cylinder`] which represents a borrowed value
                /// with the lifetime `'a`.
                #[derive(Debug)]
                #[repr(transparent)]
                pub struct CylinderBorrow<'a> {
                    rep: *mut u8,
                    _marker: core::marker::PhantomData<&'a Cylinder>,
                }
                impl<'a> CylinderBorrow<'a> {
                    #[doc(hidden)]
                    pub unsafe fn lift(rep: usize) -> Self {
                        Self {
                            rep: rep as *mut u8,
                            _marker: core::marker::PhantomData,
                        }
                    }
                    /// Gets access to the underlying `T` in this resource.
                    pub fn get<T: GuestCylinder>(&self) -> &T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    fn as_ptr<T: 'static>(&self) -> *mut _CylinderRep<T> {
                        Cylinder::type_guard::<T>();
                        self.rep.cast()
                    }
                }
                unsafe impl _rt::WasmResource for Cylinder {
                    #[inline]
                    unsafe fn drop(_handle: u32) {
                        #[cfg(not(target_arch = "wasm32"))]
                        unreachable!();
                        #[cfg(target_arch = "wasm32")]
                        {
                            #[link(wasm_import_module = "[export]unavi:shapes/api")]
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
                type _CuboidRep<T> = Option<T>;
                impl Cuboid {
                    /// Creates a new resource from the specified representation.
                    ///
                    /// This function will create a new resource handle by moving `val` onto
                    /// the heap and then passing that heap pointer to the component model to
                    /// create a handle. The owned handle is then returned as `Cuboid`.
                    pub fn new<T: GuestCuboid>(val: T) -> Self {
                        Self::type_guard::<T>();
                        let val: _CuboidRep<T> = Some(val);
                        let ptr: *mut _CuboidRep<T> = _rt::Box::into_raw(_rt::Box::new(val));
                        unsafe { Self::from_handle(T::_resource_new(ptr.cast())) }
                    }
                    /// Gets access to the underlying `T` which represents this resource.
                    pub fn get<T: GuestCuboid>(&self) -> &T {
                        let ptr = unsafe { &*self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    /// Gets mutable access to the underlying `T` which represents this
                    /// resource.
                    pub fn get_mut<T: GuestCuboid>(&mut self) -> &mut T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_mut().unwrap()
                    }
                    /// Consumes this resource and returns the underlying `T`.
                    pub fn into_inner<T: GuestCuboid>(self) -> T {
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
                        let _ = _rt::Box::from_raw(handle as *mut _CuboidRep<T>);
                    }
                    fn as_ptr<T: GuestCuboid>(&self) -> *mut _CuboidRep<T> {
                        Cuboid::type_guard::<T>();
                        T::_resource_rep(self.handle()).cast()
                    }
                }
                /// A borrowed version of [`Cuboid`] which represents a borrowed value
                /// with the lifetime `'a`.
                #[derive(Debug)]
                #[repr(transparent)]
                pub struct CuboidBorrow<'a> {
                    rep: *mut u8,
                    _marker: core::marker::PhantomData<&'a Cuboid>,
                }
                impl<'a> CuboidBorrow<'a> {
                    #[doc(hidden)]
                    pub unsafe fn lift(rep: usize) -> Self {
                        Self {
                            rep: rep as *mut u8,
                            _marker: core::marker::PhantomData,
                        }
                    }
                    /// Gets access to the underlying `T` in this resource.
                    pub fn get<T: GuestCuboid>(&self) -> &T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    fn as_ptr<T: 'static>(&self) -> *mut _CuboidRep<T> {
                        Cuboid::type_guard::<T>();
                        self.rep.cast()
                    }
                }
                unsafe impl _rt::WasmResource for Cuboid {
                    #[inline]
                    unsafe fn drop(_handle: u32) {
                        #[cfg(not(target_arch = "wasm32"))]
                        unreachable!();
                        #[cfg(target_arch = "wasm32")]
                        {
                            #[link(wasm_import_module = "[export]unavi:shapes/api")]
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
                            SphereKind::Ico(e) => {
                                f.debug_tuple("SphereKind::Ico").field(e).finish()
                            }
                            SphereKind::Uv(e) => f.debug_tuple("SphereKind::Uv").field(e).finish(),
                        }
                    }
                }
                #[derive(Debug)]
                #[repr(transparent)]
                pub struct Sphere {
                    handle: _rt::Resource<Sphere>,
                }
                type _SphereRep<T> = Option<T>;
                impl Sphere {
                    /// Creates a new resource from the specified representation.
                    ///
                    /// This function will create a new resource handle by moving `val` onto
                    /// the heap and then passing that heap pointer to the component model to
                    /// create a handle. The owned handle is then returned as `Sphere`.
                    pub fn new<T: GuestSphere>(val: T) -> Self {
                        Self::type_guard::<T>();
                        let val: _SphereRep<T> = Some(val);
                        let ptr: *mut _SphereRep<T> = _rt::Box::into_raw(_rt::Box::new(val));
                        unsafe { Self::from_handle(T::_resource_new(ptr.cast())) }
                    }
                    /// Gets access to the underlying `T` which represents this resource.
                    pub fn get<T: GuestSphere>(&self) -> &T {
                        let ptr = unsafe { &*self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    /// Gets mutable access to the underlying `T` which represents this
                    /// resource.
                    pub fn get_mut<T: GuestSphere>(&mut self) -> &mut T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_mut().unwrap()
                    }
                    /// Consumes this resource and returns the underlying `T`.
                    pub fn into_inner<T: GuestSphere>(self) -> T {
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
                        let _ = _rt::Box::from_raw(handle as *mut _SphereRep<T>);
                    }
                    fn as_ptr<T: GuestSphere>(&self) -> *mut _SphereRep<T> {
                        Sphere::type_guard::<T>();
                        T::_resource_rep(self.handle()).cast()
                    }
                }
                /// A borrowed version of [`Sphere`] which represents a borrowed value
                /// with the lifetime `'a`.
                #[derive(Debug)]
                #[repr(transparent)]
                pub struct SphereBorrow<'a> {
                    rep: *mut u8,
                    _marker: core::marker::PhantomData<&'a Sphere>,
                }
                impl<'a> SphereBorrow<'a> {
                    #[doc(hidden)]
                    pub unsafe fn lift(rep: usize) -> Self {
                        Self {
                            rep: rep as *mut u8,
                            _marker: core::marker::PhantomData,
                        }
                    }
                    /// Gets access to the underlying `T` in this resource.
                    pub fn get<T: GuestSphere>(&self) -> &T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    fn as_ptr<T: 'static>(&self) -> *mut _SphereRep<T> {
                        Sphere::type_guard::<T>();
                        self.rep.cast()
                    }
                }
                unsafe impl _rt::WasmResource for Sphere {
                    #[inline]
                    unsafe fn drop(_handle: u32) {
                        #[cfg(not(target_arch = "wasm32"))]
                        unreachable!();
                        #[cfg(target_arch = "wasm32")]
                        {
                            #[link(wasm_import_module = "[export]unavi:shapes/api")]
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
                type _AxesRep<T> = Option<T>;
                impl Axes {
                    /// Creates a new resource from the specified representation.
                    ///
                    /// This function will create a new resource handle by moving `val` onto
                    /// the heap and then passing that heap pointer to the component model to
                    /// create a handle. The owned handle is then returned as `Axes`.
                    pub fn new<T: GuestAxes>(val: T) -> Self {
                        Self::type_guard::<T>();
                        let val: _AxesRep<T> = Some(val);
                        let ptr: *mut _AxesRep<T> = _rt::Box::into_raw(_rt::Box::new(val));
                        unsafe { Self::from_handle(T::_resource_new(ptr.cast())) }
                    }
                    /// Gets access to the underlying `T` which represents this resource.
                    pub fn get<T: GuestAxes>(&self) -> &T {
                        let ptr = unsafe { &*self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    /// Gets mutable access to the underlying `T` which represents this
                    /// resource.
                    pub fn get_mut<T: GuestAxes>(&mut self) -> &mut T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_mut().unwrap()
                    }
                    /// Consumes this resource and returns the underlying `T`.
                    pub fn into_inner<T: GuestAxes>(self) -> T {
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
                        let _ = _rt::Box::from_raw(handle as *mut _AxesRep<T>);
                    }
                    fn as_ptr<T: GuestAxes>(&self) -> *mut _AxesRep<T> {
                        Axes::type_guard::<T>();
                        T::_resource_rep(self.handle()).cast()
                    }
                }
                /// A borrowed version of [`Axes`] which represents a borrowed value
                /// with the lifetime `'a`.
                #[derive(Debug)]
                #[repr(transparent)]
                pub struct AxesBorrow<'a> {
                    rep: *mut u8,
                    _marker: core::marker::PhantomData<&'a Axes>,
                }
                impl<'a> AxesBorrow<'a> {
                    #[doc(hidden)]
                    pub unsafe fn lift(rep: usize) -> Self {
                        Self {
                            rep: rep as *mut u8,
                            _marker: core::marker::PhantomData,
                        }
                    }
                    /// Gets access to the underlying `T` in this resource.
                    pub fn get<T: GuestAxes>(&self) -> &T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    fn as_ptr<T: 'static>(&self) -> *mut _AxesRep<T> {
                        Axes::type_guard::<T>();
                        self.rep.cast()
                    }
                }
                unsafe impl _rt::WasmResource for Axes {
                    #[inline]
                    unsafe fn drop(_handle: u32) {
                        #[cfg(not(target_arch = "wasm32"))]
                        unreachable!();
                        #[cfg(target_arch = "wasm32")]
                        {
                            #[link(wasm_import_module = "[export]unavi:shapes/api")]
                            extern "C" {
                                #[link_name = "[resource-drop]axes"]
                                fn drop(_: u32);
                            }
                            drop(_handle);
                        }
                    }
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_constructor_rectangle_cabi<T: GuestRectangle>(
                    arg0: f32,
                    arg1: f32,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = Rectangle::new(T::new(
                        super::super::super::super::wired::math::types::Vec2 { x: arg0, y: arg1 },
                    ));
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_rectangle_size_cabi<T: GuestRectangle>(
                    arg0: *mut u8,
                ) -> *mut u8 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::size(RectangleBorrow::lift(arg0 as u32 as usize).get());
                    let ptr1 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    let super::super::super::super::wired::math::types::Vec2 { x: x2, y: y2 } =
                        result0;
                    *ptr1.add(0).cast::<f32>() = _rt::as_f32(x2);
                    *ptr1.add(4).cast::<f32>() = _rt::as_f32(y2);
                    ptr1
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_rectangle_set_size_cabi<T: GuestRectangle>(
                    arg0: *mut u8,
                    arg1: f32,
                    arg2: f32,
                ) {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    T::set_size(
                        RectangleBorrow::lift(arg0 as u32 as usize).get(),
                        super::super::super::super::wired::math::types::Vec2 { x: arg1, y: arg2 },
                    );
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_rectangle_to_mesh_cabi<T: GuestRectangle>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::to_mesh(RectangleBorrow::lift(arg0 as u32 as usize).get());
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_rectangle_to_node_cabi<T: GuestRectangle>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::to_node(RectangleBorrow::lift(arg0 as u32 as usize).get());
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_rectangle_to_physics_node_cabi<T: GuestRectangle>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 =
                        T::to_physics_node(RectangleBorrow::lift(arg0 as u32 as usize).get());
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_constructor_circle_cabi<T: GuestCircle>(arg0: f32) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = Circle::new(T::new(arg0));
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_circle_radius_cabi<T: GuestCircle>(
                    arg0: *mut u8,
                ) -> f32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::radius(CircleBorrow::lift(arg0 as u32 as usize).get());
                    _rt::as_f32(result0)
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_circle_set_radius_cabi<T: GuestCircle>(
                    arg0: *mut u8,
                    arg1: f32,
                ) {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    T::set_radius(CircleBorrow::lift(arg0 as u32 as usize).get(), arg1);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_circle_resolution_cabi<T: GuestCircle>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::resolution(CircleBorrow::lift(arg0 as u32 as usize).get());
                    _rt::as_i32(result0)
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_circle_set_resolution_cabi<T: GuestCircle>(
                    arg0: *mut u8,
                    arg1: i32,
                ) {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    T::set_resolution(CircleBorrow::lift(arg0 as u32 as usize).get(), arg1 as u16);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_circle_to_mesh_cabi<T: GuestCircle>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::to_mesh(CircleBorrow::lift(arg0 as u32 as usize).get());
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_circle_to_node_cabi<T: GuestCircle>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::to_node(CircleBorrow::lift(arg0 as u32 as usize).get());
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_circle_to_physics_node_cabi<T: GuestCircle>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 =
                        T::to_physics_node(CircleBorrow::lift(arg0 as u32 as usize).get());
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_constructor_ellipse_cabi<T: GuestEllipse>(
                    arg0: f32,
                    arg1: f32,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = Ellipse::new(T::new(
                        super::super::super::super::wired::math::types::Vec2 { x: arg0, y: arg1 },
                    ));
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_ellipse_half_size_cabi<T: GuestEllipse>(
                    arg0: *mut u8,
                ) -> *mut u8 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::half_size(EllipseBorrow::lift(arg0 as u32 as usize).get());
                    let ptr1 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    let super::super::super::super::wired::math::types::Vec2 { x: x2, y: y2 } =
                        result0;
                    *ptr1.add(0).cast::<f32>() = _rt::as_f32(x2);
                    *ptr1.add(4).cast::<f32>() = _rt::as_f32(y2);
                    ptr1
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_ellipse_set_half_size_cabi<T: GuestEllipse>(
                    arg0: *mut u8,
                    arg1: f32,
                    arg2: f32,
                ) {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    T::set_half_size(
                        EllipseBorrow::lift(arg0 as u32 as usize).get(),
                        super::super::super::super::wired::math::types::Vec2 { x: arg1, y: arg2 },
                    );
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_ellipse_resolution_cabi<T: GuestEllipse>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::resolution(EllipseBorrow::lift(arg0 as u32 as usize).get());
                    _rt::as_i32(result0)
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_ellipse_set_resolution_cabi<T: GuestEllipse>(
                    arg0: *mut u8,
                    arg1: i32,
                ) {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    T::set_resolution(EllipseBorrow::lift(arg0 as u32 as usize).get(), arg1 as u16);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_ellipse_to_mesh_cabi<T: GuestEllipse>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::to_mesh(EllipseBorrow::lift(arg0 as u32 as usize).get());
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_ellipse_to_node_cabi<T: GuestEllipse>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::to_node(EllipseBorrow::lift(arg0 as u32 as usize).get());
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_ellipse_to_physics_node_cabi<T: GuestEllipse>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 =
                        T::to_physics_node(EllipseBorrow::lift(arg0 as u32 as usize).get());
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_constructor_cylinder_cabi<T: GuestCylinder>(
                    arg0: f32,
                    arg1: f32,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = Cylinder::new(T::new(arg0, arg1));
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_cylinder_cap_cabi<T: GuestCylinder>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::cap(CylinderBorrow::lift(arg0 as u32 as usize).get());
                    match result0 {
                        true => 1,
                        false => 0,
                    }
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_cylinder_set_cap_cabi<T: GuestCylinder>(
                    arg0: *mut u8,
                    arg1: i32,
                ) {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    T::set_cap(
                        CylinderBorrow::lift(arg0 as u32 as usize).get(),
                        _rt::bool_lift(arg1 as u8),
                    );
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_cylinder_height_cabi<T: GuestCylinder>(
                    arg0: *mut u8,
                ) -> f32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::height(CylinderBorrow::lift(arg0 as u32 as usize).get());
                    _rt::as_f32(result0)
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_cylinder_set_height_cabi<T: GuestCylinder>(
                    arg0: *mut u8,
                    arg1: f32,
                ) {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    T::set_height(CylinderBorrow::lift(arg0 as u32 as usize).get(), arg1);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_cylinder_radius_cabi<T: GuestCylinder>(
                    arg0: *mut u8,
                ) -> f32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::radius(CylinderBorrow::lift(arg0 as u32 as usize).get());
                    _rt::as_f32(result0)
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_cylinder_set_radius_cabi<T: GuestCylinder>(
                    arg0: *mut u8,
                    arg1: f32,
                ) {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    T::set_radius(CylinderBorrow::lift(arg0 as u32 as usize).get(), arg1);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_cylinder_resolution_cabi<T: GuestCylinder>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::resolution(CylinderBorrow::lift(arg0 as u32 as usize).get());
                    _rt::as_i32(result0)
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_cylinder_set_resolution_cabi<T: GuestCylinder>(
                    arg0: *mut u8,
                    arg1: i32,
                ) {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    T::set_resolution(CylinderBorrow::lift(arg0 as u32 as usize).get(), arg1 as u8);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_cylinder_segments_cabi<T: GuestCylinder>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::segments(CylinderBorrow::lift(arg0 as u32 as usize).get());
                    _rt::as_i32(result0)
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_cylinder_set_segments_cabi<T: GuestCylinder>(
                    arg0: *mut u8,
                    arg1: i32,
                ) {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    T::set_segments(CylinderBorrow::lift(arg0 as u32 as usize).get(), arg1 as u8);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_cylinder_to_mesh_cabi<T: GuestCylinder>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::to_mesh(CylinderBorrow::lift(arg0 as u32 as usize).get());
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_cylinder_to_node_cabi<T: GuestCylinder>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::to_node(CylinderBorrow::lift(arg0 as u32 as usize).get());
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_cylinder_to_physics_node_cabi<T: GuestCylinder>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 =
                        T::to_physics_node(CylinderBorrow::lift(arg0 as u32 as usize).get());
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_constructor_cuboid_cabi<T: GuestCuboid>(
                    arg0: f32,
                    arg1: f32,
                    arg2: f32,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = Cuboid::new(T::new(
                        super::super::super::super::wired::math::types::Vec3 {
                            x: arg0,
                            y: arg1,
                            z: arg2,
                        },
                    ));
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_cuboid_size_cabi<T: GuestCuboid>(
                    arg0: *mut u8,
                ) -> *mut u8 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::size(CuboidBorrow::lift(arg0 as u32 as usize).get());
                    let ptr1 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    let super::super::super::super::wired::math::types::Vec3 {
                        x: x2,
                        y: y2,
                        z: z2,
                    } = result0;
                    *ptr1.add(0).cast::<f32>() = _rt::as_f32(x2);
                    *ptr1.add(4).cast::<f32>() = _rt::as_f32(y2);
                    *ptr1.add(8).cast::<f32>() = _rt::as_f32(z2);
                    ptr1
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_cuboid_set_size_cabi<T: GuestCuboid>(
                    arg0: *mut u8,
                    arg1: f32,
                    arg2: f32,
                    arg3: f32,
                ) {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    T::set_size(
                        CuboidBorrow::lift(arg0 as u32 as usize).get(),
                        super::super::super::super::wired::math::types::Vec3 {
                            x: arg1,
                            y: arg2,
                            z: arg3,
                        },
                    );
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_cuboid_to_mesh_cabi<T: GuestCuboid>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::to_mesh(CuboidBorrow::lift(arg0 as u32 as usize).get());
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_cuboid_to_node_cabi<T: GuestCuboid>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::to_node(CuboidBorrow::lift(arg0 as u32 as usize).get());
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_cuboid_to_physics_node_cabi<T: GuestCuboid>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 =
                        T::to_physics_node(CuboidBorrow::lift(arg0 as u32 as usize).get());
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_static_sphere_new_ico_cabi<T: GuestSphere>(arg0: f32) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::new_ico(arg0);
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_static_sphere_new_uv_cabi<T: GuestSphere>(arg0: f32) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::new_uv(arg0);
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_sphere_radius_cabi<T: GuestSphere>(
                    arg0: *mut u8,
                ) -> f32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::radius(SphereBorrow::lift(arg0 as u32 as usize).get());
                    _rt::as_f32(result0)
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_sphere_set_radius_cabi<T: GuestSphere>(
                    arg0: *mut u8,
                    arg1: f32,
                ) {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    T::set_radius(SphereBorrow::lift(arg0 as u32 as usize).get(), arg1);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_sphere_kind_cabi<T: GuestSphere>(
                    arg0: *mut u8,
                ) -> *mut u8 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::kind(SphereBorrow::lift(arg0 as u32 as usize).get());
                    let ptr1 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    match result0 {
                        SphereKind::Ico(e) => {
                            *ptr1.add(0).cast::<u8>() = (0i32) as u8;
                            let SphereIco {
                                subdivisions: subdivisions2,
                            } = e;
                            *ptr1.add(1).cast::<u8>() = (_rt::as_i32(subdivisions2)) as u8;
                        }
                        SphereKind::Uv(e) => {
                            *ptr1.add(0).cast::<u8>() = (1i32) as u8;
                            let SphereUv {
                                sectors: sectors3,
                                stacks: stacks3,
                            } = e;
                            *ptr1.add(1).cast::<u8>() = (_rt::as_i32(sectors3)) as u8;
                            *ptr1.add(2).cast::<u8>() = (_rt::as_i32(stacks3)) as u8;
                        }
                    }
                    ptr1
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_sphere_set_kind_cabi<T: GuestSphere>(
                    arg0: *mut u8,
                    arg1: i32,
                    arg2: i32,
                    arg3: i32,
                ) {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let v0 = match arg1 {
                        0 => {
                            let e0 = SphereIco {
                                subdivisions: arg2 as u8,
                            };
                            SphereKind::Ico(e0)
                        }
                        n => {
                            debug_assert_eq!(n, 1, "invalid enum discriminant");
                            let e0 = SphereUv {
                                sectors: arg2 as u8,
                                stacks: arg3 as u8,
                            };
                            SphereKind::Uv(e0)
                        }
                    };
                    T::set_kind(SphereBorrow::lift(arg0 as u32 as usize).get(), v0);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_sphere_to_mesh_cabi<T: GuestSphere>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::to_mesh(SphereBorrow::lift(arg0 as u32 as usize).get());
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_sphere_to_node_cabi<T: GuestSphere>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::to_node(SphereBorrow::lift(arg0 as u32 as usize).get());
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_sphere_to_physics_node_cabi<T: GuestSphere>(
                    arg0: *mut u8,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 =
                        T::to_physics_node(SphereBorrow::lift(arg0 as u32 as usize).get());
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_constructor_axes_cabi<T: GuestAxes>() -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = Axes::new(T::new());
                    (result0).take_handle() as i32
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_axes_size_cabi<T: GuestAxes>(arg0: *mut u8) -> f32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::size(AxesBorrow::lift(arg0 as u32 as usize).get());
                    _rt::as_f32(result0)
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_axes_set_size_cabi<T: GuestAxes>(
                    arg0: *mut u8,
                    arg1: f32,
                ) {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    T::set_size(AxesBorrow::lift(arg0 as u32 as usize).get(), arg1);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_axes_to_node_cabi<T: GuestAxes>(arg0: *mut u8) -> i32 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::to_node(AxesBorrow::lift(arg0 as u32 as usize).get());
                    (result0).take_handle() as i32
                }
                pub trait Guest {
                    type Rectangle: GuestRectangle;
                    type Circle: GuestCircle;
                    type Ellipse: GuestEllipse;
                    type Cylinder: GuestCylinder;
                    type Cuboid: GuestCuboid;
                    type Sphere: GuestSphere;
                    type Axes: GuestAxes;
                }
                pub trait GuestRectangle: 'static {
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
                            #[link(wasm_import_module = "[export]unavi:shapes/api")]
                            extern "C" {
                                #[link_name = "[resource-new]rectangle"]
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
                            #[link(wasm_import_module = "[export]unavi:shapes/api")]
                            extern "C" {
                                #[link_name = "[resource-rep]rectangle"]
                                fn rep(_: u32) -> *mut u8;
                            }
                            unsafe { rep(handle) }
                        }
                    }
                    fn new(size: Vec2) -> Self;
                    fn size(&self) -> Vec2;
                    fn set_size(&self, value: Vec2);
                    /// Creates a mesh of this shape.
                    fn to_mesh(&self) -> Mesh;
                    /// Creates a node with a mesh of this shape.
                    fn to_node(&self) -> Node;
                    /// Creates a node with a mesh and physics collider of this shape.
                    fn to_physics_node(&self) -> Node;
                }
                pub trait GuestCircle: 'static {
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
                            #[link(wasm_import_module = "[export]unavi:shapes/api")]
                            extern "C" {
                                #[link_name = "[resource-new]circle"]
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
                            #[link(wasm_import_module = "[export]unavi:shapes/api")]
                            extern "C" {
                                #[link_name = "[resource-rep]circle"]
                                fn rep(_: u32) -> *mut u8;
                            }
                            unsafe { rep(handle) }
                        }
                    }
                    fn new(radius: f32) -> Self;
                    fn radius(&self) -> f32;
                    fn set_radius(&self, value: f32);
                    /// The number of vertices used for the mesh.
                    fn resolution(&self) -> u16;
                    fn set_resolution(&self, value: u16);
                    /// Creates a mesh of this shape.
                    fn to_mesh(&self) -> Mesh;
                    /// Creates a node with a mesh of this shape.
                    fn to_node(&self) -> Node;
                    /// Creates a node with a mesh and physics collider of this shape.
                    fn to_physics_node(&self) -> Node;
                }
                pub trait GuestEllipse: 'static {
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
                            #[link(wasm_import_module = "[export]unavi:shapes/api")]
                            extern "C" {
                                #[link_name = "[resource-new]ellipse"]
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
                            #[link(wasm_import_module = "[export]unavi:shapes/api")]
                            extern "C" {
                                #[link_name = "[resource-rep]ellipse"]
                                fn rep(_: u32) -> *mut u8;
                            }
                            unsafe { rep(handle) }
                        }
                    }
                    fn new(half_size: Vec2) -> Self;
                    fn half_size(&self) -> Vec2;
                    fn set_half_size(&self, value: Vec2);
                    /// The number of vertices used for the mesh.
                    fn resolution(&self) -> u16;
                    fn set_resolution(&self, value: u16);
                    /// Creates a mesh of this shape.
                    fn to_mesh(&self) -> Mesh;
                    /// Creates a node with a mesh of this shape.
                    fn to_node(&self) -> Node;
                    /// Creates a node with a mesh and physics collider of this shape.
                    fn to_physics_node(&self) -> Node;
                }
                pub trait GuestCylinder: 'static {
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
                            #[link(wasm_import_module = "[export]unavi:shapes/api")]
                            extern "C" {
                                #[link_name = "[resource-new]cylinder"]
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
                            #[link(wasm_import_module = "[export]unavi:shapes/api")]
                            extern "C" {
                                #[link_name = "[resource-rep]cylinder"]
                                fn rep(_: u32) -> *mut u8;
                            }
                            unsafe { rep(handle) }
                        }
                    }
                    fn new(radius: f32, height: f32) -> Self;
                    /// Whether to cap the ends of the cylinder.
                    fn cap(&self) -> bool;
                    fn set_cap(&self, value: bool);
                    fn height(&self) -> f32;
                    fn set_height(&self, value: f32);
                    fn radius(&self) -> f32;
                    fn set_radius(&self, value: f32);
                    /// The number of vertices used for the top and bottom of the cylinder.
                    fn resolution(&self) -> u8;
                    fn set_resolution(&self, value: u8);
                    /// The number of segments along the height of the cylinder.
                    fn segments(&self) -> u8;
                    fn set_segments(&self, value: u8);
                    /// Creates a mesh of this shape.
                    fn to_mesh(&self) -> Mesh;
                    /// Creates a node with a mesh of this shape.
                    fn to_node(&self) -> Node;
                    /// Creates a node with a mesh and physics collider of this shape.
                    fn to_physics_node(&self) -> Node;
                }
                pub trait GuestCuboid: 'static {
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
                            #[link(wasm_import_module = "[export]unavi:shapes/api")]
                            extern "C" {
                                #[link_name = "[resource-new]cuboid"]
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
                            #[link(wasm_import_module = "[export]unavi:shapes/api")]
                            extern "C" {
                                #[link_name = "[resource-rep]cuboid"]
                                fn rep(_: u32) -> *mut u8;
                            }
                            unsafe { rep(handle) }
                        }
                    }
                    fn new(size: Vec3) -> Self;
                    fn size(&self) -> Vec3;
                    fn set_size(&self, value: Vec3);
                    /// Creates a mesh of this shape.
                    fn to_mesh(&self) -> Mesh;
                    /// Creates a node with a mesh of this shape.
                    fn to_node(&self) -> Node;
                    /// Creates a node with a mesh and physics collider of this shape.
                    fn to_physics_node(&self) -> Node;
                }
                pub trait GuestSphere: 'static {
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
                            #[link(wasm_import_module = "[export]unavi:shapes/api")]
                            extern "C" {
                                #[link_name = "[resource-new]sphere"]
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
                            #[link(wasm_import_module = "[export]unavi:shapes/api")]
                            extern "C" {
                                #[link_name = "[resource-rep]sphere"]
                                fn rep(_: u32) -> *mut u8;
                            }
                            unsafe { rep(handle) }
                        }
                    }
                    fn new_ico(radius: f32) -> Sphere;
                    fn new_uv(radius: f32) -> Sphere;
                    fn radius(&self) -> f32;
                    fn set_radius(&self, value: f32);
                    fn kind(&self) -> SphereKind;
                    fn set_kind(&self, value: SphereKind);
                    /// Creates a mesh of this shape.
                    fn to_mesh(&self) -> Mesh;
                    /// Creates a node with a mesh of this shape.
                    fn to_node(&self) -> Node;
                    /// Creates a node with a mesh and physics collider of this shape.
                    fn to_physics_node(&self) -> Node;
                }
                pub trait GuestAxes: 'static {
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
                            #[link(wasm_import_module = "[export]unavi:shapes/api")]
                            extern "C" {
                                #[link_name = "[resource-new]axes"]
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
                            #[link(wasm_import_module = "[export]unavi:shapes/api")]
                            extern "C" {
                                #[link_name = "[resource-rep]axes"]
                                fn rep(_: u32) -> *mut u8;
                            }
                            unsafe { rep(handle) }
                        }
                    }
                    fn new() -> Self;
                    fn size(&self) -> f32;
                    fn set_size(&self, value: f32);
                    /// Creates a node with a mesh of this shape.
                    fn to_node(&self) -> Node;
                }
                #[doc(hidden)]
                macro_rules! __export_unavi_shapes_api_cabi {
                    ($ty:ident with_types_in $($path_to_types:tt)*) => {
                        const _ : () = { #[export_name =
                        "unavi:shapes/api#[constructor]rectangle"] unsafe extern "C" fn
                        export_constructor_rectangle(arg0 : f32, arg1 : f32,) -> i32 {
                        $($path_to_types)*:: _export_constructor_rectangle_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Rectangle > (arg0, arg1) }
                        #[export_name = "unavi:shapes/api#[method]rectangle.size"] unsafe
                        extern "C" fn export_method_rectangle_size(arg0 : * mut u8,) -> *
                        mut u8 { $($path_to_types)*::
                        _export_method_rectangle_size_cabi::<<$ty as $($path_to_types)*::
                        Guest >::Rectangle > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]rectangle.set-size"] unsafe extern "C"
                        fn export_method_rectangle_set_size(arg0 : * mut u8, arg1 : f32,
                        arg2 : f32,) { $($path_to_types)*::
                        _export_method_rectangle_set_size_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Rectangle > (arg0, arg1, arg2) }
                        #[export_name = "unavi:shapes/api#[method]rectangle.to-mesh"]
                        unsafe extern "C" fn export_method_rectangle_to_mesh(arg0 : * mut
                        u8,) -> i32 { $($path_to_types)*::
                        _export_method_rectangle_to_mesh_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Rectangle > (arg0) } #[export_name
                        = "unavi:shapes/api#[method]rectangle.to-node"] unsafe extern "C"
                        fn export_method_rectangle_to_node(arg0 : * mut u8,) -> i32 {
                        $($path_to_types)*:: _export_method_rectangle_to_node_cabi::<<$ty
                        as $($path_to_types)*:: Guest >::Rectangle > (arg0) }
                        #[export_name =
                        "unavi:shapes/api#[method]rectangle.to-physics-node"] unsafe
                        extern "C" fn export_method_rectangle_to_physics_node(arg0 : *
                        mut u8,) -> i32 { $($path_to_types)*::
                        _export_method_rectangle_to_physics_node_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Rectangle > (arg0) } #[export_name
                        = "unavi:shapes/api#[constructor]circle"] unsafe extern "C" fn
                        export_constructor_circle(arg0 : f32,) -> i32 {
                        $($path_to_types)*:: _export_constructor_circle_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Circle > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]circle.radius"] unsafe extern "C" fn
                        export_method_circle_radius(arg0 : * mut u8,) -> f32 {
                        $($path_to_types)*:: _export_method_circle_radius_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Circle > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]circle.set-radius"] unsafe extern "C"
                        fn export_method_circle_set_radius(arg0 : * mut u8, arg1 : f32,)
                        { $($path_to_types)*::
                        _export_method_circle_set_radius_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Circle > (arg0, arg1) }
                        #[export_name = "unavi:shapes/api#[method]circle.resolution"]
                        unsafe extern "C" fn export_method_circle_resolution(arg0 : * mut
                        u8,) -> i32 { $($path_to_types)*::
                        _export_method_circle_resolution_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Circle > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]circle.set-resolution"] unsafe extern
                        "C" fn export_method_circle_set_resolution(arg0 : * mut u8, arg1
                        : i32,) { $($path_to_types)*::
                        _export_method_circle_set_resolution_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Circle > (arg0, arg1) }
                        #[export_name = "unavi:shapes/api#[method]circle.to-mesh"] unsafe
                        extern "C" fn export_method_circle_to_mesh(arg0 : * mut u8,) ->
                        i32 { $($path_to_types)*::
                        _export_method_circle_to_mesh_cabi::<<$ty as $($path_to_types)*::
                        Guest >::Circle > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]circle.to-node"] unsafe extern "C" fn
                        export_method_circle_to_node(arg0 : * mut u8,) -> i32 {
                        $($path_to_types)*:: _export_method_circle_to_node_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Circle > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]circle.to-physics-node"] unsafe extern
                        "C" fn export_method_circle_to_physics_node(arg0 : * mut u8,) ->
                        i32 { $($path_to_types)*::
                        _export_method_circle_to_physics_node_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Circle > (arg0) } #[export_name =
                        "unavi:shapes/api#[constructor]ellipse"] unsafe extern "C" fn
                        export_constructor_ellipse(arg0 : f32, arg1 : f32,) -> i32 {
                        $($path_to_types)*:: _export_constructor_ellipse_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Ellipse > (arg0, arg1) }
                        #[export_name = "unavi:shapes/api#[method]ellipse.half-size"]
                        unsafe extern "C" fn export_method_ellipse_half_size(arg0 : * mut
                        u8,) -> * mut u8 { $($path_to_types)*::
                        _export_method_ellipse_half_size_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Ellipse > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]ellipse.set-half-size"] unsafe extern
                        "C" fn export_method_ellipse_set_half_size(arg0 : * mut u8, arg1
                        : f32, arg2 : f32,) { $($path_to_types)*::
                        _export_method_ellipse_set_half_size_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Ellipse > (arg0, arg1, arg2) }
                        #[export_name = "unavi:shapes/api#[method]ellipse.resolution"]
                        unsafe extern "C" fn export_method_ellipse_resolution(arg0 : *
                        mut u8,) -> i32 { $($path_to_types)*::
                        _export_method_ellipse_resolution_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Ellipse > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]ellipse.set-resolution"] unsafe extern
                        "C" fn export_method_ellipse_set_resolution(arg0 : * mut u8, arg1
                        : i32,) { $($path_to_types)*::
                        _export_method_ellipse_set_resolution_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Ellipse > (arg0, arg1) }
                        #[export_name = "unavi:shapes/api#[method]ellipse.to-mesh"]
                        unsafe extern "C" fn export_method_ellipse_to_mesh(arg0 : * mut
                        u8,) -> i32 { $($path_to_types)*::
                        _export_method_ellipse_to_mesh_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Ellipse > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]ellipse.to-node"] unsafe extern "C" fn
                        export_method_ellipse_to_node(arg0 : * mut u8,) -> i32 {
                        $($path_to_types)*:: _export_method_ellipse_to_node_cabi::<<$ty
                        as $($path_to_types)*:: Guest >::Ellipse > (arg0) } #[export_name
                        = "unavi:shapes/api#[method]ellipse.to-physics-node"] unsafe
                        extern "C" fn export_method_ellipse_to_physics_node(arg0 : * mut
                        u8,) -> i32 { $($path_to_types)*::
                        _export_method_ellipse_to_physics_node_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Ellipse > (arg0) } #[export_name =
                        "unavi:shapes/api#[constructor]cylinder"] unsafe extern "C" fn
                        export_constructor_cylinder(arg0 : f32, arg1 : f32,) -> i32 {
                        $($path_to_types)*:: _export_constructor_cylinder_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Cylinder > (arg0, arg1) }
                        #[export_name = "unavi:shapes/api#[method]cylinder.cap"] unsafe
                        extern "C" fn export_method_cylinder_cap(arg0 : * mut u8,) -> i32
                        { $($path_to_types)*:: _export_method_cylinder_cap_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Cylinder > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]cylinder.set-cap"] unsafe extern "C" fn
                        export_method_cylinder_set_cap(arg0 : * mut u8, arg1 : i32,) {
                        $($path_to_types)*:: _export_method_cylinder_set_cap_cabi::<<$ty
                        as $($path_to_types)*:: Guest >::Cylinder > (arg0, arg1) }
                        #[export_name = "unavi:shapes/api#[method]cylinder.height"]
                        unsafe extern "C" fn export_method_cylinder_height(arg0 : * mut
                        u8,) -> f32 { $($path_to_types)*::
                        _export_method_cylinder_height_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Cylinder > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]cylinder.set-height"] unsafe extern "C"
                        fn export_method_cylinder_set_height(arg0 : * mut u8, arg1 :
                        f32,) { $($path_to_types)*::
                        _export_method_cylinder_set_height_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Cylinder > (arg0, arg1) }
                        #[export_name = "unavi:shapes/api#[method]cylinder.radius"]
                        unsafe extern "C" fn export_method_cylinder_radius(arg0 : * mut
                        u8,) -> f32 { $($path_to_types)*::
                        _export_method_cylinder_radius_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Cylinder > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]cylinder.set-radius"] unsafe extern "C"
                        fn export_method_cylinder_set_radius(arg0 : * mut u8, arg1 :
                        f32,) { $($path_to_types)*::
                        _export_method_cylinder_set_radius_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Cylinder > (arg0, arg1) }
                        #[export_name = "unavi:shapes/api#[method]cylinder.resolution"]
                        unsafe extern "C" fn export_method_cylinder_resolution(arg0 : *
                        mut u8,) -> i32 { $($path_to_types)*::
                        _export_method_cylinder_resolution_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Cylinder > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]cylinder.set-resolution"] unsafe extern
                        "C" fn export_method_cylinder_set_resolution(arg0 : * mut u8,
                        arg1 : i32,) { $($path_to_types)*::
                        _export_method_cylinder_set_resolution_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Cylinder > (arg0, arg1) }
                        #[export_name = "unavi:shapes/api#[method]cylinder.segments"]
                        unsafe extern "C" fn export_method_cylinder_segments(arg0 : * mut
                        u8,) -> i32 { $($path_to_types)*::
                        _export_method_cylinder_segments_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Cylinder > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]cylinder.set-segments"] unsafe extern
                        "C" fn export_method_cylinder_set_segments(arg0 : * mut u8, arg1
                        : i32,) { $($path_to_types)*::
                        _export_method_cylinder_set_segments_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Cylinder > (arg0, arg1) }
                        #[export_name = "unavi:shapes/api#[method]cylinder.to-mesh"]
                        unsafe extern "C" fn export_method_cylinder_to_mesh(arg0 : * mut
                        u8,) -> i32 { $($path_to_types)*::
                        _export_method_cylinder_to_mesh_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Cylinder > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]cylinder.to-node"] unsafe extern "C" fn
                        export_method_cylinder_to_node(arg0 : * mut u8,) -> i32 {
                        $($path_to_types)*:: _export_method_cylinder_to_node_cabi::<<$ty
                        as $($path_to_types)*:: Guest >::Cylinder > (arg0) }
                        #[export_name =
                        "unavi:shapes/api#[method]cylinder.to-physics-node"] unsafe
                        extern "C" fn export_method_cylinder_to_physics_node(arg0 : * mut
                        u8,) -> i32 { $($path_to_types)*::
                        _export_method_cylinder_to_physics_node_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Cylinder > (arg0) } #[export_name =
                        "unavi:shapes/api#[constructor]cuboid"] unsafe extern "C" fn
                        export_constructor_cuboid(arg0 : f32, arg1 : f32, arg2 : f32,) ->
                        i32 { $($path_to_types)*:: _export_constructor_cuboid_cabi::<<$ty
                        as $($path_to_types)*:: Guest >::Cuboid > (arg0, arg1, arg2) }
                        #[export_name = "unavi:shapes/api#[method]cuboid.size"] unsafe
                        extern "C" fn export_method_cuboid_size(arg0 : * mut u8,) -> *
                        mut u8 { $($path_to_types)*::
                        _export_method_cuboid_size_cabi::<<$ty as $($path_to_types)*::
                        Guest >::Cuboid > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]cuboid.set-size"] unsafe extern "C" fn
                        export_method_cuboid_set_size(arg0 : * mut u8, arg1 : f32, arg2 :
                        f32, arg3 : f32,) { $($path_to_types)*::
                        _export_method_cuboid_set_size_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Cuboid > (arg0, arg1, arg2, arg3) }
                        #[export_name = "unavi:shapes/api#[method]cuboid.to-mesh"] unsafe
                        extern "C" fn export_method_cuboid_to_mesh(arg0 : * mut u8,) ->
                        i32 { $($path_to_types)*::
                        _export_method_cuboid_to_mesh_cabi::<<$ty as $($path_to_types)*::
                        Guest >::Cuboid > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]cuboid.to-node"] unsafe extern "C" fn
                        export_method_cuboid_to_node(arg0 : * mut u8,) -> i32 {
                        $($path_to_types)*:: _export_method_cuboid_to_node_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Cuboid > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]cuboid.to-physics-node"] unsafe extern
                        "C" fn export_method_cuboid_to_physics_node(arg0 : * mut u8,) ->
                        i32 { $($path_to_types)*::
                        _export_method_cuboid_to_physics_node_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Cuboid > (arg0) } #[export_name =
                        "unavi:shapes/api#[static]sphere.new-ico"] unsafe extern "C" fn
                        export_static_sphere_new_ico(arg0 : f32,) -> i32 {
                        $($path_to_types)*:: _export_static_sphere_new_ico_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Sphere > (arg0) } #[export_name =
                        "unavi:shapes/api#[static]sphere.new-uv"] unsafe extern "C" fn
                        export_static_sphere_new_uv(arg0 : f32,) -> i32 {
                        $($path_to_types)*:: _export_static_sphere_new_uv_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Sphere > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]sphere.radius"] unsafe extern "C" fn
                        export_method_sphere_radius(arg0 : * mut u8,) -> f32 {
                        $($path_to_types)*:: _export_method_sphere_radius_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Sphere > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]sphere.set-radius"] unsafe extern "C"
                        fn export_method_sphere_set_radius(arg0 : * mut u8, arg1 : f32,)
                        { $($path_to_types)*::
                        _export_method_sphere_set_radius_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Sphere > (arg0, arg1) }
                        #[export_name = "unavi:shapes/api#[method]sphere.kind"] unsafe
                        extern "C" fn export_method_sphere_kind(arg0 : * mut u8,) -> *
                        mut u8 { $($path_to_types)*::
                        _export_method_sphere_kind_cabi::<<$ty as $($path_to_types)*::
                        Guest >::Sphere > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]sphere.set-kind"] unsafe extern "C" fn
                        export_method_sphere_set_kind(arg0 : * mut u8, arg1 : i32, arg2 :
                        i32, arg3 : i32,) { $($path_to_types)*::
                        _export_method_sphere_set_kind_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Sphere > (arg0, arg1, arg2, arg3) }
                        #[export_name = "unavi:shapes/api#[method]sphere.to-mesh"] unsafe
                        extern "C" fn export_method_sphere_to_mesh(arg0 : * mut u8,) ->
                        i32 { $($path_to_types)*::
                        _export_method_sphere_to_mesh_cabi::<<$ty as $($path_to_types)*::
                        Guest >::Sphere > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]sphere.to-node"] unsafe extern "C" fn
                        export_method_sphere_to_node(arg0 : * mut u8,) -> i32 {
                        $($path_to_types)*:: _export_method_sphere_to_node_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Sphere > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]sphere.to-physics-node"] unsafe extern
                        "C" fn export_method_sphere_to_physics_node(arg0 : * mut u8,) ->
                        i32 { $($path_to_types)*::
                        _export_method_sphere_to_physics_node_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Sphere > (arg0) } #[export_name =
                        "unavi:shapes/api#[constructor]axes"] unsafe extern "C" fn
                        export_constructor_axes() -> i32 { $($path_to_types)*::
                        _export_constructor_axes_cabi::<<$ty as $($path_to_types)*::
                        Guest >::Axes > () } #[export_name =
                        "unavi:shapes/api#[method]axes.size"] unsafe extern "C" fn
                        export_method_axes_size(arg0 : * mut u8,) -> f32 {
                        $($path_to_types)*:: _export_method_axes_size_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Axes > (arg0) } #[export_name =
                        "unavi:shapes/api#[method]axes.set-size"] unsafe extern "C" fn
                        export_method_axes_set_size(arg0 : * mut u8, arg1 : f32,) {
                        $($path_to_types)*:: _export_method_axes_set_size_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Axes > (arg0, arg1) } #[export_name
                        = "unavi:shapes/api#[method]axes.to-node"] unsafe extern "C" fn
                        export_method_axes_to_node(arg0 : * mut u8,) -> i32 {
                        $($path_to_types)*:: _export_method_axes_to_node_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::Axes > (arg0) } const _ : () = {
                        #[doc(hidden)] #[export_name =
                        "unavi:shapes/api#[dtor]rectangle"] #[allow(non_snake_case)]
                        unsafe extern "C" fn dtor(rep : * mut u8) { $($path_to_types)*::
                        Rectangle::dtor::< <$ty as $($path_to_types)*:: Guest
                        >::Rectangle > (rep) } }; const _ : () = { #[doc(hidden)]
                        #[export_name = "unavi:shapes/api#[dtor]circle"]
                        #[allow(non_snake_case)] unsafe extern "C" fn dtor(rep : * mut
                        u8) { $($path_to_types)*:: Circle::dtor::< <$ty as
                        $($path_to_types)*:: Guest >::Circle > (rep) } }; const _ : () =
                        { #[doc(hidden)] #[export_name =
                        "unavi:shapes/api#[dtor]ellipse"] #[allow(non_snake_case)] unsafe
                        extern "C" fn dtor(rep : * mut u8) { $($path_to_types)*::
                        Ellipse::dtor::< <$ty as $($path_to_types)*:: Guest >::Ellipse >
                        (rep) } }; const _ : () = { #[doc(hidden)] #[export_name =
                        "unavi:shapes/api#[dtor]cylinder"] #[allow(non_snake_case)]
                        unsafe extern "C" fn dtor(rep : * mut u8) { $($path_to_types)*::
                        Cylinder::dtor::< <$ty as $($path_to_types)*:: Guest >::Cylinder
                        > (rep) } }; const _ : () = { #[doc(hidden)] #[export_name =
                        "unavi:shapes/api#[dtor]cuboid"] #[allow(non_snake_case)] unsafe
                        extern "C" fn dtor(rep : * mut u8) { $($path_to_types)*::
                        Cuboid::dtor::< <$ty as $($path_to_types)*:: Guest >::Cuboid >
                        (rep) } }; const _ : () = { #[doc(hidden)] #[export_name =
                        "unavi:shapes/api#[dtor]sphere"] #[allow(non_snake_case)] unsafe
                        extern "C" fn dtor(rep : * mut u8) { $($path_to_types)*::
                        Sphere::dtor::< <$ty as $($path_to_types)*:: Guest >::Sphere >
                        (rep) } }; const _ : () = { #[doc(hidden)] #[export_name =
                        "unavi:shapes/api#[dtor]axes"] #[allow(non_snake_case)] unsafe
                        extern "C" fn dtor(rep : * mut u8) { $($path_to_types)*::
                        Axes::dtor::< <$ty as $($path_to_types)*:: Guest >::Axes > (rep)
                        } }; };
                    };
                }
                #[doc(hidden)]
                pub(crate) use __export_unavi_shapes_api_cabi;
                #[repr(align(4))]
                struct _RetArea([::core::mem::MaybeUninit<u8>; 12]);
                static mut _RET_AREA: _RetArea = _RetArea([::core::mem::MaybeUninit::uninit(); 12]);
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
    pub use alloc_crate::boxed::Box;
    #[cfg(target_arch = "wasm32")]
    pub fn run_ctors_once() {
        wit_bindgen_rt::run_ctors_once();
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
macro_rules! __export_guest_impl {
    ($ty:ident) => {
        self::export!($ty with_types_in self);
    };
    ($ty:ident with_types_in $($path_to_types_root:tt)*) => {
        $($path_to_types_root)*::
        exports::unavi::shapes::api::__export_unavi_shapes_api_cabi!($ty with_types_in
        $($path_to_types_root)*:: exports::unavi::shapes::api);
    };
}
#[doc(inline)]
pub(crate) use __export_guest_impl as export;
#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.30.0:guest:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 6234] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\xde/\x01A\x02\x01A\x1d\
\x01B\x04\x01m\x04\x05debug\x04info\x04warn\x05error\x04\0\x09log-level\x03\0\0\x01\
@\x02\x05level\x01\x07messages\x01\0\x04\0\x03log\x01\x02\x03\x01\x0dwired:log/a\
pi\x05\0\x01B\x13\x01r\x04\x01rv\x01gv\x01bv\x01av\x04\0\x05color\x03\0\0\x04\0\x08\
material\x03\x01\x01i\x02\x01@\0\0\x03\x04\0\x15[constructor]material\x01\x04\x01\
h\x02\x01@\x01\x04self\x05\0y\x04\0\x13[method]material.id\x01\x06\x01@\x01\x04s\
elf\x05\0\x03\x04\0\x14[method]material.ref\x01\x07\x01@\x01\x04self\x05\0s\x04\0\
\x15[method]material.name\x01\x08\x01@\x02\x04self\x05\x05values\x01\0\x04\0\x19\
[method]material.set-name\x01\x09\x01@\x01\x04self\x05\0\x01\x04\0\x16[method]ma\
terial.color\x01\x0a\x01@\x02\x04self\x05\x05value\x01\x01\0\x04\0\x1a[method]ma\
terial.set-color\x01\x0b\x03\x01\x14wired:scene/material\x05\x01\x02\x03\0\x01\x08\
material\x01B+\x02\x03\x02\x01\x02\x04\0\x08material\x03\0\0\x04\0\x09primitive\x03\
\x01\x04\0\x04mesh\x03\x01\x01h\x02\x01@\x01\x04self\x04\0y\x04\0\x14[method]pri\
mitive.id\x01\x05\x01i\x01\x01k\x06\x01@\x01\x04self\x04\0\x07\x04\0\x1a[method]\
primitive.material\x01\x08\x01h\x01\x01k\x09\x01@\x02\x04self\x04\x05value\x0a\x01\
\0\x04\0\x1e[method]primitive.set-material\x01\x0b\x01py\x01@\x02\x04self\x04\x05\
value\x0c\x01\0\x04\0\x1d[method]primitive.set-indices\x01\x0d\x01pv\x01@\x02\x04\
self\x04\x05value\x0e\x01\0\x04\0\x1d[method]primitive.set-normals\x01\x0f\x04\0\
\x1f[method]primitive.set-positions\x01\x0f\x04\0\x19[method]primitive.set-uvs\x01\
\x0f\x01i\x03\x01@\0\0\x10\x04\0\x11[constructor]mesh\x01\x11\x01h\x03\x01@\x01\x04\
self\x12\0y\x04\0\x0f[method]mesh.id\x01\x13\x01@\x01\x04self\x12\0\x10\x04\0\x10\
[method]mesh.ref\x01\x14\x01@\x01\x04self\x12\0s\x04\0\x11[method]mesh.name\x01\x15\
\x01@\x02\x04self\x12\x05values\x01\0\x04\0\x15[method]mesh.set-name\x01\x16\x01\
i\x02\x01p\x17\x01@\x01\x04self\x12\0\x18\x04\0\x1c[method]mesh.list-primitives\x01\
\x19\x01@\x01\x04self\x12\0\x17\x04\0\x1d[method]mesh.create-primitive\x01\x1a\x01\
@\x02\x04self\x12\x05value\x17\x01\0\x04\0\x1d[method]mesh.remove-primitive\x01\x1b\
\x03\x01\x10wired:scene/mesh\x05\x03\x01B\x10\x01r\x02\x01xv\x01yv\x04\0\x04vec2\
\x03\0\0\x01r\x03\x01xv\x01yv\x01zv\x04\0\x04vec3\x03\0\x02\x01r\x04\x01xv\x01yv\
\x01zv\x01wv\x04\0\x04quat\x03\0\x04\x01r\x03\x08rotation\x05\x05scale\x03\x0btr\
anslation\x03\x04\0\x09transform\x03\0\x06\x01@\0\0\x01\x04\0\x09fake-fn-a\x01\x08\
\x01@\0\0\x03\x04\0\x09fake-fn-b\x01\x09\x01@\0\0\x05\x04\0\x09fake-fn-c\x01\x0a\
\x01@\0\0\x07\x04\0\x09fake-fn-d\x01\x0b\x03\x01\x10wired:math/types\x05\x04\x02\
\x03\0\x03\x04vec3\x02\x03\0\x03\x04quat\x01B\x15\x02\x03\x02\x01\x05\x04\0\x04v\
ec3\x03\0\0\x02\x03\x02\x01\x06\x04\0\x04quat\x03\0\x02\x01m\x02\x04left\x05righ\
t\x04\0\x09hand-side\x03\0\x04\x01r\x03\x0btranslation\x01\x08rotation\x03\x06ra\
diusv\x04\0\x05joint\x03\0\x06\x01r\x04\x03tip\x07\x06distal\x07\x08proximal\x07\
\x0ametacarpal\x07\x04\0\x06finger\x03\0\x08\x01k\x07\x01r\x09\x04side\x05\x05th\
umb\x09\x05index\x09\x06middle\x09\x04ring\x09\x06little\x09\x04palm\x07\x05wris\
t\x07\x05elbow\x0a\x04\0\x04hand\x03\0\x0b\x01r\x02\x0borientation\x03\x06origin\
\x01\x04\0\x03ray\x03\0\x0d\x01q\x02\x04hand\x01\x0c\0\x03ray\x01\x0e\0\x04\0\x0a\
input-data\x03\0\x0f\x01q\x02\x09collision\0\0\x05hover\0\0\x04\0\x0cinput-actio\
n\x03\0\x11\x01r\x03\x02idw\x06action\x12\x04data\x10\x04\0\x0binput-event\x03\0\
\x13\x03\x01\x11wired:input/types\x05\x07\x02\x03\0\x04\x0binput-event\x01B\x0a\x02\
\x03\x02\x01\x08\x04\0\x0binput-event\x03\0\0\x04\0\x0dinput-handler\x03\x01\x01\
i\x02\x01@\0\0\x03\x04\0\x1a[constructor]input-handler\x01\x04\x01h\x02\x01k\x01\
\x01@\x01\x04self\x05\0\x06\x04\0\x1a[method]input-handler.next\x01\x07\x03\x01\x13\
wired:input/handler\x05\x09\x01B\x1c\x02\x03\x02\x01\x05\x04\0\x04vec3\x03\0\0\x04\
\0\x08collider\x03\x01\x01r\x02\x06heightv\x06radiusv\x04\0\x0eshape-cylinder\x03\
\0\x03\x01q\x03\x06cuboid\x01\x01\0\x08cylinder\x01\x04\0\x06sphere\x01v\0\x04\0\
\x05shape\x03\0\x05\x04\0\x0arigid-body\x03\x01\x01m\x03\x07dynamic\x05fixed\x09\
kinematic\x04\0\x0frigid-body-type\x03\0\x08\x01i\x02\x01@\x01\x05shape\x06\0\x0a\
\x04\0\x15[constructor]collider\x01\x0b\x01h\x02\x01@\x01\x04self\x0c\0v\x04\0\x18\
[method]collider.density\x01\x0d\x01@\x02\x04self\x0c\x05valuev\x01\0\x04\0\x1c[\
method]collider.set-density\x01\x0e\x01i\x07\x01@\x01\x0frigid-body-type\x09\0\x0f\
\x04\0\x17[constructor]rigid-body\x01\x10\x01h\x07\x01@\x01\x04self\x11\0\x01\x04\
\0\x19[method]rigid-body.angvel\x01\x12\x01@\x02\x04self\x11\x05value\x01\x01\0\x04\
\0\x1d[method]rigid-body.set-angvel\x01\x13\x04\0\x19[method]rigid-body.linvel\x01\
\x12\x04\0\x1d[method]rigid-body.set-linvel\x01\x13\x03\x01\x13wired:physics/typ\
es\x05\x0a\x02\x03\0\x02\x04mesh\x02\x03\0\x05\x0dinput-handler\x02\x03\0\x03\x09\
transform\x02\x03\0\x06\x08collider\x02\x03\0\x06\x0arigid-body\x01BE\x02\x03\x02\
\x01\x0b\x04\0\x04mesh\x03\0\0\x02\x03\x02\x01\x0c\x04\0\x0dinput-handler\x03\0\x02\
\x02\x03\x02\x01\x0d\x04\0\x09transform\x03\0\x04\x02\x03\x02\x01\x0e\x04\0\x08c\
ollider\x03\0\x06\x02\x03\x02\x01\x0f\x04\0\x0arigid-body\x03\0\x08\x04\0\x04nod\
e\x03\x01\x01i\x0a\x01@\0\0\x0b\x04\0\x11[constructor]node\x01\x0c\x01h\x0a\x01@\
\x01\x04self\x0d\0y\x04\0\x0f[method]node.id\x01\x0e\x01@\x01\x04self\x0d\0\x0b\x04\
\0\x10[method]node.ref\x01\x0f\x01@\x01\x04self\x0d\0s\x04\0\x11[method]node.nam\
e\x01\x10\x01@\x02\x04self\x0d\x05values\x01\0\x04\0\x15[method]node.set-name\x01\
\x11\x01k\x0b\x01@\x01\x04self\x0d\0\x12\x04\0\x13[method]node.parent\x01\x13\x01\
p\x0b\x01@\x01\x04self\x0d\0\x14\x04\0\x15[method]node.children\x01\x15\x01@\x02\
\x04self\x0d\x05value\x0d\x01\0\x04\0\x16[method]node.add-child\x01\x16\x04\0\x19\
[method]node.remove-child\x01\x16\x01@\x01\x04self\x0d\0\x05\x04\0\x1d[method]no\
de.global-transform\x01\x17\x04\0\x16[method]node.transform\x01\x17\x01@\x02\x04\
self\x0d\x05value\x05\x01\0\x04\0\x1a[method]node.set-transform\x01\x18\x01i\x01\
\x01k\x19\x01@\x01\x04self\x0d\0\x1a\x04\0\x11[method]node.mesh\x01\x1b\x01h\x01\
\x01k\x1c\x01@\x02\x04self\x0d\x05value\x1d\x01\0\x04\0\x15[method]node.set-mesh\
\x01\x1e\x01i\x07\x01k\x1f\x01@\x01\x04self\x0d\0\x20\x04\0\x15[method]node.coll\
ider\x01!\x01h\x07\x01k\"\x01@\x02\x04self\x0d\x05value#\x01\0\x04\0\x19[method]\
node.set-collider\x01$\x01i\x09\x01k%\x01@\x01\x04self\x0d\0&\x04\0\x17[method]n\
ode.rigid-body\x01'\x01h\x09\x01k(\x01@\x02\x04self\x0d\x05value)\x01\0\x04\0\x1b\
[method]node.set-rigid-body\x01*\x01i\x03\x01k+\x01@\x01\x04self\x0d\0,\x04\0\x1a\
[method]node.input-handler\x01-\x01h\x03\x01k.\x01@\x02\x04self\x0d\x05value/\x01\
\0\x04\0\x1e[method]node.set-input-handler\x010\x03\x01\x10wired:scene/node\x05\x10\
\x02\x03\0\x03\x04vec2\x02\x03\0\x07\x04node\x01B\x88\x01\x02\x03\x02\x01\x11\x04\
\0\x04vec2\x03\0\0\x02\x03\x02\x01\x05\x04\0\x04vec3\x03\0\x02\x02\x03\x02\x01\x0b\
\x04\0\x04mesh\x03\0\x04\x02\x03\x02\x01\x12\x04\0\x04node\x03\0\x06\x04\0\x09re\
ctangle\x03\x01\x04\0\x06circle\x03\x01\x04\0\x07ellipse\x03\x01\x04\0\x08cylind\
er\x03\x01\x04\0\x06cuboid\x03\x01\x01r\x01\x0csubdivisions}\x04\0\x0asphere-ico\
\x03\0\x0d\x01r\x02\x07sectors}\x06stacks}\x04\0\x09sphere-uv\x03\0\x0f\x01q\x02\
\x03ico\x01\x0e\0\x02uv\x01\x10\0\x04\0\x0bsphere-kind\x03\0\x11\x04\0\x06sphere\
\x03\x01\x04\0\x04axes\x03\x01\x01i\x08\x01@\x01\x04size\x01\0\x15\x04\0\x16[con\
structor]rectangle\x01\x16\x01h\x08\x01@\x01\x04self\x17\0\x01\x04\0\x16[method]\
rectangle.size\x01\x18\x01@\x02\x04self\x17\x05value\x01\x01\0\x04\0\x1a[method]\
rectangle.set-size\x01\x19\x01i\x05\x01@\x01\x04self\x17\0\x1a\x04\0\x19[method]\
rectangle.to-mesh\x01\x1b\x01i\x07\x01@\x01\x04self\x17\0\x1c\x04\0\x19[method]r\
ectangle.to-node\x01\x1d\x04\0![method]rectangle.to-physics-node\x01\x1d\x01i\x09\
\x01@\x01\x06radiusv\0\x1e\x04\0\x13[constructor]circle\x01\x1f\x01h\x09\x01@\x01\
\x04self\x20\0v\x04\0\x15[method]circle.radius\x01!\x01@\x02\x04self\x20\x05valu\
ev\x01\0\x04\0\x19[method]circle.set-radius\x01\"\x01@\x01\x04self\x20\0{\x04\0\x19\
[method]circle.resolution\x01#\x01@\x02\x04self\x20\x05value{\x01\0\x04\0\x1d[me\
thod]circle.set-resolution\x01$\x01@\x01\x04self\x20\0\x1a\x04\0\x16[method]circ\
le.to-mesh\x01%\x01@\x01\x04self\x20\0\x1c\x04\0\x16[method]circle.to-node\x01&\x04\
\0\x1e[method]circle.to-physics-node\x01&\x01i\x0a\x01@\x01\x09half-size\x01\0'\x04\
\0\x14[constructor]ellipse\x01(\x01h\x0a\x01@\x01\x04self)\0\x01\x04\0\x19[metho\
d]ellipse.half-size\x01*\x01@\x02\x04self)\x05value\x01\x01\0\x04\0\x1d[method]e\
llipse.set-half-size\x01+\x01@\x01\x04self)\0{\x04\0\x1a[method]ellipse.resoluti\
on\x01,\x01@\x02\x04self)\x05value{\x01\0\x04\0\x1e[method]ellipse.set-resolutio\
n\x01-\x01@\x01\x04self)\0\x1a\x04\0\x17[method]ellipse.to-mesh\x01.\x01@\x01\x04\
self)\0\x1c\x04\0\x17[method]ellipse.to-node\x01/\x04\0\x1f[method]ellipse.to-ph\
ysics-node\x01/\x01i\x0b\x01@\x02\x06radiusv\x06heightv\00\x04\0\x15[constructor\
]cylinder\x011\x01h\x0b\x01@\x01\x04self2\0\x7f\x04\0\x14[method]cylinder.cap\x01\
3\x01@\x02\x04self2\x05value\x7f\x01\0\x04\0\x18[method]cylinder.set-cap\x014\x01\
@\x01\x04self2\0v\x04\0\x17[method]cylinder.height\x015\x01@\x02\x04self2\x05val\
uev\x01\0\x04\0\x1b[method]cylinder.set-height\x016\x04\0\x17[method]cylinder.ra\
dius\x015\x04\0\x1b[method]cylinder.set-radius\x016\x01@\x01\x04self2\0}\x04\0\x1b\
[method]cylinder.resolution\x017\x01@\x02\x04self2\x05value}\x01\0\x04\0\x1f[met\
hod]cylinder.set-resolution\x018\x04\0\x19[method]cylinder.segments\x017\x04\0\x1d\
[method]cylinder.set-segments\x018\x01@\x01\x04self2\0\x1a\x04\0\x18[method]cyli\
nder.to-mesh\x019\x01@\x01\x04self2\0\x1c\x04\0\x18[method]cylinder.to-node\x01:\
\x04\0\x20[method]cylinder.to-physics-node\x01:\x01i\x0c\x01@\x01\x04size\x03\0;\
\x04\0\x13[constructor]cuboid\x01<\x01h\x0c\x01@\x01\x04self=\0\x03\x04\0\x13[me\
thod]cuboid.size\x01>\x01@\x02\x04self=\x05value\x03\x01\0\x04\0\x17[method]cubo\
id.set-size\x01?\x01@\x01\x04self=\0\x1a\x04\0\x16[method]cuboid.to-mesh\x01@\x01\
@\x01\x04self=\0\x1c\x04\0\x16[method]cuboid.to-node\x01A\x04\0\x1e[method]cuboi\
d.to-physics-node\x01A\x01i\x13\x01@\x01\x06radiusv\0\xc2\0\x04\0\x16[static]sph\
ere.new-ico\x01C\x04\0\x15[static]sphere.new-uv\x01C\x01h\x13\x01@\x01\x04self\xc4\
\0\0v\x04\0\x15[method]sphere.radius\x01E\x01@\x02\x04self\xc4\0\x05valuev\x01\0\
\x04\0\x19[method]sphere.set-radius\x01F\x01@\x01\x04self\xc4\0\0\x12\x04\0\x13[\
method]sphere.kind\x01G\x01@\x02\x04self\xc4\0\x05value\x12\x01\0\x04\0\x17[meth\
od]sphere.set-kind\x01H\x01@\x01\x04self\xc4\0\0\x1a\x04\0\x16[method]sphere.to-\
mesh\x01I\x01@\x01\x04self\xc4\0\0\x1c\x04\0\x16[method]sphere.to-node\x01J\x04\0\
\x1e[method]sphere.to-physics-node\x01J\x01i\x14\x01@\0\0\xcb\0\x04\0\x11[constr\
uctor]axes\x01L\x01h\x14\x01@\x01\x04self\xcd\0\0v\x04\0\x11[method]axes.size\x01\
N\x01@\x02\x04self\xcd\0\x05valuev\x01\0\x04\0\x15[method]axes.set-size\x01O\x01\
@\x01\x04self\xcd\0\0\x1c\x04\0\x14[method]axes.to-node\x01P\x04\x01\x10unavi:sh\
apes/api\x05\x13\x04\x01\x12unavi:shapes/guest\x04\0\x0b\x0b\x01\0\x05guest\x03\0\
\0\0G\x09producers\x01\x0cprocessed-by\x02\x0dwit-component\x070.215.0\x10wit-bi\
ndgen-rust\x060.30.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
