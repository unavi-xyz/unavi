import {
  Bone,
  Group,
  Line,
  LineLoop,
  LineSegments,
  Matrix4,
  Mesh,
  Points,
  Skeleton,
  SkinnedMesh,
} from "three";

import { Entity } from "../../scene";
import { WEBGL_CONSTANTS } from "../constants";
import { RenderScene } from "../RenderScene";
import { defaultMaterial } from "./constants";
import { copyTransform } from "./copyTransform";
import { createColliderVisual } from "./createColliderVisual";
import { createMeshGeometry } from "./createMeshGeometry";
import { moveEntity } from "./moveEntity";
import { removeEntityObject } from "./removeEntityObject";
import { SceneMap } from "./SceneMap";
import { setMaterial } from "./setMaterial";
import { updateGlobalTransform } from "./updateGlobalTransform";
import { updateMesh } from "./updateMesh";

export function createEntity(
  entity: Entity,
  map: SceneMap,
  scene: RenderScene,
  visuals: Group
) {
  if (entity.id === "root") return;

  const parent = map.objects.get(entity.parentId);
  if (!parent) {
    if (!entity.parent) throw new Error("Parent not found");
    createEntity(entity.parent, map, scene, visuals);
    createEntity(entity, map, scene, visuals);
    return;
  }

  // Create object
  switch (entity.mesh?.type) {
    case "Box":
    case "Sphere":
    case "Cylinder":
      // Get material
      const material = entity.materialId
        ? map.materials.get(entity.materialId)
        : defaultMaterial;
      if (!material) throw new Error("Material not found");

      // Create geometry
      const geometry = createMeshGeometry(entity.mesh, map, scene);

      // Create mesh
      const mesh = new Mesh(geometry, material);

      // Add to scene
      map.objects.set(entity.id, mesh);
      copyTransform(mesh, entity);
      parent.add(mesh);
      break;
    case "Primitive":
    case "Skin":
      // Get material
      const primitiveMaterial = entity.materialId
        ? map.materials.get(entity.materialId)
        : defaultMaterial;
      if (!primitiveMaterial) throw new Error("Material not found");

      // Create geometry
      const primitiveGeometry = createMeshGeometry(
        entity.mesh.toJSON(),
        map,
        scene
      );

      let primitiveMesh:
        | Mesh
        | SkinnedMesh
        | LineSegments
        | LineLoop
        | Line
        | Points;
      switch (entity.mesh.mode) {
        case WEBGL_CONSTANTS.TRIANGLES:
        case WEBGL_CONSTANTS.TRIANGLE_STRIP:
        case WEBGL_CONSTANTS.TRIANGLE_FAN:
          const isSkin = entity.mesh.type === "Skin";
          primitiveMesh = isSkin
            ? new SkinnedMesh(primitiveGeometry, primitiveMaterial)
            : new Mesh(primitiveGeometry, primitiveMaterial);

          if (primitiveMesh instanceof SkinnedMesh) {
            const normalized =
              primitiveMesh.geometry.attributes.skinWeight.normalized;
            if (!normalized) primitiveMesh.normalizeSkinWeights();
          }
          break;
        case WEBGL_CONSTANTS.LINES:
          primitiveMesh = new LineSegments(
            primitiveGeometry,
            primitiveMaterial
          );
          break;
        case WEBGL_CONSTANTS.LINE_STRIP:
          primitiveMesh = new Line(primitiveGeometry, primitiveMaterial);
          break;
        case WEBGL_CONSTANTS.LINE_LOOP:
          primitiveMesh = new LineLoop(primitiveGeometry, primitiveMaterial);
          break;
        case WEBGL_CONSTANTS.POINTS:
          primitiveMesh = new Points(primitiveGeometry, primitiveMaterial);
          break;
        default:
          throw new Error(`Unknown primitive mode: ${entity.mesh.mode}`);
      }

      // Add to scene
      map.objects.set(entity.id, primitiveMesh);
      copyTransform(primitiveMesh, entity);
      parent.add(primitiveMesh);
      break;
    default:
      // Check if joint
      let skinParent: Entity | undefined;
      Object.values(scene.entities).forEach((entity) => {
        if (
          entity.mesh?.type === "Skin" &&
          entity.mesh.jointIds.includes(entity.id)
        )
          skinParent = entity;
      });

      // Create object
      const object = skinParent ? new Bone() : new Group();

      if (skinParent) {
        const skinObject = map.objects.get(skinParent.id);
        if (!skinObject) throw new Error("Skin not found");

        // Create skeleton
        skinObject.traverse((child) => {
          if (!(child instanceof SkinnedMesh)) return;

          const bones: Bone[] = [];
          const boneInverses: Matrix4[] = [];

          if (entity.mesh?.type !== "Skin") throw new Error("Mesh not skin");
          if (!entity.mesh.inverseBindMatricesId)
            throw new Error("No inverse bind matrices");

          const inverseBindMatrices =
            scene.accessors[entity.mesh.inverseBindMatricesId].array;

          entity.mesh.jointIds.forEach((jointId, i) => {
            const bone = map.objects.get(jointId);
            if (!bone) throw new Error("Joint not found");
            if (!(bone instanceof Bone)) throw new Error("Joint is not a bone");

            const matrix = new Matrix4();
            matrix.fromArray(inverseBindMatrices, i * 16);

            bones.push(bone);
            boneInverses.push(matrix);
          });

          child.bind(new Skeleton(bones, boneInverses), child.matrixWorld);
        });
      }

      // Add to scene
      map.objects.set(entity.id, object);
      copyTransform(object, entity);
      parent.add(object);
  }

  // Create collider visual
  createColliderVisual(entity.id, map, scene, visuals);

  // Subscribe to entity updates
  entity.mesh$.subscribe({
    next: (mesh) => {
      if (!mesh?.type || mesh.type === "glTF") return;
      updateMesh(entity.id, mesh?.toJSON(), map, scene);
    },
    complete: () => {
      // Remove object when entity is removed
      // Nothing special about this callback, could be added to any of these subscriptions
      removeEntityObject(entity.id, map);
    },
  });

  entity.parentId$.subscribe({
    next: (parentId) => {
      moveEntity(entity.id, parentId, map, scene);
    },
  });

  entity.materialId$.subscribe({
    next: (materialId) => {
      setMaterial(entity.id, materialId, map);
    },
  });

  entity.position$.subscribe({
    next: (position) => {
      const object = map.objects.get(entity.id);
      if (!object) throw new Error("Object not found");

      // Translate object
      object.position.fromArray(position);

      updateGlobalTransform(entity.id, map, scene);
    },
  });

  entity.rotation$.subscribe({
    next: (rotation) => {
      const object = map.objects.get(entity.id);
      if (!object) throw new Error("Object not found");

      // Rotate object
      object.rotation.fromArray(rotation);

      updateGlobalTransform(entity.id, map, scene);
    },
  });

  entity.scale$.subscribe({
    next: (scale) => {
      const object = map.objects.get(entity.id);
      if (!object) throw new Error("Object not found");

      // Scale object
      object.scale.fromArray(scale);
    },
  });

  entity.collider$.subscribe({
    next: () => {
      createColliderVisual(entity.id, map, scene, visuals);
    },
  });
}
