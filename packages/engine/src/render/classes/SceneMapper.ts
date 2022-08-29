import { DESERIALIZE_MODE, Not, createWorld, defineQuery } from "bitecs";
import {
  BoxBufferGeometry,
  CylinderBufferGeometry,
  Group,
  Mesh,
  MeshStandardMaterial,
  Object3D,
  SphereBufferGeometry,
} from "three";

import { Box, Child, Cylinder, SceneObject, Sphere, config, deserialize } from "../../ecs";

export class SceneMapper {
  root = new Group();

  #world = createWorld(config);
  #boxQuery = defineQuery([Box]);
  #sphereQuery = defineQuery([Sphere]);
  #cylinderQuery = defineQuery([Cylinder]);
  #sceneObjectQuery = defineQuery([SceneObject]);
  #childObjectsQuery = defineQuery([SceneObject, Child]);
  #rootObjectsQuery = defineQuery([SceneObject, Not(Child)]);

  #objectMap = new Map<number, Object3D>();
  #meshMap = new Map<number, Mesh>();

  updateScene(buffer: ArrayBuffer) {
    // Load world changes
    deserialize(this.#world, buffer, DESERIALIZE_MODE.REPLACE);

    // Update scene
    const boxes = this.#boxQuery(this.#world);
    boxes.forEach((eid) => {
      const width = Box.width[eid];
      const height = Box.height[eid];
      const depth = Box.depth[eid];

      const object = this.#meshMap.get(eid);
      if (object) {
        const newGeometry = new BoxBufferGeometry(width, height, depth);
        object.geometry.dispose();
        object.geometry = newGeometry;
      } else {
        // Create new object
        const geometry = new BoxBufferGeometry(width, height, depth);
        const material = new MeshStandardMaterial({ color: 0xffffff });
        const object = new Mesh(geometry, material);

        this.#objectMap.set(eid, object);
        this.#meshMap.set(eid, object);
      }
    });

    const spheres = this.#sphereQuery(this.#world);
    spheres.forEach((eid) => {
      const radius = Sphere.radius[eid];
      const widthSegments = Sphere.widthSegments[eid];
      const heightSegments = Sphere.heightSegments[eid];

      const object = this.#meshMap.get(eid);
      if (object) {
        const newGeometry = new SphereBufferGeometry(radius, widthSegments, heightSegments);
        object.geometry.dispose();
        object.geometry = newGeometry;
      } else {
        // Create new object
        const geometry = new SphereBufferGeometry(radius, widthSegments, heightSegments);
        const material = new MeshStandardMaterial({ color: 0xffffff });
        const object = new Mesh(geometry, material);

        this.#objectMap.set(eid, object);
        this.#meshMap.set(eid, object);
      }
    });

    const cylinders = this.#cylinderQuery(this.#world);
    cylinders.forEach((eid) => {
      const radiusTop = Cylinder.radiusTop[eid];
      const radiusBottom = Cylinder.radiusBottom[eid];
      const height = Cylinder.height[eid];
      const radialSegments = Cylinder.radialSegments[eid];

      const object = this.#meshMap.get(eid);
      if (object) {
        const newGeometry = new CylinderBufferGeometry(
          radiusTop,
          radiusBottom,
          height,
          radialSegments
        );
        object.geometry.dispose();
        object.geometry = newGeometry;
      } else {
        // Create new object
        const geometry = new CylinderBufferGeometry(
          radiusTop,
          radiusBottom,
          height,
          radialSegments
        );
        const material = new MeshStandardMaterial({ color: 0xffffff });
        const object = new Mesh(geometry, material);

        this.#objectMap.set(eid, object);
        this.#meshMap.set(eid, object);
      }
    });

    const sceneObjects = this.#sceneObjectQuery(this.#world);
    sceneObjects.forEach((eid) => {
      const object = this.#objectMap.get(eid);
      if (!object) throw new Error("Object not found");

      // Update transform
      object.position.x = SceneObject.position.x[eid];
      object.position.y = SceneObject.position.y[eid];
      object.position.z = SceneObject.position.z[eid];
      object.quaternion.x = SceneObject.rotation.x[eid];
      object.quaternion.y = SceneObject.rotation.y[eid];
      object.quaternion.z = SceneObject.rotation.z[eid];
      object.quaternion.w = SceneObject.rotation.w[eid];
      object.scale.x = SceneObject.scale.x[eid];
      object.scale.y = SceneObject.scale.y[eid];
      object.scale.z = SceneObject.scale.z[eid];
    });

    const childSceneObjects = this.#childObjectsQuery(this.#world);
    childSceneObjects.forEach((eid) => {
      const object = this.#objectMap.get(eid);
      if (!object) throw new Error("Object not found");

      const peid = Child.parent[eid];
      const parent = this.#objectMap.get(peid);
      if (!parent) throw new Error("Parent not found");

      // Add to parent
      parent.add(object);
    });

    const rootObjects = this.#rootObjectsQuery(this.#world);
    rootObjects.forEach((eid) => {
      const object = this.#objectMap.get(eid);
      if (!object) throw new Error("Object not found");

      // Add to root
      this.root.add(object);
    });
  }

  findObject(eid: number): Object3D | undefined {
    return this.#objectMap.get(eid);
  }

  findId(object: Object3D): number | undefined {
    let found: number | undefined;

    this.#objectMap.forEach((value, key) => {
      if (value === object) found = key;
    });

    return found;
  }
}
