const cabiDispose = Symbol.for("cabiDispose");
const cabiRep = Symbol.for("cabiRep");

function scriptId() {
  return globalThis.__unavi_current_script_id;
}

function host() {
  return globalThis.__unavi_host;
}

export class Node {
  static [cabiDispose](rep) {
    host()?.hostSceneNodeDrop?.(scriptId(), rep);
  }
  id() {
    return host().hostSceneNodeId(scriptId(), this[cabiRep]);
  }
  clone() {
    const rep = host().hostSceneNodeClone(scriptId(), this[cabiRep]);
    const node = Object.create(Node.prototype);
    node[cabiRep] = rep;
    return node;
  }
  name() {
    return host().hostSceneNodeName(scriptId(), this[cabiRep]);
  }
  setName(value) {
    host().hostSceneNodeSetName(scriptId(), this[cabiRep], value);
  }
  translation() {
    return host().hostSceneNodeTranslation(scriptId(), this[cabiRep]);
  }
  setTranslation(value) {
    host().hostSceneNodeSetTranslation(
      scriptId(),
      this[cabiRep],
      value.x,
      value.y,
      value.z,
    );
  }
  rotation() {
    return host().hostSceneNodeRotation(scriptId(), this[cabiRep]);
  }
  setRotation(value) {
    host().hostSceneNodeSetRotation(
      scriptId(),
      this[cabiRep],
      value.x,
      value.y,
      value.z,
      value.w,
    );
  }
  scale() {
    return host().hostSceneNodeScale(scriptId(), this[cabiRep]);
  }
  setScale(value) {
    host().hostSceneNodeSetScale(
      scriptId(),
      this[cabiRep],
      value.x,
      value.y,
      value.z,
    );
  }
  transform() {
    return host().hostSceneNodeTransform(scriptId(), this[cabiRep]);
  }
  setTransform(value) {
    host().hostSceneNodeSetTransform(
      scriptId(),
      this[cabiRep],
      new Float32Array([
        value.translation.x,
        value.translation.y,
        value.translation.z,
        value.rotation.x,
        value.rotation.y,
        value.rotation.z,
        value.rotation.w,
        value.scale.x,
        value.scale.y,
        value.scale.z,
      ]),
    );
  }
  globalTransform() {
    return host().hostSceneNodeGlobalTransform(scriptId(), this[cabiRep]);
  }
  parent() {
    const rep = host().hostSceneNodeParent(scriptId(), this[cabiRep]);
    if (!rep) return null;
    const node = Object.create(Node.prototype);
    node[cabiRep] = rep;
    return node;
  }
  children() {
    const reps = host().hostSceneNodeChildren(scriptId(), this[cabiRep]);
    return Array.from(reps).map((rep) => {
      const node = Object.create(Node.prototype);
      node[cabiRep] = rep;
      return node;
    });
  }
  addChild(child) {
    host().hostSceneNodeAddChild(scriptId(), this[cabiRep], child[cabiRep]);
  }
  removeChild(child) {
    host().hostSceneNodeRemoveChild(scriptId(), this[cabiRep], child[cabiRep]);
  }
  mesh() {
    const rep = host().hostSceneNodeMesh(scriptId(), this[cabiRep]);
    if (!rep) return null;
    const mesh = Object.create(Mesh.prototype);
    mesh[cabiRep] = rep;
    return mesh;
  }
  setMesh(value) {
    host().hostSceneNodeSetMesh(
      scriptId(),
      this[cabiRep],
      value ? value[cabiRep] : 0,
    );
  }
  material() {
    const rep = host().hostSceneNodeMaterial(scriptId(), this[cabiRep]);
    if (!rep) return null;
    const material = Object.create(Material.prototype);
    material[cabiRep] = rep;
    return material;
  }
  setMaterial(value) {
    host().hostSceneNodeSetMaterial(
      scriptId(),
      this[cabiRep],
      value ? value[cabiRep] : 0,
    );
  }
  collider() {
    return null;
  }
  setCollider(_value) {}
  rigidBody() {
    return null;
  }
  setRigidBody(_value) {}
  sync() {
    return host().hostSceneNodeSync(scriptId(), this[cabiRep]);
  }
  setSync(value) {
    host().hostSceneNodeSetSync(scriptId(), this[cabiRep], value);
  }
  drop() {
    Node[cabiDispose](this[cabiRep]);
  }
}

export class Document {
  static [cabiDispose](rep) {
    host()?.hostSceneDocumentDrop?.(scriptId(), rep);
  }
  id() {
    return host().hostSceneDocumentId(scriptId(), this[cabiRep]);
  }
  clone() {
    const rep = host().hostSceneDocumentClone(scriptId(), this[cabiRep]);
    const doc = Object.create(Document.prototype);
    doc[cabiRep] = rep;
    return doc;
  }
  createNode() {
    const rep = host().hostSceneDocumentCreateNode(scriptId(), this[cabiRep]);
    const node = Object.create(Node.prototype);
    node[cabiRep] = rep;
    return node;
  }
  createMesh() {
    const rep = host().hostSceneDocumentCreateMesh(scriptId(), this[cabiRep]);
    const mesh = Object.create(Mesh.prototype);
    mesh[cabiRep] = rep;
    return mesh;
  }
  createMaterial() {
    const rep = host().hostSceneDocumentCreateMaterial(
      scriptId(),
      this[cabiRep],
    );
    const material = Object.create(Material.prototype);
    material[cabiRep] = rep;
    return material;
  }
  roots() {
    const reps = host().hostSceneDocumentRoots(scriptId(), this[cabiRep]);
    return Array.from(reps).map((rep) => {
      const node = Object.create(Node.prototype);
      node[cabiRep] = rep;
      return node;
    });
  }
  nodes() {
    const reps = host().hostSceneDocumentNodes(scriptId(), this[cabiRep]);
    return Array.from(reps).map((rep) => {
      const node = Object.create(Node.prototype);
      node[cabiRep] = rep;
      return node;
    });
  }
  meshes() {
    const reps = host().hostSceneDocumentMeshes(scriptId(), this[cabiRep]);
    return Array.from(reps).map((rep) => {
      const mesh = Object.create(Mesh.prototype);
      mesh[cabiRep] = rep;
      return mesh;
    });
  }
  materials() {
    const reps = host().hostSceneDocumentMaterials(scriptId(), this[cabiRep]);
    return Array.from(reps).map((rep) => {
      const material = Object.create(Material.prototype);
      material[cabiRep] = rep;
      return material;
    });
  }
  removeNode(node) {
    host().hostSceneDocumentRemoveNode(
      scriptId(),
      this[cabiRep],
      node[cabiRep],
    );
  }
  removeMesh(mesh) {
    host().hostSceneDocumentRemoveMesh(
      scriptId(),
      this[cabiRep],
      mesh[cabiRep],
    );
  }
  removeMaterial(material) {
    host().hostSceneDocumentRemoveMaterial(
      scriptId(),
      this[cabiRep],
      material[cabiRep],
    );
  }
  sync() {
    return host().hostSceneDocumentSync(scriptId(), this[cabiRep]);
  }
  setSync(value) {
    host().hostSceneDocumentSetSync(scriptId(), this[cabiRep], value);
  }
  drop() {
    Document[cabiDispose](this[cabiRep]);
  }
}

export class Mesh {
  static [cabiDispose](rep) {
    host()?.hostSceneMeshDrop?.(scriptId(), rep);
  }
  id() {
    return host().hostSceneMeshId(scriptId(), this[cabiRep]);
  }
  clone() {
    const rep = host().hostSceneMeshClone(scriptId(), this[cabiRep]);
    const mesh = Object.create(Mesh.prototype);
    mesh[cabiRep] = rep;
    return mesh;
  }
  name() {
    return host().hostSceneMeshName(scriptId(), this[cabiRep]);
  }
  setName(value) {
    host().hostSceneMeshSetName(scriptId(), this[cabiRep], value);
  }
  topology() {
    return host().hostSceneMeshTopology(scriptId(), this[cabiRep]);
  }
  setTopology(value) {
    host().hostSceneMeshSetTopology(scriptId(), this[cabiRep], value);
  }
  indices() {
    return host().hostSceneMeshIndices(scriptId(), this[cabiRep]);
  }
  setIndices(value) {
    host().hostSceneMeshSetIndices(scriptId(), this[cabiRep], value);
  }
  positions() {
    return host().hostSceneMeshPositions(scriptId(), this[cabiRep]);
  }
  setPositions(value) {
    host().hostSceneMeshSetPositions(scriptId(), this[cabiRep], value);
  }
  normals() {
    return host().hostSceneMeshNormals(scriptId(), this[cabiRep]);
  }
  setNormals(value) {
    host().hostSceneMeshSetNormals(scriptId(), this[cabiRep], value);
  }
  tangents() {
    return host().hostSceneMeshTangents(scriptId(), this[cabiRep]);
  }
  setTangents(value) {
    host().hostSceneMeshSetTangents(scriptId(), this[cabiRep], value);
  }
  colors() {
    return host().hostSceneMeshColors(scriptId(), this[cabiRep]);
  }
  setColors(value) {
    host().hostSceneMeshSetColors(scriptId(), this[cabiRep], value);
  }
  uv0() {
    return host().hostSceneMeshUv0(scriptId(), this[cabiRep]);
  }
  setUv0(value) {
    host().hostSceneMeshSetUv0(scriptId(), this[cabiRep], value);
  }
  uv1() {
    return host().hostSceneMeshUv1(scriptId(), this[cabiRep]);
  }
  setUv1(value) {
    host().hostSceneMeshSetUv1(scriptId(), this[cabiRep], value);
  }
  sync() {
    return host().hostSceneMeshSync(scriptId(), this[cabiRep]);
  }
  setSync(value) {
    host().hostSceneMeshSetSync(scriptId(), this[cabiRep], value);
  }
  drop() {
    Mesh[cabiDispose](this[cabiRep]);
  }
}

export class Material {
  static [cabiDispose](rep) {
    host()?.hostSceneMaterialDrop?.(scriptId(), rep);
  }
  id() {
    return host().hostSceneMaterialId(scriptId(), this[cabiRep]);
  }
  clone() {
    const rep = host().hostSceneMaterialClone(scriptId(), this[cabiRep]);
    const material = Object.create(Material.prototype);
    material[cabiRep] = rep;
    return material;
  }
  name() {
    return host().hostSceneMaterialName(scriptId(), this[cabiRep]);
  }
  setName(value) {
    host().hostSceneMaterialSetName(scriptId(), this[cabiRep], value);
  }
  alphaCutoff() {
    return host().hostSceneMaterialAlphaCutoff(scriptId(), this[cabiRep]);
  }
  setAlphaCutoff(value) {
    host().hostSceneMaterialSetAlphaCutoff(scriptId(), this[cabiRep], value);
  }
  alphaMode() {
    return host().hostSceneMaterialAlphaMode(scriptId(), this[cabiRep]);
  }
  setAlphaMode(value) {
    host().hostSceneMaterialSetAlphaMode(scriptId(), this[cabiRep], value);
  }
  baseColor() {
    return host().hostSceneMaterialBaseColor(scriptId(), this[cabiRep]);
  }
  setBaseColor(value) {
    host().hostSceneMaterialSetBaseColor(
      scriptId(),
      this[cabiRep],
      value.r,
      value.g,
      value.b,
      value.a,
    );
  }
  metallic() {
    return host().hostSceneMaterialMetallic(scriptId(), this[cabiRep]);
  }
  setMetallic(value) {
    host().hostSceneMaterialSetMetallic(scriptId(), this[cabiRep], value);
  }
  roughness() {
    return host().hostSceneMaterialRoughness(scriptId(), this[cabiRep]);
  }
  setRoughness(value) {
    host().hostSceneMaterialSetRoughness(scriptId(), this[cabiRep], value);
  }
  doubleSided() {
    return host().hostSceneMaterialDoubleSided(scriptId(), this[cabiRep]);
  }
  setDoubleSided(value) {
    host().hostSceneMaterialSetDoubleSided(scriptId(), this[cabiRep], value);
  }
  unlit() {
    return host().hostSceneMaterialUnlit(scriptId(), this[cabiRep]);
  }
  setUnlit(value) {
    host().hostSceneMaterialSetUnlit(scriptId(), this[cabiRep], value);
  }
  sync() {
    return host().hostSceneMaterialSync(scriptId(), this[cabiRep]);
  }
  setSync(value) {
    host().hostSceneMaterialSetSync(scriptId(), this[cabiRep], value);
  }
  drop() {
    Material[cabiDispose](this[cabiRep]);
  }
}

export function Vec3_zero() {
  return { x: 0, y: 0, z: 0 };
}
export function Vec3_one() {
  return { x: 1, y: 1, z: 1 };
}
export function Quat_default() {
  return { x: 0, y: 0, z: 0, w: 1 };
}
