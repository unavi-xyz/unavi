import { InputStruct } from "houseki/input";
import { PhysicsConfig } from "houseki/physics";
import { Scene, SceneStruct } from "houseki/scene";
import { TransformControls } from "houseki/transform";
import { Commands, Entity, Mut, Query, Res, With } from "thyseus";

export function enterEditMode(
  commands: Commands,
  sceneStruct: Res<SceneStruct>,
  inputStruct: Res<Mut<InputStruct>>,
  physicsConfig: Res<Mut<PhysicsConfig>>,
  scenes: Query<Entity, With<Scene>>
) {
  for (const entity of scenes) {
    if (entity.id !== sceneStruct.activeScene) continue;
    commands.getById(entity.id).addType(TransformControls);
  }

  physicsConfig.debug = true;
  inputStruct.enablePointerLock = false;
}
