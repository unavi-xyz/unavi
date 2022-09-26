import {
  BoxBufferGeometry,
  BufferAttribute,
  BufferGeometry,
  CanvasTexture,
  ClampToEdgeWrapping,
  Color,
  CylinderBufferGeometry,
  DoubleSide,
  FrontSide,
  Group,
  Line,
  LinearFilter,
  LinearMipMapLinearFilter,
  LinearMipMapNearestFilter,
  LineLoop,
  LineSegments,
  Mesh,
  MeshBasicMaterial,
  MeshStandardMaterial,
  MirroredRepeatWrapping,
  NearestFilter,
  NearestMipMapLinearFilter,
  NearestMipMapNearestFilter,
  Object3D,
  Points,
  Quaternion,
  RepeatWrapping,
  SphereBufferGeometry,
  Texture as ThreeTexture,
  Vector2,
  Vector3,
} from "three";

import { Entity, Material, MeshJSON, SceneMessage } from "../../scene";
import { PostMessage, Quad, Triplet } from "../../types";
import { disposeMaterial, disposeObject } from "../../utils/disposeObject";
import { WEBGL_CONSTANTS } from "../constants";
import { RenderScene } from "../RenderScene";
import { ToRenderMessage } from "../types";

/*
 * SceneLink handles the synchronization between the {@link Scene} and Three.js.
 */
export class SceneLink {
  root = new Group();
  meshes = new Group();
  visuals = new Group();

  #scene: RenderScene;

  #materials = new Map<string, MeshStandardMaterial>();
  #objects = new Map<string, Object3D>();

  #colliders = new Map<string, Mesh>();
  #wireframeMaterial = new MeshBasicMaterial({
    color: new Color(0x000000),
    wireframe: true,
  });

  #tempVector3 = new Vector3();
  #tempQuaternion = new Quaternion();
  #defaultMaterial = new MeshStandardMaterial({ color: 0xffffff });

  #cache = {
    attributes: new Map<string, BufferAttribute>(),
    images: new Map<string, ImageBitmap>(),
  };

  constructor(postMessage: PostMessage) {
    this.#scene = new RenderScene(postMessage);

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

    const { subject, data } = event.data;
    switch (subject) {
      case "show_visuals":
        this.visuals.visible = data.visible;
        break;
    }
  };

  addMaterial(material: Material) {
    const materialObject = new MeshStandardMaterial();
    this.#materials.set(material.id, materialObject);

    // Subscribe to material updates
    material.alpha$.subscribe({
      next: (alpha) => {
        materialObject.opacity = alpha;
      },
    });

    material.alphaCutoff$.subscribe({
      next: (alphaCutoff) => {
        materialObject.alphaTest = alphaCutoff;
      },
    });

    material.alphaMode$.subscribe({
      next: (alphaMode) => {
        if (alphaMode === "BLEND") {
          materialObject.transparent = true;
          materialObject.depthWrite = false;
        } else {
          materialObject.transparent = false;
          materialObject.depthWrite = true;
        }
      },
    });

    material.doubleSided$.subscribe({
      next: (doubleSided) => {
        materialObject.side = doubleSided ? DoubleSide : FrontSide;
      },
    });

    material.emissive$.subscribe({
      next: (emissive) => {
        materialObject.emissive = new Color().fromArray(emissive);
      },
    });

    material.normalScale$.subscribe({
      next: (normalScale) => {
        materialObject.normalScale = new Vector2(normalScale, normalScale);
      },
    });

    material.occlusionStrength$.subscribe({
      next: (occlusionStrength) => {
        materialObject.aoMapIntensity = occlusionStrength;
      },
    });

    material.color$.subscribe({
      next: (color) => {
        materialObject.color = new Color().fromArray(color);
      },
      complete: () => {
        this.removeMaterial(material.id);
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

    material.colorTextureId$.subscribe({
      next: (colorTextureId) => {
        materialObject.map = colorTextureId
          ? this.#loadTexture(colorTextureId)
          : null;
      },
    });

    material.normalTextureId$.subscribe({
      next: (normalTextureId) => {
        materialObject.normalMap = normalTextureId
          ? this.#loadTexture(normalTextureId)
          : null;
      },
    });

    material.occlusionTextureId$.subscribe({
      next: (occlusionTextureId) => {
        materialObject.aoMap = occlusionTextureId
          ? this.#loadTexture(occlusionTextureId)
          : null;
      },
    });

    material.emissiveTextureId$.subscribe({
      next: (emissiveTextureId) => {
        materialObject.emissiveMap = emissiveTextureId
          ? this.#loadTexture(emissiveTextureId)
          : null;
      },
    });

    material.metallicRoughnessTextureId$.subscribe({
      next: (metallicRoughnessTextureId) => {
        materialObject.metalnessMap = metallicRoughnessTextureId
          ? this.#loadTexture(metallicRoughnessTextureId)
          : null;
        materialObject.roughnessMap = metallicRoughnessTextureId
          ? this.#loadTexture(metallicRoughnessTextureId)
          : null;
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
    if (!parent) {
      if (!entity.parent) throw new Error("Parent not found");
      this.addEntity(entity.parent);
      this.addEntity(entity);
      return;
    }

    // Create object
    switch (entity.mesh?.type) {
      case "Box":
      case "Sphere":
      case "Cylinder":
        // Get material
        const material = entity.materialId
          ? this.#materials.get(entity.materialId)
          : this.#defaultMaterial;
        if (!material) throw new Error("Material not found");

        // Create geometry
        const geometry = this.#createMeshGeometry(entity.mesh);

        // Create mesh
        const mesh = new Mesh(geometry, material);
        this.#objects.set(entity.id, mesh);

        // Add to scene
        copyTransform(mesh, entity);
        parent.add(mesh);
        break;
      case "Primitive":
        // Get material
        const primitiveMaterial = entity.materialId
          ? this.#materials.get(entity.materialId)
          : this.#defaultMaterial;
        if (!primitiveMaterial) throw new Error("Material not found");

        // Create geometry
        const primitiveGeometry = this.#createMeshGeometry(entity.mesh);

        let primitiveMesh;
        switch (entity.mesh.mode) {
          case WEBGL_CONSTANTS.TRIANGLES:
          case WEBGL_CONSTANTS.TRIANGLE_STRIP:
          case WEBGL_CONSTANTS.TRIANGLE_FAN:
            // const isSkinnedMesh = withMeshAndSkin.reduce((acc, neid) => {
            //   const nodeMesh = NodeMesh.mesh[neid];
            //   return acc || nodeMesh === primitiveMesh;
            // }, false);

            primitiveMesh = new Mesh(primitiveGeometry, primitiveMaterial);

            // if (isSkinnedMesh) {
            //   const normalized = mesh.geometry.attributes.skinWeight.normalized;
            //   if (!normalized) mesh.normalizeSkinWeights();
            // }
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
        this.#objects.set(entity.id, primitiveMesh);

        // Add to scene
        copyTransform(primitiveMesh, entity);
        parent.add(primitiveMesh);
        break;
      default:
        // Create group
        const group = new Group();
        this.#objects.set(entity.id, group);

        // Add to scene
        copyTransform(group, entity);
        this.meshes.add(group);
    }

    // Create collider visual
    this.#createColliderVisual(entity.id);

    // Subscribe to entity updates
    entity.mesh$.subscribe({
      next: (mesh) => {
        if (!mesh?.type || mesh.type === "glTF") return;
        this.#updateMesh(entity.id, mesh?.toJSON());
      },
    });

    entity.parentId$.subscribe({
      next: (parentId) => {
        this.#moveEntity(entity.id, parentId);
      },
      complete: () => {
        this.#removeEntityObject(entity.id);
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

        this.#updateGlobalTransform(entity.id);
      },
    });

    entity.rotation$.subscribe({
      next: (rotation) => {
        const object = this.#objects.get(entity.id);
        if (!object) throw new Error("Object not found");
        object.rotation.fromArray(rotation);

        this.#updateGlobalTransform(entity.id);
      },
    });

    entity.scale$.subscribe({
      next: (scale) => {
        const object = this.#objects.get(entity.id);
        if (!object) throw new Error("Object not found");
        object.scale.fromArray(scale);
      },
    });

    entity.collider$.subscribe({
      next: () => {
        this.#createColliderVisual(entity.id);
      },
    });
  }

  removeEntity(entityId: string) {
    const entity = this.#scene.entities[entityId];
    if (!entity) throw new Error(`Entity not found: ${entityId}`);

    // Repeat for children
    entity.childrenIds.forEach((childId) => this.removeEntity(childId));

    // Remove object
    this.#removeEntityObject(entityId);
  }

  #removeEntityObject(entityId: string) {
    const object = this.#objects.get(entityId);
    if (!object) throw new Error(`Object not found: ${entityId}`);

    // Don't remove root object
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

    const parentObject = this.#objects.get(parentId);
    if (!parentObject) throw new Error(`Parent not found: ${parentId}`);

    // Save object transform
    const position = object.getWorldPosition(new Vector3());
    const quaternion = object.getWorldQuaternion(new Quaternion());

    // Set parent
    parentObject.add(object);

    // Restore object transform
    const inverseParentRotation = parentObject
      .getWorldQuaternion(new Quaternion())
      .invert();
    object.position.copy(parentObject.worldToLocal(position));
    object.quaternion.multiplyQuaternions(quaternion, inverseParentRotation);

    // Subscribe to global transform changes
    const parent = this.#scene.entities[parentId];

    parent.globalPosition$.subscribe({
      next: () => {
        this.#updateGlobalTransform(entityId);
      },
    });

    parent.globalQuaternion$.subscribe({
      next: () => {
        this.#updateGlobalTransform(entityId);
      },
    });
  }

  #setMaterial(entityId: string, materialId: string | null) {
    const object = this.#objects.get(entityId);
    if (!object) throw new Error("Object not found");
    const isMesh = object instanceof Mesh;
    if (!isMesh) {
      if (materialId === null) return;
      throw new Error("Object is not a mesh");
    }

    const material = materialId
      ? this.#materials.get(materialId)
      : this.#defaultMaterial;
    if (!material) throw new Error("Material not found");

    // Set material
    object.material = material;
  }

  #loadTexture(textureId: string): ThreeTexture {
    const { sourceId, magFilter, minFilter, wrapS, wrapT } =
      this.#scene.textures[textureId];

    if (sourceId === null) throw new Error("Texture source not found");

    const image = this.#loadImage(sourceId);
    const threeTexture = new CanvasTexture(image);
    threeTexture.needsUpdate = true;

    switch (magFilter) {
      case WEBGL_CONSTANTS.NEAREST:
        threeTexture.magFilter = NearestFilter;
        break;
      case WEBGL_CONSTANTS.LINEAR:
        threeTexture.magFilter = LinearFilter;
        break;
      default:
        throw new Error(`Unknown magFilter: ${magFilter}`);
    }

    switch (minFilter) {
      case WEBGL_CONSTANTS.NEAREST:
        threeTexture.minFilter = NearestFilter;
        break;
      case WEBGL_CONSTANTS.LINEAR:
        threeTexture.minFilter = LinearFilter;
        break;
      case WEBGL_CONSTANTS.NEAREST_MIPMAP_NEAREST:
        threeTexture.minFilter = NearestMipMapNearestFilter;
        break;
      case WEBGL_CONSTANTS.LINEAR_MIPMAP_NEAREST:
        threeTexture.minFilter = LinearMipMapNearestFilter;
        break;
      case WEBGL_CONSTANTS.NEAREST_MIPMAP_LINEAR:
        threeTexture.minFilter = NearestMipMapLinearFilter;
        break;
      case WEBGL_CONSTANTS.LINEAR_MIPMAP_LINEAR:
        threeTexture.minFilter = LinearMipMapLinearFilter;
        break;
      default:
        throw new Error(`Unknown minFilter: ${minFilter}`);
    }

    switch (wrapS) {
      case WEBGL_CONSTANTS.CLAMP_TO_EDGE:
        threeTexture.wrapS = ClampToEdgeWrapping;
        break;
      case WEBGL_CONSTANTS.MIRRORED_REPEAT:
        threeTexture.wrapS = MirroredRepeatWrapping;
        break;
      case WEBGL_CONSTANTS.REPEAT:
        threeTexture.wrapS = RepeatWrapping;
        break;
      default:
        throw new Error(`Unknown wrapS: ${wrapS}`);
    }

    switch (wrapT) {
      case WEBGL_CONSTANTS.CLAMP_TO_EDGE:
        threeTexture.wrapT = ClampToEdgeWrapping;
        break;
      case WEBGL_CONSTANTS.MIRRORED_REPEAT:
        threeTexture.wrapT = MirroredRepeatWrapping;
        break;
      case WEBGL_CONSTANTS.REPEAT:
        threeTexture.wrapT = RepeatWrapping;
        break;
      default:
        throw new Error(`Unknown wrapT: ${wrapT}`);
    }

    return threeTexture;
  }

  #loadImage(imageId: string): ImageBitmap {
    const cached = this.#cache.images.get(imageId);
    if (cached) return cached;

    const image = this.#scene.images[imageId];
    const bitmap = image.bitmap;

    this.#cache.images.set(imageId, bitmap);
    return bitmap;
  }

  #updateMesh(entityId: string, json: MeshJSON) {
    const object = this.#objects.get(entityId);
    if (!object) throw new Error(`Object not found: ${entityId}`);
    const isMesh = object instanceof Mesh;
    if (!isMesh) throw new Error("Object is not a mesh");

    object.geometry.dispose();
    object.geometry = this.#createMeshGeometry(json);

    switch (json.type) {
      case "Primitive":
        // Occlusion map needs a second set of UVs
        if (
          object.material.aoMap &&
          object.geometry.attributes.uv &&
          !object.geometry.attributes.uv2
        ) {
          object.geometry.setAttribute("uv2", object.geometry.attributes.uv);
        }

        // Enable flat shading if no normal attribute
        if (!object.geometry.attributes.normal)
          object.material.flatShading = true;

        // Enable vertex colors if color attribute
        if (object.geometry.attributes.color)
          object.material.vertexColors = true;

        // If three.js needs to generate tangents, flip normal map y
        // https://github.com/mrdoob/three.js/issues/11438#issuecomment-507003995
        if (!object.geometry.attributes.tangent)
          object.material.normalScale.y *= -1;
    }
  }

  #createMeshGeometry(json: MeshJSON) {
    switch (json.type) {
      case "Box":
        // Update geometry
        return new BoxBufferGeometry(json.width, json.height, json.depth);
      case "Sphere":
        // Update geometry
        return new SphereBufferGeometry(
          json.radius,
          json.widthSegments,
          json.heightSegments
        );
      case "Cylinder":
        // Update geometry
        return new CylinderBufferGeometry(
          json.radius,
          json.radius,
          json.height,
          json.radialSegments
        );
      case "Primitive":
        // Update geometry
        const primitiveGeometry = new BufferGeometry();
        primitiveGeometry.morphTargetsRelative = true;

        // Indices
        if (json.indicesId) {
          const attribute = this.#loadAttribute(json.indicesId);
          primitiveGeometry.setIndex(attribute);
        }

        // Attributes
        this.#setAttribute(primitiveGeometry, "position", json.POSITION);
        this.#setAttribute(primitiveGeometry, "normal", json.NORMAL);
        this.#setAttribute(primitiveGeometry, "uv", json.TEXCOORD_0);
        this.#setAttribute(primitiveGeometry, "uv2", json.TEXCOORD_1);
        this.#setAttribute(primitiveGeometry, "color", json.COLOR_0);
        this.#setAttribute(primitiveGeometry, "skinIndex", json.JOINTS_0);
        this.#setAttribute(primitiveGeometry, "skinWeight", json.WEIGHTS_0);

        return primitiveGeometry;
    }
  }

  #setAttribute(
    geometry: BufferGeometry,
    threeName: string,
    accessorId: string | null
  ) {
    if (accessorId === null) return;

    const attribute = this.#loadAttribute(accessorId);
    geometry.setAttribute(threeName, attribute);
  }

  #loadAttribute(accessorId: string): BufferAttribute {
    const cached = this.#cache.attributes.get(accessorId);
    if (cached) return cached;

    const { array, elementSize, normalized } =
      this.#scene.accessors[accessorId];
    const attribute = new BufferAttribute(array, elementSize, normalized);

    this.#cache.attributes.set(accessorId, attribute);
    return attribute;
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

    // Repeat for children
    entity.childrenIds.forEach((childId) => this.saveTransform(childId));
  }

  #createColliderVisual(entityId: string) {
    const entity = this.#scene.entities[entityId];

    // Remove previous collider
    this.#removeColliderVisual(entityId);

    // Create new collider
    let collider: Mesh | null = null;
    switch (entity.collider?.type) {
      case "Box":
        collider = new Mesh(
          new BoxBufferGeometry(...entity.collider.size),
          this.#wireframeMaterial
        );
        break;
      case "Sphere":
        collider = new Mesh(
          new SphereBufferGeometry(entity.collider.radius),
          this.#wireframeMaterial
        );
        break;
      case "Cylinder":
        collider = new Mesh(
          new CylinderBufferGeometry(
            entity.collider.radius,
            entity.collider.radius,
            entity.collider.height
          ),
          this.#wireframeMaterial
        );
        break;
    }

    if (collider) {
      const object = this.#objects.get(entityId);
      if (!object) throw new Error("Object not found");

      // Add new collider
      this.#colliders.set(entityId, collider);
      this.visuals.add(collider);

      // Subscribe to global transform changes
      entity.globalPosition$.subscribe({
        next: (position) => {
          if (collider) collider.position.fromArray(position);
        },
      });

      entity.globalQuaternion$.subscribe({
        next: (quaternion) => {
          if (collider) collider.quaternion.fromArray(quaternion);
        },
      });
    }
  }

  #removeColliderVisual(entityId: string) {
    const collider = this.#colliders.get(entityId);
    if (!collider) return;

    this.#colliders.delete(entityId);
    collider.removeFromParent();
    collider.geometry.dispose();
  }

  #updateGlobalTransform(entityId: string) {
    const object = this.#objects.get(entityId);
    if (!object) throw new Error("Object not found");

    const globalPosition = object.getWorldPosition(this.#tempVector3);
    const globalQuaternion = object.getWorldQuaternion(this.#tempQuaternion);

    this.#scene.updateGlobalTransform(
      entityId,
      globalPosition.toArray(),
      globalQuaternion.toArray() as Quad
    );
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
