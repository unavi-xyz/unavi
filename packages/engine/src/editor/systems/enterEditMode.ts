import { InputStruct } from "houseki/input";
import { PhysicsConfig } from "houseki/physics";
import { RenderView, Scene, SceneView } from "houseki/scene";
import { TransformControls } from "houseki/transform";
import { Commands, Entity, Mut, Query, Res, With, Without } from "thyseus";

export function enterEditMode(
  commands: Commands,
  inputStruct: Res<Mut<InputStruct>>,
  physicsConfig: Res<Mut<PhysicsConfig>>,
  scenes: Query<Entity, [With<Scene>, Without<TransformControls>]>,
  views: Query<SceneView, With<RenderView>>
) {
  for (const view of views) {
    for (const entity of scenes) {
      if (entity.id !== view.active) continue;
      commands.getById(entity.id).addType(TransformControls);
    }
  }

  physicsConfig.debug = true;
  inputStruct.enablePointerLock = false;
}
