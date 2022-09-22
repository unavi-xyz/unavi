import {
  BoxBufferGeometry,
  Color,
  CylinderBufferGeometry,
  Group,
  Mesh,
  MeshBasicMaterial,
  MeshStandardMaterial,
  Object3D,
  Quaternion,
  SphereBufferGeometry,
  Vector3,
} from "three";

import {
  BoxMeshJSON,
  Entity,
  EntityJSON,
  Material,
  MaterialJSON,
  MeshJSON,
  Scene,
  SceneMessage,
} from "../../scene";
import { PostMessage, Triplet } from "../../types";
import { disposeMaterial, disposeObject } from "../../utils/disposeObject";
import { FromRenderMessage, ToRenderMessage } from "../types";

/*
 * SceneLink handles the synchronization between the {@link Scene} and Three.js.
 */
export class SceneLink {
  root = new Group();
  meshes = new Group();
  visuals = new Group();

  #scene = new Scene();

  #materials = new Map<string, MeshStandardMaterial>();
  #objects = new Map<string, Object3D>();

  #showColliders: boolean = true;
  #colliders = new Map<string, Mesh>();
  #wireframeMaterial = new MeshBasicMaterial({
    color: new Color(0x000000),
    wireframe: true,
  });

  #tempVector3 = new Vector3();
  #defaultMaterial = new MeshStandardMaterial({ color: 0xffffff });
  #postMessage: PostMessage<FromRenderMessage>;

  constructor(postMessage: PostMessage) {
    this.#postMessage = postMessage;
    this.#scene.addThread(postMessage);
    this.root.add(this.visuals);
    this.root.add(this.meshes);
    this.#objects.set("root", this.meshes);

    this.#scene.entities$.subscribe({
      next: (entities) => {
        // Add new entities
        Object.values(entities).forEach((entity) => {
          if (!this.#objects.has(entity.id)) {
            this.addEntity(entity);
          }
        });
      },
    });

    this.#scene.materials$.subscribe({
      next: (materials) => {
        // Add new materials
        Object.values(materials).forEach((material) => {
          if (!this.#materials.has(material.id)) {
            this.addMaterial(material);
          }
        });
      },
    });
  }

  onmessage = (event: MessageEvent<ToRenderMessage>) => {
    this.#scene.onmessage(event as MessageEvent<SceneMessage>);
  };

  addMaterial(material: Material) {
    const materialObject = new MeshStandardMaterial({
      color: new Color().fromArray(material.color),
      roughness: material.roughness,
      metalness: material.metalness,
    });

    this.#materials.set(material.id, materialObject);

    // Subscribe to material updates
    material.color$.subscribe({
      next: (color) => {
        materialObject.color = new Color().fromArray(color);
      },
    });

    material.roughness$.subscribe({
      next: (roughness) => {
        materialObject.roughness = roughness;
      },
    });

    material.metalness$.subscribe({
      next: (metalness) => {
        materialObject.metalness = metalness;
      },
    });
  }

  removeMaterial(materialId: string) {
    const material = this.#materials.get(materialId);
    if (!material) throw new Error(`Material not found: ${materialId}`);

    // Remove material from objects
    this.#objects.forEach((object) => {
      if (object instanceof Mesh && object.material === material) {
        object.material = this.#defaultMaterial;
      }
    });

    // Remove from materials
    this.#materials.delete(materialId);

    // Dispose material
    disposeMaterial(material);
  }

  addEntity(entity: Entity) {
    if (entity.id === "root") return;

    const parent = this.#objects.get(entity.parentId);
    if (!parent) throw new Error("Parent not found");

    // Create object
    switch (entity.mesh?.type) {
      case "Box":
        const boxMaterial = entity.materialId
          ? this.#materials.get(entity.materialId)
          : this.#defaultMaterial;
        if (!boxMaterial) throw new Error("Material not found");

        const box = new Mesh(
          new BoxBufferGeometry(
            entity.mesh.width,
            entity.mesh.height,
            entity.mesh.depth
          ),
          boxMaterial
        );

        copyTransform(box, entity);

        parent.add(box);
        this.#objects.set(entity.id, box);
        break;
      case "Sphere":
        const sphereMaterial = entity.materialId
          ? this.#materials.get(entity.materialId)
          : this.#defaultMaterial;
        if (!sphereMaterial) throw new Error("Material not found");

        const sphere = new Mesh(
          new SphereBufferGeometry(
            entity.mesh.radius,
            entity.mesh.widthSegments,
            entity.mesh.heightSegments
          ),
          sphereMaterial
        );

        copyTransform(sphere, entity);

        parent.add(sphere);
        this.#objects.set(entity.id, sphere);
        break;
      case "Cylinder":
        const cylinderMaterial = entity.materialId
          ? this.#materials.get(entity.materialId)
          : this.#defaultMaterial;
        if (!cylinderMaterial) throw new Error("Material not found");

        const cylinder = new Mesh(
          new CylinderBufferGeometry(
            entity.mesh.radius,
            entity.mesh.radius,
            entity.mesh.height,
            entity.mesh.radialSegments
          ),
          cylinderMaterial
        );

        copyTransform(cylinder, entity);

        parent.add(cylinder);
        this.#objects.set(entity.id, cylinder);
        break;
      default:
        const group = new Group();
        copyTransform(group, entity);

        this.meshes.add(group);
        this.#objects.set(entity.id, group);
    }

    // Create collider visual
    this.#createColliderVisual(entity.id);

    // Subscribe to entity updates
    entity.mesh$.subscribe({
      next: (mesh) => {
        this.#updateMesh(entity.id, mesh);
      },
    });

    entity.parentId$.subscribe({
      next: (parentId) => {
        this.#moveEntity(entity.id, parentId);
      },
    });

    entity.materialId$.subscribe({
      next: (materialId) => {
        this.#setMaterial(entity.id, materialId);
      },
    });

    entity.position$.subscribe({
      next: (position) => {
        const object = this.#objects.get(entity.id);
        if (!object) throw new Error("Object not found");
        object.position.fromArray(position);
      },
    });

    entity.rotation$.subscribe({
      next: (rotation) => {
        const object = this.#objects.get(entity.id);
        if (!object) throw new Error("Object not found");
        object.rotation.fromArray(rotation);
      },
    });

    entity.scale$.subscribe({
      next: (scale) => {
        const object = this.#objects.get(entity.id);
        if (!object) throw new Error("Object not found");
        object.scale.fromArray(scale);
      },
    });
  }

  removeEntity(entityId: string) {
    const object = this.#objects.get(entityId);
    if (!object) throw new Error(`Object not found: ${entityId}`);

    // Repeat for children
    const entity = this.#scene.entities[entityId];
    if (!entity) throw new Error(`Entity not found: ${entityId}`);
    entity.childrenIds.forEach((childId) => this.removeEntity(childId));

    if (entityId === "root") return;

    // Remove from scene
    object.removeFromParent();
    this.#objects.delete(entityId);

    // Dispose object
    disposeObject(object);

    // Remove collider visual
    this.#removeColliderVisual(entityId);
  }

  #moveEntity(entityId: string, parentId: string) {
    const object = this.#objects.get(entityId);
    if (!object) throw new Error(`Object not found: ${entityId}`);

    const parent = this.#objects.get(parentId);
    if (!parent) throw new Error(`Parent not found: ${parentId}`);

    // Save object transform
    const position = object.getWorldPosition(new Vector3());
    const quaternion = object.getWorldQuaternion(new Quaternion());

    // Set parent
    parent.add(object);

    // Restore object transform
    const inverseParentRotation = parent
      .getWorldQuaternion(new Quaternion())
      .invert();
    object.position.copy(parent.worldToLocal(position));
    object.quaternion.multiplyQuaternions(quaternion, inverseParentRotation);

    // Update collider visual
    this.#createColliderVisual(entityId);
  }

  #setMaterial(entityId: string, materialId: string | null) {
    const object = this.#objects.get(entityId);
    if (!object) throw new Error("Object not found");
    if (!(object instanceof Mesh)) throw new Error("Object is not a mesh");

    const material = materialId
      ? this.#materials.get(materialId)
      : this.#defaultMaterial;
    if (!material) throw new Error("Material not found");

    // Set material
    object.material = material;
  }

  #updateMesh(entityId: string, mesh: MeshJSON | null) {
    const object = this.#objects.get(entityId);
    if (!object) throw new Error(`Object not found: ${entityId}`);
    if (!(object instanceof Mesh)) throw new Error("Object is not a mesh");

    object.geometry.dispose();

    switch (mesh?.type) {
      case "Box":
        // Update geometry
        object.geometry = new BoxBufferGeometry(
          mesh.width,
          mesh.height,
          mesh.depth
        );
        break;
      case "Sphere":
        // Update geometry
        object.geometry = new SphereBufferGeometry(
          mesh.radius,
          mesh.widthSegments,
          mesh.heightSegments
        );
        break;
      case "Cylinder":
        // Update geometry
        object.geometry = new CylinderBufferGeometry(
          mesh.radius,
          mesh.radius,
          mesh.height,
          mesh.radialSegments
        );
        break;
    }

    // Update collider visual
    this.#createColliderVisual(entityId);
  }

  findId(target: Object3D): string | undefined {
    for (const [id, object] of this.#objects) {
      if (object === target) return id;
    }
    return undefined;
  }

  findObject(entityId: string): Object3D | undefined {
    return this.#objects.get(entityId);
  }

  saveTransform(entityId: string) {
    const entity = this.#scene.entities[entityId];
    if (!entity) throw new Error(`Entity not found: ${entityId}`);

    const object = this.findObject(entityId);
    if (!object) throw new Error("Object not found");

    const position = object.position.toArray();
    const scale = object.scale.toArray();
    const euler = object.rotation.toArray();
    const rotation = [euler[0], euler[1], euler[2]] as Triplet;

    this.#scene.updateEntity(entityId, {
      position,
      rotation,
      scale,
    });

    // Update collider visual
    this.#createColliderVisual(entityId);

    // Repeat for children
    entity.childrenIds.forEach((childId) => this.saveTransform(childId));
  }

  #removeColliderVisual(entityId: string) {
    const collider = this.#colliders.get(entityId);
    if (!collider) return;

    this.#colliders.delete(entityId);
    collider.removeFromParent();
    collider.geometry.dispose();
  }

  #createColliderVisual(entityId: string) {
    // const entity = this.#scene.entities[entityId];
    // let collider: Mesh | null = null;
    // switch (entity.collider?.type) {
    //   case "box":
    //     const extents = entity.collider.extents ?? [1, 1, 1];
    //     collider = new Mesh(
    //       new BoxBufferGeometry(...extents),
    //       this.#wireframeMaterial
    //     );
    //     break;
    //   case "sphere":
    //     const radius = entity.collider.radius ?? 1;
    //     collider = new Mesh(
    //       new SphereBufferGeometry(radius),
    //       this.#wireframeMaterial
    //     );
    //     break;
    //   case "cylinder":
    //     collider = new Mesh(
    //       new CylinderBufferGeometry(
    //         entity.collider.radius ?? 1,
    //         entity.collider.radius ?? 1,
    //         entity.collider.height ?? 1
    //       ),
    //       this.#wireframeMaterial
    //     );
    //     break;
    // }
    // // Remove previous collider
    // this.#removeColliderVisual(entityId);
    // // Add new collider
    // if (collider) {
    //   const object = this.#objects.get(entityId);
    //   if (!object) throw new Error("Object not found");
    //   const position = object.getWorldPosition(this.#tempVector3);
    //   collider.position.copy(position);
    //   this.#colliders.set(entityId, collider);
    //   this.visuals.add(collider);
    // }
  }

  destroy() {
    this.#scene.destroy();
  }
}

function copyTransform(object: Object3D, entity: Entity) {
  object.position.fromArray(entity.position);
  object.rotation.fromArray(entity.rotation);
  object.scale.fromArray(entity.scale);
}
