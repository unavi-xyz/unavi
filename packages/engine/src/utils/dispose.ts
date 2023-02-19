import { Material, Mesh, Node, Primitive } from "@gltf-transform/core";

export function deepDisposeNode(node: Node) {
  const children = node.listChildren();
  children.forEach((child) => deepDisposeNode(child));

  disposeNode(node);
}

export function disposeNode(node: Node) {
  // Dispose mesh if not used by other nodes
  const mesh = node.getMesh();
  if (mesh) {
    const isMeshUsed = mesh.listParents().filter((parent) => parent instanceof Node).length > 1;
    if (!isMeshUsed) disposeMesh(mesh);
  }

  // Dispose skin if not used by other nodes
  const skin = node.getSkin();
  if (skin) {
    const isSkinUsed = skin.listParents().filter((parent) => parent instanceof Node).length > 1;
    if (!isSkinUsed) skin.dispose();
  }

  node.dispose();
}

export function disposeMesh(mesh: Mesh) {
  // Dispose primitives
  const primitives = mesh.listPrimitives();
  primitives.forEach((primitive) => disposePrimitive(primitive));

  mesh.dispose();
}

export function disposePrimitive(primitive: Primitive) {
  // Dispose material if not used by other primitives
  const material = primitive.getMaterial();
  if (material) {
    const isMaterialUsed =
      material.listParents().filter((parent) => parent instanceof Primitive).length > 1;
    if (!isMaterialUsed) disposeMaterial(material);
  }

  // Dispose morph targets
  const morphTargets = primitive.listTargets();
  morphTargets.forEach((morphTarget) => morphTarget.dispose());

  // Dispose accessors if not in use
  const indices = primitive.getIndices();
  if (indices) {
    const isIndicesUsed =
      indices.listParents().filter((parent) => parent instanceof Primitive).length > 1;
    if (!isIndicesUsed) indices.dispose();
  }

  const attributes = primitive.listAttributes();
  attributes.forEach((attribute) => {
    const isAttributeUsed =
      attribute.listParents().filter((parent) => parent instanceof Primitive).length > 1;
    if (!isAttributeUsed) attribute.dispose();
  });

  primitive.dispose();
}

export function disposeMaterial(material: Material) {
  // Dispose textures if not used by other materials
  const textures = [
    material.getBaseColorTexture(),
    material.getMetallicRoughnessTexture(),
    material.getNormalTexture(),
    material.getOcclusionTexture(),
    material.getEmissiveTexture(),
  ];

  textures.forEach((texture) => {
    if (!texture) return;
    const isTextureUsed =
      texture?.listParents().filter((parent) => parent instanceof Material).length > 1;
    if (!isTextureUsed) texture.dispose();
  });

  // Dispose texture infos
  const textureInfos = [
    material.getBaseColorTextureInfo(),
    material.getMetallicRoughnessTextureInfo(),
    material.getNormalTextureInfo(),
    material.getOcclusionTextureInfo(),
    material.getEmissiveTextureInfo(),
  ];

  textureInfos.forEach((textureInfo) => {
    if (!textureInfo) return;
    textureInfo.dispose();
  });

  material.dispose();
}
