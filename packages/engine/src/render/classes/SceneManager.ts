import {
  BoxBufferGeometry,
  Color,
  CylinderBufferGeometry,
  Group,
  Mesh,
  MeshStandardMaterial,
  Object3D,
  Quaternion,
  SphereBufferGeometry,
  Vector3,
} from "three";

import { Entity, Material, PostMessage } from "../../types";
import { disposeMaterial, disposeObject } from "../../utils/disposeObject";
import { FromRenderMessage, ToRenderMessage } from "../types";

export class SceneManager {
  scene = new Group();
  #root = new Group();

  #entities = new Map<string, Entity>();
  #materials = new Map<string, MeshStandardMaterial>();
  #objects = new Map<string, Object3D>();

  #defaultMaterial = new MeshStandardMaterial({ color: 0xffffff });
  #postMessage: PostMessage<FromRenderMessage>;

  constructor(postMessage: PostMessage<FromRenderMessage>) {
    this.#postMessage = postMessage;
    this.scene.add(this.#root);
    this.#objects.set("root", this.#root);
  }

  onmessage(event: MessageEvent<ToRenderMessage>) {
    const { subject, data } = event.data;

    switch (subject) {
      case "add_entity":
        this.addEntity(data);
        break;
      case "set_transform":
        this.setTransform(
          data.entityId,
          data.position,
          data.rotation,
          data.scale
        );
        break;
      case "set_geometry":
        this.setGeometry(data.entityId, data.geometry);
        break;
      case "remove_entity":
        this.removeEntity(data);
        break;
      case "move_entity":
        this.moveEntity(data.entityId, data.parentId);
        break;
      case "add_material":
        this.addMaterial(data);
        break;
      case "set_material":
        this.setMaterial(data.entityId, data.materialId);
        break;
      case "remove_material":
        this.removeMaterial(data);
        break;
    }
  }

  addMaterial(material: Material) {
    const color = new Color().fromArray(material.color);

    this.#materials.set(
      material.id,
      new MeshStandardMaterial({
        color,
        roughness: material.roughness,
        metalness: material.metalness,
      })
    );
  }

  setMaterial(entityId: string, materialId: string | null) {
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

  removeMaterial(materialId: string) {
    const material = this.#materials.get(materialId);
    if (!material) throw new Error(`Material not found: ${materialId}`);

    // Remove material from objects
    this.#objects.forEach((object) => {
      if (object instanceof Mesh && object.material.uuid === material.uuid) {
        object.material = this.#defaultMaterial;
      }
    });

    // Remove material
    this.#materials.delete(materialId);

    // Dispose material
    disposeMaterial(material);
  }

  addEntity(entity: Entity) {
    if (entity.id === "root") {
      this.#entities.set(entity.id, entity);
      return;
    }

    const parent = entity.parent
      ? this.#objects.get(entity.parent)
      : this.#root;

    if (!parent) throw new Error("Parent not found");

    this.#entities.set(entity.id, entity);

    switch (entity.type) {
      case "Group":
        const group = new Group();
        copyTransform(group, entity);

        this.#root.add(group);
        this.#objects.set(entity.id, group);
        break;
      case "Box":
        const boxMaterial = entity.material
          ? this.#materials.get(entity.material)
          : this.#defaultMaterial;
        if (!boxMaterial) throw new Error("Material not found");

        const box = new Mesh(
          new BoxBufferGeometry(entity.width, entity.height, entity.depth),
          boxMaterial
        );
        copyTransform(box, entity);

        parent.add(box);

        this.#objects.set(entity.id, box);
        break;
      case "Sphere":
        const sphereMaterial = entity.material
          ? this.#materials.get(entity.material)
          : this.#defaultMaterial;
        if (!sphereMaterial) throw new Error("Material not found");

        const sphere = new Mesh(
          new SphereBufferGeometry(
            entity.radius,
            entity.widthSegments,
            entity.heightSegments
          ),
          sphereMaterial
        );
        copyTransform(sphere, entity);

        parent.add(sphere);
        this.#objects.set(entity.id, sphere);
        break;
      case "Cylinder":
        const cylinderMaterial = entity.material
          ? this.#materials.get(entity.material)
          : this.#defaultMaterial;
        if (!cylinderMaterial) throw new Error("Material not found");

        const cylinder = new Mesh(
          new CylinderBufferGeometry(
            entity.radiusTop,
            entity.radiusBottom,
            entity.height,
            entity.radialSegments
          ),
          cylinderMaterial
        );
        copyTransform(cylinder, entity);

        parent.add(cylinder);
        this.#objects.set(entity.id, cylinder);
        break;
      default:
        throw new Error(`Unknown entity type: ${entity}`);
    }
  }

  setTransform(
    entityId: string,
    position: number[],
    rotation: number[],
    scale: number[]
  ) {
    const object = this.#objects.get(entityId);
    if (!object) throw new Error(`Object not found: ${entityId}`);

    const entity = this.#entities.get(entityId);
    if (!entity) throw new Error(`Entity not found: ${entityId}`);

    // Set entity transform
    entity.position = [position[0], position[1], position[2]];
    entity.rotation = [rotation[0], rotation[1], rotation[2]];
    entity.scale = [scale[0], scale[1], scale[2]];

    // Set object transform
    copyTransform(object, entity);
  }

  setGeometry(entityId: string, geometry: number[]) {
    const object = this.#objects.get(entityId);
    if (!object) throw new Error(`Object not found: ${entityId}`);

    const entity = this.#entities.get(entityId);
    if (!entity) throw new Error(`Entity not found: ${entityId}`);

    switch (entity.type) {
      case "Box":
        // Update entity
        entity.width = geometry[0];
        entity.height = geometry[1];
        entity.depth = geometry[2];

        // Update geometry
        const box = object as Mesh;
        box.geometry.dispose();
        box.geometry = new BoxBufferGeometry(
          entity.width,
          entity.height,
          entity.depth
        );
        break;
      case "Sphere":
        // Update entity
        entity.radius = geometry[0];
        entity.widthSegments = geometry[1];
        entity.heightSegments = geometry[2];

        // Update geometry
        const sphere = object as Mesh;
        sphere.geometry.dispose();
        sphere.geometry = new SphereBufferGeometry(
          entity.radius,
          entity.widthSegments,
          entity.heightSegments
        );
        break;
      case "Cylinder":
        // Update entity
        entity.radiusTop = geometry[0];
        entity.radiusBottom = geometry[1];
        entity.height = geometry[2];
        entity.radialSegments = geometry[3];

        // Update geometry
        const cylinder = object as Mesh;
        cylinder.geometry.dispose();
        cylinder.geometry = new CylinderBufferGeometry(
          entity.radiusTop,
          entity.radiusBottom,
          entity.height,
          entity.radialSegments
        );
        break;
    }
  }

  removeEntity(entityId: string) {
    const object = this.#objects.get(entityId);
    if (!object) throw new Error(`Object not found: ${entityId}`);

    // Repeat for children
    const children = this.findChildren(entityId);
    children.forEach((child) => this.removeEntity(child));

    // Remove from scene
    object.removeFromParent();
    this.#objects.delete(entityId);
    this.#entities.delete(entityId);

    // Dispose object
    disposeObject(object);

    // Create new root if needed
    if (entityId === "root") {
      this.#root = new Group();
      this.scene.add(this.#root);
      this.#objects.set("root", this.#root);
    }
  }

  moveEntity(entityId: string, parentId: string | null) {
    const object = this.#objects.get(entityId);
    if (!object) throw new Error(`Object not found: ${entityId}`);

    const parent = parentId ? this.#objects.get(parentId) : this.#root;
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

  findChildren(entityId: string): string[] {
    const object = this.#objects.get(entityId);
    if (!object) throw new Error(`Object not found: ${entityId}`);

    const entity = this.#entities.get(entityId);
    if (!entity) throw new Error(`Entity not found: ${entityId}`);

    return entity.children;
  }

  saveTransform(id: string) {
    const object = this.findObject(id);
    if (!object) throw new Error("Object not found");

    const position = object.position.toArray();
    const rotation = object.quaternion.toArray();
    const scale = object.scale.toArray();

    this.#postMessage({
      subject: "set_transform",
      data: {
        id,
        position,
        rotation,
        scale,
      },
    });

    // Repeat for children
    const children = this.findChildren(id);
    children.forEach((child) => this.saveTransform(child));
  }
}

function copyTransform(object: Object3D, entity: Entity) {
  object.position.fromArray(entity.position);
  object.rotation.fromArray(entity.rotation);
  object.scale.fromArray(entity.scale);
}
