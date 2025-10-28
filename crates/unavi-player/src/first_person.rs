use bevy::prelude::*;
use bevy_vrm::{VrmInstance, first_person::SetupFirstPerson, loader::Vrm};

#[derive(Component)]
pub struct FirstPerson;

pub(crate) fn setup_first_person(
    avatars: Query<(Entity, &VrmInstance), With<FirstPerson>>,
    mut events: EventReader<AssetEvent<Vrm>>,
    mut writer: EventWriter<SetupFirstPerson>,
) {
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            let (ent, _) = avatars
                .iter()
                .find(|(_, handle)| handle.0.id() == *id)
                .expect("Avatar not found");

            writer.write(SetupFirstPerson(ent));
        }
    }
}
