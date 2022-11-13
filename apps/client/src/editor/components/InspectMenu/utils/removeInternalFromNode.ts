import { useEditorStore } from "../../../store";

export function removeInternalFromNode(nodeId: string) {
  const { engine } = useEditorStore.getState();
  if (!engine) throw new Error("Engine not found");

  const node = engine.scene.nodes[nodeId];
  if (!node) throw new Error("Node not found");

  node.isInternal = false;

  // Mesh
  if (node.meshId) {
    const mesh = engine.scene.meshes[node.meshId];
    if (mesh) {
      mesh.isInternal = false;

      // Material
      if (mesh.materialId) removeInternalFromMaterial(mesh.materialId);

      // Primitives
      if (mesh.type === "Primitives") {
        mesh.primitives.forEach((primitive) => {
          primitive.isInternal = false;

          // Material
          if (primitive.materialId)
            removeInternalFromMaterial(primitive.materialId);

          // Accessors
          if (primitive.COLOR_0) removeInternalFromAccessor(primitive.COLOR_0);
          if (primitive.JOINTS_0)
            removeInternalFromAccessor(primitive.JOINTS_0);
          if (primitive.NORMAL) removeInternalFromAccessor(primitive.NORMAL);
          if (primitive.POSITION)
            removeInternalFromAccessor(primitive.POSITION);
          if (primitive.TANGENT) removeInternalFromAccessor(primitive.TANGENT);
          if (primitive.TEXCOORD_0)
            removeInternalFromAccessor(primitive.TEXCOORD_0);
          if (primitive.TEXCOORD_1)
            removeInternalFromAccessor(primitive.TEXCOORD_1);
          if (primitive.WEIGHTS_0)
            removeInternalFromAccessor(primitive.WEIGHTS_0);
        });
      }
    }
  }

  // Animations
  Object.values(engine.scene.animations).forEach((animation) => {
    animation.channels.forEach((channel) => {
      if (channel.targetId === nodeId) animation.isInternal = false;
    });
  });

  // Repeat for children
  node.childrenIds.forEach((childId) => removeInternalFromNode(childId));
}

export function removeInternalFromMaterial(materialId: string) {
  const { engine } = useEditorStore.getState();
  if (!engine) throw new Error("Engine not found");

  const material = engine.scene.materials[materialId];
  if (!material) throw new Error("Material not found");

  material.isInternal = false;

  // Images
  if (material.colorTexture?.imageId) {
    const image = engine.scene.images[material.colorTexture.imageId];
    if (image) image.isInternal = false;
  }

  if (material.emissiveTexture?.imageId) {
    const image = engine.scene.images[material.emissiveTexture.imageId];
    if (image) image.isInternal = false;
  }

  if (material.normalTexture?.imageId) {
    const image = engine.scene.images[material.normalTexture.imageId];
    if (image) image.isInternal = false;
  }

  if (material.metallicRoughnessTexture?.imageId) {
    const image =
      engine.scene.images[material.metallicRoughnessTexture.imageId];
    if (image) image.isInternal = false;
  }

  if (material.occlusionTexture?.imageId) {
    const image = engine.scene.images[material.occlusionTexture.imageId];
    if (image) image.isInternal = false;
  }
}

export function removeInternalFromAccessor(accessorId: string) {
  const { engine } = useEditorStore.getState();
  if (!engine) throw new Error("Engine not found");

  const accessor = engine.scene.accessors[accessorId];
  if (!accessor) throw new Error("Accessor not found");

  accessor.isInternal = false;
}
