import { Animation, AnimationChannel, Material, Mesh, Node, Primitive } from "@gltf-transform/core";

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

  // Dispose animations if not used by other nodes
  node
    .listParents()
    .filter((parent): parent is AnimationChannel => parent instanceof AnimationChannel)
    .forEach((channel) => {
      const isChannelUsed = channel.listParents().length > 1;

      const animations = channel
        .listParents()
        .filter((parent): parent is Animation => parent instanceof Animation);

      if (!isChannelUsed) {
        const sampler = channel.getSampler();

        if (sampler) {
          const isSamplerUsed =
            sampler.listParents().filter((parent) => parent instanceof AnimationChannel).length > 1;

          if (!isSamplerUsed) {
            const input = sampler.getInput();
            const output = sampler.getOutput();

            if (input) {
              const isInputUsed = input.listParents().length > 2; // Root node + sampler
              if (!isInputUsed) input.dispose();
            }

            if (output) {
              const isOutputUsed = output.listParents().length > 2; // Root node + sampler
              if (!isOutputUsed) output.dispose();
            }

            sampler.dispose();
          }
        }

        channel.dispose();
      }

      animations.forEach((animation) => {
        const isAnimationUsed = animation.listParents().length === 0;
        if (!isAnimationUsed) animation.dispose();
      });
    });

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
