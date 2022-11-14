import { BehaviorSubject } from "rxjs";

import { Accessor } from "./Accessor";
import { Animation } from "./Animation";
import { Image } from "./Image";
import { Material } from "./Material";
import { BoxMesh } from "./mesh/BoxMesh";
import { CylinderMesh } from "./mesh/CylinderMesh";
import { GLTFMesh } from "./mesh/GLTFMesh";
import { Primitive } from "./mesh/Primitive";
import { PrimitivesMesh } from "./mesh/PrimitivesMesh";
import { SphereMesh } from "./mesh/SphereMesh";
import { Mesh, MeshJSON } from "./mesh/types";
import { Node } from "./Node";
import { MaterialJSON, NodeJSON, SceneJSON } from "./types";
import { sortNodes } from "./utils/sortNodes";

/*
 * Stores the scene in a custom internal format.
 * State is stored using RxJS, allowing for subscriptions to state changes.
 * This is especially useful for the editor's React UI.
 */
export class Scene {
  nodes$ = new BehaviorSubject<{ [id: string]: Node }>({
    root: new Node({ id: "root" }),
  });
  meshes$ = new BehaviorSubject<{ [id: string]: Mesh }>({});
  materials$ = new BehaviorSubject<{ [id: string]: Material }>({});
  accessors$ = new BehaviorSubject<{ [id: string]: Accessor }>({});
  images$ = new BehaviorSubject<{ [id: string]: Image }>({});
  animations$ = new BehaviorSubject<{ [id: string]: Animation }>({});

  spawnId$ = new BehaviorSubject<string | null>(null);

  get spawnId() {
    return this.spawnId$.value;
  }

  set spawnId(spawnId: string | null) {
    this.spawnId$.next(spawnId);
  }

  get nodes() {
    return this.nodes$.value;
  }

  set nodes(nodes: { [id: string]: Node }) {
    this.nodes$.next(nodes);
  }

  get meshes() {
    return this.meshes$.value;
  }

  set meshes(meshes: { [id: string]: Mesh }) {
    this.meshes$.next(meshes);
  }

  get materials() {
    return this.materials$.value;
  }

  set materials(materials: { [id: string]: Material }) {
    this.materials$.next(materials);
  }

  get accessors() {
    return this.accessors$.value;
  }

  set accessors(accessors: { [id: string]: Accessor }) {
    this.accessors$.next(accessors);
  }

  get images() {
    return this.images$.value;
  }

  set images(images: { [id: string]: Image }) {
    this.images$.next(images);
  }

  get animations() {
    return this.animations$.value;
  }

  set animations(animations: { [id: string]: Animation }) {
    this.animations$.next(animations);
  }

  addAccessor(accessor: Accessor) {
    const previous = this.accessors[accessor.id];
    if (previous) this.removeAccessor(previous.id);

    // Save to accessors
    this.accessors = {
      ...this.accessors,
      [accessor.id]: accessor,
    };
  }

  removeAccessor(accessorId: string) {
    const accessors = { ...this.accessors };
    delete accessors[accessorId];
    this.accessors = accessors;
  }

  addNode(node: Node) {
    if (node.id === "root") return;

    const previous = this.nodes[node.id];
    if (previous) this.removeNode(previous.id);

    // Set scene
    node.scene = this;

    // Add to parent
    const parent = node.parent;
    if (parent)
      parent.childrenIds$.next([...parent.childrenIds$.value, node.id]);

    // Save to nodes
    this.nodes = { ...this.nodes, [node.id]: node };
  }

  updateNode(nodeId: string, data: Partial<NodeJSON>) {
    if (nodeId === "root") return;

    const node = this.nodes[nodeId];
    if (!node) throw new Error(`Node ${nodeId} not found`);

    node.applyJSON(data);
  }

  removeNode(nodeId: string) {
    if (nodeId === "root") return;

    // If node is a spawn point, remove it
    if (this.spawnId === nodeId) this.spawnId = null;

    const node = this.nodes[nodeId];
    if (!node) throw new Error(`Node ${nodeId} not found`);

    // Repeat for children
    node.childrenIds.forEach((childId) => this.removeNode(childId));

    // Remove from nodes
    this.nodes = Object.fromEntries(
      Object.entries(this.nodes).filter(([id]) => id !== nodeId)
    );

    // Remove from parent
    if (node.parent) node.parentId = "";

    // Remove mesh if no other nodes use it
    if (node.meshId) {
      const mesh = this.meshes[node.meshId];
      if (!mesh) throw new Error(`Mesh ${node.meshId} not found`);

      const isUsed = Object.values(this.nodes).some(
        (n) => n.meshId === node.meshId
      );

      if (!isUsed) this.removeMesh(node.meshId);
    }

    // Remove animations
    Object.values(this.animations).forEach((animation) => {
      // Remove animation if it doesn't have any other node using it
      const targetIds = animation.channels.map((channel) => channel.targetId);
      const isUsed = targetIds.some((targetId) => {
        const targetNode = this.nodes[targetId];
        if (!targetNode) return false;
        return true;
      });

      if (!isUsed) this.removeAnimation(animation.id);
    });

    // Destroy node
    node.destroy();
  }

  addMesh(mesh: Mesh) {
    const previous = this.meshes[mesh.id];
    if (previous) this.removeMesh(previous.id);

    // Save to nodes
    this.meshes = {
      ...this.meshes,
      [mesh.id]: mesh,
    };
  }

  updateMesh(meshId: string, data: Partial<MeshJSON>) {
    const mesh = this.meshes[meshId];
    if (!mesh) throw new Error(`Mesh ${meshId} not found`);

    mesh.applyJSON(data as any);
  }

  removeMesh(meshId: string) {
    const mesh = this.meshes[meshId];
    if (!mesh) throw new Error(`Mesh ${meshId} not found`);

    // Remove from meshes
    this.meshes = Object.fromEntries(
      Object.entries(this.meshes).filter(([id]) => id !== meshId)
    );

    // Remove from nodes
    Object.values(this.nodes).forEach((node) => {
      if (node.meshId === meshId) node.meshId = null;
    });

    const materialMeshes: (Mesh | Primitive)[] = [];

    if (mesh.type === "Primitives") {
      // Remove primitive accessors
      mesh.primitives.forEach((primitive) => {
        if (primitive.indicesId) this.removeAccessor(primitive.indicesId);
        if (primitive.POSITION) this.removeAccessor(primitive.POSITION);
        if (primitive.NORMAL) this.removeAccessor(primitive.NORMAL);
        if (primitive.TEXCOORD_0) this.removeAccessor(primitive.TEXCOORD_0);
        if (primitive.TANGENT) this.removeAccessor(primitive.TANGENT);
        if (primitive.WEIGHTS_0) this.removeAccessor(primitive.WEIGHTS_0);
        if (primitive.JOINTS_0) this.removeAccessor(primitive.JOINTS_0);

        primitive.morphPositionIds.forEach((id) => this.removeAccessor(id));
        primitive.morphNormalIds.forEach((id) => this.removeAccessor(id));
        primitive.morphTangentIds.forEach((id) => this.removeAccessor(id));

        materialMeshes.push(primitive);
      });
    } else {
      materialMeshes.push(mesh);
    }

    // Remove materials
    materialMeshes.forEach((mesh) => {
      if (mesh.materialId) {
        // Remove material
        if (mesh.materialId) {
          const material = this.materials[mesh.materialId];
          if (!material) return;

          // Only remove internal materials
          if (material.isInternal) {
            // Only remove material if it's not used by any other mesh
            const isUsed = Object.values(this.nodes).some((node) => {
              if (!node.meshId) return false;

              const mesh = this.meshes[node.meshId];
              if (!mesh) return false;

              if (mesh.type === "Primitives")
                return mesh.primitives.some(
                  (primitive) => primitive.materialId === material.id
                );

              return mesh.materialId === material.id;
            });

            if (!isUsed) this.removeMaterial(mesh.materialId);
          }
        }
      }
    });

    // Destroy mesh
    mesh.destroy();
  }

  addMaterial(material: Material) {
    const previous = this.materials[material.id];
    if (previous) this.removeMaterial(previous.id);

    // Save to materials
    this.materials = {
      ...this.materials,
      [material.id]: material,
    };
  }

  removeMaterial(materialId: string) {
    const material = this.materials[materialId];
    if (!material) throw new Error(`Material ${materialId} not found`);

    // Remove from all meshes
    this.meshes = Object.fromEntries(
      Object.entries(this.meshes).map(([id, mesh]) => {
        if (mesh.materialId === materialId) mesh.materialId = null;
        if (mesh.type === "Primitives") {
          mesh.primitives.forEach((primitive) => {
            if (primitive.materialId === materialId)
              primitive.materialId = null;
          });
        }
        return [id, mesh];
      })
    );

    // Remove texture images
    [
      material.colorTexture,
      material.normalTexture,
      material.emissiveTexture,
      material.occlusionTexture,
      material.metallicRoughnessTexture,
    ].forEach((texture) => {
      if (!texture) return;

      const imageId = texture.imageId;
      if (!imageId) return;

      const image = this.images[imageId];
      if (!image) return;

      // Only remove image if it's not used by any other material
      const otherMaterial = Object.values(this.materials).find((m) => {
        m.id !== material.id &&
          [
            m.colorTexture,
            m.normalTexture,
            m.occlusionTexture,
            m.emissiveTexture,
            m.metallicRoughnessTexture,
          ].some((t) => t?.imageId === imageId);
      });

      if (!otherMaterial) this.removeImage(imageId);
    });

    // Destroy material
    material.destroy();

    // Remove from materials
    this.materials = Object.fromEntries(
      Object.entries(this.materials).filter(([id]) => id !== materialId)
    );
  }

  updateMaterial(materialId: string, data: Partial<MaterialJSON>) {
    const material = this.materials[materialId];
    if (!material) throw new Error(`Material ${materialId} not found`);

    material.applyJSON(data);
  }

  addImage(image: Image) {
    const previous = this.images[image.id];
    if (previous) this.removeImage(previous.id);

    // Save to images
    this.images = {
      ...this.images,
      [image.id]: image,
    };
  }

  removeImage(imageId: string) {
    const image = this.images[imageId];
    if (!image) throw new Error(`Image ${imageId} not found`);

    // Remove from images
    this.images = Object.fromEntries(
      Object.entries(this.images).filter(([id]) => id !== imageId)
    );
  }

  addAnimation(animation: Animation) {
    const previous = this.animations[animation.id];
    if (previous) this.removeAnimation(previous.id);

    // Save to animations
    this.animations = {
      ...this.animations,
      [animation.id]: animation,
    };
  }

  removeAnimation(animationId: string) {
    const animation = this.animations[animationId];
    if (!animation) throw new Error(`Animation ${animationId} not found`);

    // Remove from animations
    this.animations = Object.fromEntries(
      Object.entries(this.animations).filter(([id]) => id !== animationId)
    );

    // Remove sampler accessors
    animation.channels.forEach((channel) => {
      this.removeAccessor(channel.sampler.inputId);
      this.removeAccessor(channel.sampler.outputId);
    });
  }

  toJSON(includeInternal = false): SceneJSON {
    return {
      spawnId: this.spawnId,

      meshes: Object.values(this.meshes)
        .filter((m) => (m.isInternal ? includeInternal : true))
        .map((m) => m.toJSON()),

      nodes: Object.values(this.nodes)
        .filter((e) => (e.isInternal ? includeInternal : true))
        .map((e) => e.toJSON()),

      materials: Object.values(this.materials)
        .filter((m) => (m.isInternal ? includeInternal : true))
        .map((m) => m.toJSON()),

      accessors: Object.values(this.accessors)
        .filter((a) => (a.isInternal ? includeInternal : true))
        .map((a) => a.toJSON()),

      images: Object.values(this.images)
        .filter((i) => (i.isInternal ? includeInternal : true))
        .map((i) => i.toJSON()),

      animations: Object.values(this.animations)
        .filter((a) => (a.isInternal ? includeInternal : true))
        .map((a) => a.toJSON()),
    };
  }

  loadJSON(json: Partial<SceneJSON>) {
    if (json.spawnId !== undefined) this.spawnId = json.spawnId;

    // Add accessors
    if (json.accessors) {
      json.accessors.forEach((accessor) =>
        this.addAccessor(Accessor.fromJSON(accessor))
      );
    }

    // Add images
    if (json.images) {
      json.images.forEach((image) => this.addImage(Image.fromJSON(image)));
    }

    // Add materials
    if (json.materials) {
      json.materials.forEach((material) =>
        this.addMaterial(Material.fromJSON(material))
      );
    }

    // Add meshes
    if (json.meshes) {
      json.meshes.forEach((mesh) => {
        switch (mesh.type) {
          case "Box": {
            this.addMesh(BoxMesh.fromJSON(mesh));
            break;
          }

          case "Sphere": {
            this.addMesh(SphereMesh.fromJSON(mesh));
            break;
          }

          case "Cylinder": {
            this.addMesh(CylinderMesh.fromJSON(mesh));
            break;
          }

          case "Primitives": {
            this.addMesh(PrimitivesMesh.fromJSON(mesh));
            break;
          }

          case "glTF": {
            this.addMesh(GLTFMesh.fromJSON(mesh));
          }
        }
      });
    }

    // Sort nodes
    if (json.nodes) {
      const sortedNodes = sortNodes(json);

      // Add nodes
      sortedNodes.forEach((node) => {
        if (node.id === "root") return;
        this.addNode(Node.fromJSON(node));
      });
    }

    // Add animations
    if (json.animations) {
      json.animations.forEach((animation) =>
        this.addAnimation(Animation.fromJSON(animation))
      );
    }
  }

  destroy() {
    Object.values(this.nodes).forEach((node) => node.destroy());
    Object.values(this.meshes).forEach((mesh) => mesh.destroy());
    Object.values(this.materials).forEach((material) => material.destroy());
    Object.values(this.accessors).forEach((accessor) => accessor.destroy());
    Object.values(this.images).forEach((image) => image.destroy());
    Object.values(this.animations).forEach((animation) => animation.destroy());

    this.nodes$.complete();
    this.meshes$.complete();
    this.materials$.complete();
    this.accessors$.complete();
    this.images$.complete();
    this.animations$.complete();
  }
}
