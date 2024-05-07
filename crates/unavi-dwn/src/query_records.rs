use bevy::prelude::*;
use bevy_async_task::{AsyncTaskRunner, AsyncTaskStatus};
use dwn::{
    actor::ProcessMessageError, message::descriptor::records::RecordsFilter, reply::QueryReply,
};

use crate::UserActor;

#[derive(Component)]
pub struct QueryRecords {
    pub filter: RecordsFilter,
    pub target: Option<String>,
}

#[derive(Component)]
pub struct QueryRecordsResult(pub QueryReply);

pub fn handle_query_records(
    actor: Res<UserActor>,
    querys: Query<(Entity, &QueryRecords)>,
    mut commands: Commands,
    mut processing: Local<Option<Entity>>,
    mut task: AsyncTaskRunner<Result<QueryReply, ProcessMessageError>>,
) {
    match task.poll() {
        AsyncTaskStatus::Idle => {
            if let Some((entity, query)) = querys.iter().next() {
                let actor = actor.0.clone();

                let filter = query.filter.clone();
                let target = query.target.clone();

                task.start(async move {
                    let mut msg = actor.query_records(filter);

                    if let Some(target) = target {
                        msg.send(&target).await
                    } else {
                        msg.process().await
                    }
                });

                *processing = Some(entity);
            }
        }
        AsyncTaskStatus::Pending => {}
        AsyncTaskStatus::Finished(res) => {
            match res {
                Ok(reply) => {
                    commands
                        .entity(processing.unwrap())
                        .insert(QueryRecordsResult(reply));
                }
                Err(err) => {
                    error!("Failed to query record: {}", err);
                }
            };

            *processing = None;
        }
    };
}
