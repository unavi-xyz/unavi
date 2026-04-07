use std::sync::Arc;
use std::sync::atomic::Ordering;

use js_sys::Object;
use wasm_bindgen::JsValue;

use super::state::MatEntry;
use super::with_script;
use crate::core_ops;

pub fn register(obj: &Object) {
    reg!(
        obj,
        "hostSceneMaterialDrop",
        dyn Fn(u32, u32),
        |id: u32, rep: u32| {
            with_script(id, |state| {
                state.mats.remove(&rep);
            });
        }
    );

    reg!(
        obj,
        "hostSceneMaterialId",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                state
                    .mats
                    .get(&rep)
                    .map(|entry| JsValue::from_str(&entry.inner.id))
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostSceneMaterialClone",
        dyn Fn(u32, u32) -> u32,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let entry = state.mats.get(&rep)?;
                let new_rep = state.alloc();
                state.mats.insert(
                    new_rep,
                    MatEntry {
                        inner: Arc::clone(&entry.inner),
                        doc_entity: entry.doc_entity,
                    },
                );
                Some(new_rep)
            })
            .flatten()
            .unwrap_or(0)
        }
    );

    reg!(
        obj,
        "hostSceneMaterialName",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let entry = state.mats.get(&rep)?;
                let locked = entry.inner.state.lock().ok()?;
                Some(
                    locked
                        .name
                        .as_deref()
                        .map(JsValue::from_str)
                        .unwrap_or(JsValue::NULL),
                )
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostSceneMaterialSetName",
        dyn Fn(u32, u32, JsValue),
        |id: u32, rep: u32, value: JsValue| {
            with_script(id, |state| {
                let entry = state.mats.get(&rep)?;
                let inner = Arc::clone(&entry.inner);
                let doc = entry.doc_entity;
                core_ops::material::set_name(
                    &inner,
                    doc,
                    value.as_string(),
                    &mut state.command_queue,
                );
                Some(())
            });
        }
    );

    reg!(
        obj,
        "hostSceneMaterialBaseColor",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let [r, g, b, a] = state.mats.get(&rep)?.inner.state.lock().ok()?.base_color;
                let obj = Object::new();
                js_sys::Reflect::set(&obj, &"r".into(), &JsValue::from_f64(f64::from(r))).ok();
                js_sys::Reflect::set(&obj, &"g".into(), &JsValue::from_f64(f64::from(g))).ok();
                js_sys::Reflect::set(&obj, &"b".into(), &JsValue::from_f64(f64::from(b))).ok();
                js_sys::Reflect::set(&obj, &"a".into(), &JsValue::from_f64(f64::from(a))).ok();
                Some(JsValue::from(obj))
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostSceneMaterialSetBaseColor",
        dyn Fn(u32, u32, f64, f64, f64, f64),
        |id: u32, rep: u32, r: f64, g: f64, b: f64, a: f64| {
            with_script(id, |state| {
                let entry = state.mats.get(&rep)?;
                let inner = Arc::clone(&entry.inner);
                let doc = entry.doc_entity;
                core_ops::material::set_base_color(
                    &inner,
                    doc,
                    [r as f32, g as f32, b as f32, a as f32],
                    &mut state.command_queue,
                );
                Some(())
            });
        }
    );

    macro_rules! mat_f32_prop {
        ($obj:expr, $getter:literal, $setter:literal, $field:ident, $default:expr, $setter_fn:path) => {
            reg!(
                $obj,
                $getter,
                dyn Fn(u32, u32) -> f64,
                |id: u32, rep: u32| {
                    with_script(id, |state| {
                        state.mats.get(&rep).and_then(|entry| {
                            entry
                                .inner
                                .state
                                .lock()
                                .ok()
                                .map(|locked| f64::from(locked.$field))
                        })
                    })
                    .flatten()
                    .unwrap_or($default)
                }
            );
            reg!(
                $obj,
                $setter,
                dyn Fn(u32, u32, f64),
                |id: u32, rep: u32, value: f64| {
                    with_script(id, |state| {
                        let entry = state.mats.get(&rep)?;
                        let inner = Arc::clone(&entry.inner);
                        let doc = entry.doc_entity;
                        $setter_fn(&inner, doc, value as f32, &mut state.command_queue);
                        Some(())
                    });
                }
            );
        };
    }

    mat_f32_prop!(
        obj,
        "hostSceneMaterialMetallic",
        "hostSceneMaterialSetMetallic",
        metallic,
        0.0,
        core_ops::material::set_metallic
    );
    mat_f32_prop!(
        obj,
        "hostSceneMaterialRoughness",
        "hostSceneMaterialSetRoughness",
        roughness,
        0.5,
        core_ops::material::set_roughness
    );

    macro_rules! mat_bool_prop {
        ($obj:expr, $getter:literal, $setter:literal, $field:ident, $default:expr, $setter_fn:path) => {
            reg!(
                $obj,
                $getter,
                dyn Fn(u32, u32) -> bool,
                |id: u32, rep: u32| {
                    with_script(id, |state| {
                        state.mats.get(&rep).and_then(|entry| {
                            entry.inner.state.lock().ok().map(|locked| locked.$field)
                        })
                    })
                    .flatten()
                    .unwrap_or($default)
                }
            );
            reg!(
                $obj,
                $setter,
                dyn Fn(u32, u32, bool),
                |id: u32, rep: u32, value: bool| {
                    with_script(id, |state| {
                        let entry = state.mats.get(&rep)?;
                        let inner = Arc::clone(&entry.inner);
                        let doc = entry.doc_entity;
                        $setter_fn(&inner, doc, value, &mut state.command_queue);
                        Some(())
                    });
                }
            );
        };
    }

    mat_bool_prop!(
        obj,
        "hostSceneMaterialDoubleSided",
        "hostSceneMaterialSetDoubleSided",
        double_sided,
        false,
        core_ops::material::set_double_sided
    );
    mat_bool_prop!(
        obj,
        "hostSceneMaterialUnlit",
        "hostSceneMaterialSetUnlit",
        unlit,
        false,
        core_ops::material::set_unlit
    );

    reg!(
        obj,
        "hostSceneMaterialAlphaCutoff",
        dyn Fn(u32, u32) -> f64,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                state.mats.get(&rep).and_then(|entry| {
                    entry
                        .inner
                        .state
                        .lock()
                        .ok()
                        .map(|locked| f64::from(locked.alpha_cutoff.unwrap_or(0.5)))
                })
            })
            .flatten()
            .unwrap_or(0.5)
        }
    );

    reg!(
        obj,
        "hostSceneMaterialSetAlphaCutoff",
        dyn Fn(u32, u32, f64),
        |id: u32, rep: u32, value: f64| {
            with_script(id, |state| {
                let entry = state.mats.get(&rep)?;
                let inner = Arc::clone(&entry.inner);
                let doc = entry.doc_entity;
                core_ops::material::set_alpha_cutoff(
                    &inner,
                    doc,
                    value as f32,
                    &mut state.command_queue,
                );
                Some(())
            });
        }
    );

    reg!(
        obj,
        "hostSceneMaterialAlphaMode",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let entry = state.mats.get(&rep)?;
                let locked = entry.inner.state.lock().ok()?;
                let mode = locked
                    .alpha_mode
                    .as_deref()
                    .map(|mode| match mode {
                        "add" => 0i32,
                        "blend" => 1,
                        "mask" => 2,
                        "multiply" => 3,
                        "opaque" => 4,
                        "premultiplied" => 5,
                        _ => -1,
                    })
                    .unwrap_or(-1);
                if mode < 0 {
                    Some(JsValue::NULL)
                } else {
                    Some(JsValue::from_f64(mode as f64))
                }
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostSceneMaterialSetAlphaMode",
        dyn Fn(u32, u32, JsValue),
        |id: u32, rep: u32, value: JsValue| {
            with_script(id, |state| {
                let entry = state.mats.get(&rep)?;
                let inner = Arc::clone(&entry.inner);
                let doc = entry.doc_entity;
                let mode: Option<String> = value
                    .as_f64()
                    .map(|num| match num as i32 {
                        0 => "add",
                        1 => "blend",
                        2 => "mask",
                        3 => "multiply",
                        4 => "opaque",
                        5 => "premultiplied",
                        _ => "",
                    })
                    .filter(|mode| !mode.is_empty())
                    .map(String::from);
                core_ops::material::set_alpha_mode(&inner, doc, mode, &mut state.command_queue);
                Some(())
            });
        }
    );

    reg!(
        obj,
        "hostSceneMaterialSync",
        dyn Fn(u32, u32) -> bool,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                state
                    .mats
                    .get(&rep)
                    .map(|entry| entry.inner.sync.load(Ordering::Relaxed))
            })
            .flatten()
            .unwrap_or(false)
        }
    );

    reg!(
        obj,
        "hostSceneMaterialSetSync",
        dyn Fn(u32, u32, bool),
        |id: u32, rep: u32, value: bool| {
            with_script(id, |state| {
                state
                    .mats
                    .get(&rep)?
                    .inner
                    .sync
                    .store(value, Ordering::Relaxed);
                Some(())
            });
        }
    );
}
