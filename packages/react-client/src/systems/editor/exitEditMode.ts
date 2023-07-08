import { InputStruct } from "lattice-engine/input";
import { TransformControls } from "lattice-engine/transform";
import { Commands, Entity, Mut, Query, Res, With } from "thyseus";

export function exitEditMode(
  commands: Commands,
  inputStruct: Res<Mut<InputStruct>>,
  controls: Query<Entity, With<TransformControls>>
) {
  for (const entity of controls) {
    commands.getById(entity.id).remove(TransformControls);
  }

  inputStruct.enablePointerLock = true;
}
