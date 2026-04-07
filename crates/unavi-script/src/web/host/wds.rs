use std::sync::mpsc;

use js_sys::Object;
use wasm_bindgen::JsValue;
use wired_records::{BeaconRecord, RecordValue};

use super::state::{WdsQueryFuture, WdsReadFuture, WdsRecordOut};
use super::with_script;

pub fn register(obj: &Object) {
    reg!(obj, "hostWdsGetWds", dyn Fn(u32) -> u32, |id: u32| {
        with_script(id, |state| {
            let actor = state.wds_actor.clone()?;
            let rep = state.alloc();
            state.wds_instances.insert(rep, actor);
            Some(rep)
        })
        .flatten()
        .unwrap_or(0)
    });

    reg!(
        obj,
        "hostWdsQuery",
        dyn Fn(u32, u32, JsValue) -> u32,
        |id: u32, wds_rep: u32, filter: JsValue| {
            with_script(id, |state| {
                let actor = state.wds_instances.get(&wds_rep)?.clone();
                let (tx, rx) = mpsc::channel();

                let mut builder = actor.query();

                if !filter.is_null() && !filter.is_undefined() {
                    if let Ok(creator) = js_sys::Reflect::get(&filter, &"creator".into()) {
                        if let Some(did) = creator.as_string() {
                            if let Ok(parsed) = did.parse() {
                                builder = builder.creator(&parsed);
                            }
                        }
                    }
                    if let Ok(schemas) = js_sys::Reflect::get(&filter, &"schemas".into()) {
                        if let Some(arr) = schemas.dyn_ref::<js_sys::Array>() {
                            for i in 0..arr.length() {
                                let item = arr.get(i);
                                if let Some(bytes) =
                                    item.dyn_ref::<js_sys::Uint8Array>().map(|a| a.to_vec())
                                {
                                    if let Ok(arr) = <[u8; 32]>::try_from(bytes.as_slice()) {
                                        builder = builder.schema(blake3::Hash::from_bytes(arr));
                                    }
                                }
                            }
                        }
                    }
                }

                unavi_wasm_compat::spawn_thread(async move {
                    let result = builder.send().await;
                    let _ = tx.send(result);
                });

                let rep = state.alloc();
                state.wds_query_futures.insert(rep, WdsQueryFuture { rx });
                Some(rep)
            })
            .flatten()
            .unwrap_or(0)
        }
    );

    reg!(
        obj,
        "hostWdsRead",
        dyn Fn(u32, u32, JsValue) -> u32,
        |id: u32, wds_rep: u32, record_id: JsValue| {
            with_script(id, |state| {
                let actor = state.wds_instances.get(&wds_rep)?.clone();
                let bytes = record_id.dyn_ref::<js_sys::Uint8Array>()?.to_vec();
                let hash_bytes: [u8; 32] = bytes.as_slice().try_into().ok()?;
                let hash = blake3::Hash::from_bytes(hash_bytes);

                let (tx, rx) = mpsc::channel();

                unavi_wasm_compat::spawn_thread(async move {
                    let result = read_record(&actor, hash).await;
                    let _ = tx.send(result);
                });

                let rep = state.alloc();
                state.wds_read_futures.insert(rep, WdsReadFuture { rx });
                Some(rep)
            })
            .flatten()
            .unwrap_or(0)
        }
    );

    reg!(
        obj,
        "hostWdsQueryFuturePoll",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let future = state.wds_query_futures.get(&rep)?;
                match future.rx.try_recv() {
                    Ok(Ok(hashes)) => {
                        let arr = js_sys::Array::new();
                        for hash in hashes {
                            let bytes = js_sys::Uint8Array::from(hash.as_bytes().as_slice());
                            arr.push(&bytes);
                        }
                        let result = Object::new();
                        js_sys::Reflect::set(&result, &"tag".into(), &"ok".into()).ok();
                        js_sys::Reflect::set(&result, &"val".into(), &arr).ok();
                        Some(JsValue::from(result))
                    }
                    Ok(Err(_)) | Err(mpsc::TryRecvError::Disconnected) => {
                        let result = Object::new();
                        js_sys::Reflect::set(&result, &"tag".into(), &"err".into()).ok();
                        Some(JsValue::from(result))
                    }
                    Err(mpsc::TryRecvError::Empty) => None,
                }
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(
        obj,
        "hostWdsReadFuturePoll",
        dyn Fn(u32, u32) -> JsValue,
        |id: u32, rep: u32| {
            with_script(id, |state| {
                let future = state.wds_read_futures.get(&rep)?;
                match future.rx.try_recv() {
                    Ok(Ok(record)) => {
                        let obj = Object::new();
                        js_sys::Reflect::set(&obj, &"tag".into(), &"ok".into()).ok();

                        let val = Object::new();
                        let id_bytes = js_sys::Uint8Array::from(record.id.as_bytes().as_slice());
                        js_sys::Reflect::set(&val, &"id".into(), &id_bytes).ok();
                        js_sys::Reflect::set(
                            &val,
                            &"creator".into(),
                            &JsValue::from_str(&record.creator),
                        )
                        .ok();

                        let schemas = js_sys::Array::new();
                        for hash in &record.schemas {
                            schemas.push(&js_sys::Uint8Array::from(hash.as_bytes().as_slice()));
                        }
                        js_sys::Reflect::set(&val, &"schemas".into(), &schemas).ok();

                        let containers = js_sys::Array::new();
                        for (name, bytes) in &record.containers {
                            let tuple = js_sys::Array::new();
                            tuple.push(&JsValue::from_str(name));
                            tuple.push(&js_sys::Uint8Array::from(bytes.as_slice()));
                            containers.push(&tuple);
                        }
                        js_sys::Reflect::set(&val, &"containers".into(), &containers).ok();

                        js_sys::Reflect::set(&obj, &"val".into(), &JsValue::from(val)).ok();

                        Some(JsValue::from(obj))
                    }
                    Ok(Err(_)) | Err(mpsc::TryRecvError::Disconnected) => {
                        let result = Object::new();
                        js_sys::Reflect::set(&result, &"tag".into(), &"err".into()).ok();
                        Some(JsValue::from(result))
                    }
                    Err(mpsc::TryRecvError::Empty) => None,
                }
            })
            .flatten()
            .unwrap_or(JsValue::NULL)
        }
    );

    reg!(obj, "hostWdsDrop", dyn Fn(u32, u32), |id: u32, rep: u32| {
        with_script(id, |state| {
            state.wds_instances.remove(&rep);
        });
    });

    reg!(
        obj,
        "hostWdsQueryFutureDrop",
        dyn Fn(u32, u32),
        |id: u32, rep: u32| {
            with_script(id, |state| {
                state.wds_query_futures.remove(&rep);
            });
        }
    );

    reg!(
        obj,
        "hostWdsReadFutureDrop",
        dyn Fn(u32, u32),
        |id: u32, rep: u32| {
            with_script(id, |state| {
                state.wds_read_futures.remove(&rep);
            });
        }
    );
}

async fn read_record(
    actor: &wds::actor::Actor,
    hash: blake3::Hash,
) -> anyhow::Result<WdsRecordOut> {
    let doc = actor.read(hash).send().await?;

    let record = wired_schemas::surg::record::Record::load(&doc)?;
    let creator = record.creator.0.to_string();
    let id = record.id()?;

    let mut schemas = Vec::new();
    let mut containers = Vec::new();

    for (container, schema_hash) in &record.schemas {
        if container == "acl" || container == "record" {
            continue;
        }
        schemas.push(schema_hash.0);

        let bytes = if container == "beacon" {
            postcard::to_stdvec(&BeaconRecord::load(&doc)?)?
        } else {
            let value = RecordValue::from(doc.get_map(container.as_str()).get_deep_value());
            postcard::to_stdvec(&value)?
        };
        containers.push((container.to_string(), bytes));
    }

    Ok(WdsRecordOut {
        id,
        creator,
        schemas,
        containers,
    })
}
