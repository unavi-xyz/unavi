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
      removeInternalFromMaterial(mesh.materialId);

      // Primitives
      if (mesh.type === "Primitives") {
        mesh.primitives.forEach((primitive) => {
          primitive.isInternal = false;

          // Material
          removeInternalFromMaterial(primitive.materialId);

          // Accessors
          removeInternalFromAccessor(primitive.COLOR_0);
          removeInternalFromAccessor(primitive.JOINTS_0);
          removeInternalFromAccessor(primitive.NORMAL);
          removeInternalFromAccessor(primitive.POSITION);
          removeInternalFromAccessor(primitive.TANGENT);
          removeInternalFromAccessor(primitive.TEXCOORD_0);
          removeInternalFromAccessor(primitive.TEXCOORD_1);
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

export function removeInternalFromMaterial(materialId: string | null) {
  if (!materialId) return;

  const { engine } = useEditorStore.getState();
  if (!engine) throw new Error("Engine not found");

  const material = engine.scene.materials[materialId];
  if (!material) throw new Error("Material not found");

  material.isInternal = false;

  // Images
  removeInternalFromImage(material.colorTexture?.imageId);
  removeInternalFromImage(material.emissiveTexture?.imageId);
  removeInternalFromImage(material.metallicRoughnessTexture?.imageId);
  removeInternalFromImage(material.normalTexture?.imageId);
  removeInternalFromImage(material.occlusionTexture?.imageId);
}

export function removeInternalFromImage(imageId: string | undefined | null) {
  if (!imageId) return;

  const { engine } = useEditorStore.getState();
  if (!engine) throw new Error("Engine not found");

  const image = engine.scene.images[imageId];
  if (!image) throw new Error("Image not found");

  image.isInternal = false;
}

export function removeInternalFromAccessor(accessorId: string | null) {
  if (!accessorId) return;

  const { engine } = useEditorStore.getState();
  if (!engine) throw new Error("Engine not found");

  const accessor = engine.scene.accessors[accessorId];
  if (!accessor) throw new Error("Accessor not found");

  accessor.isInternal = false;
}
