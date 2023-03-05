import { Accessor, Mesh, Node, Primitive, Skin, Texture, TextureInfo } from "@gltf-transform/core";
import {
  AnimationAction,
  AnimationClip,
  AnimationMixer,
  Bone,
  BufferAttribute,
  DoubleSide,
  FrontSide,
  InterpolateDiscrete,
  InterpolateLinear,
  KeyframeTrack,
  Mesh as ThreeMesh,
  MeshStandardMaterial,
  NumberKeyframeTrack,
  Object3D,
  QuaternionKeyframeTrack,
  Skeleton,
  SkinnedMesh,
  sRGBEncoding,
  VectorKeyframeTrack,
} from "three";
import { CSM } from "three/examples/jsm/csm/CSM";

import { AccessorJSON, MeshJSON, NodeJSON } from "../../scene";
import { MaterialJSON } from "../../scene/attributes/Materials";
import { PrimitiveJSON } from "../../scene/attributes/Primitives";
import { TextureInfoJSON, TextureInfoUtils } from "../../scene/attributes/TextureInfoUtils";
import { SceneMessage } from "../../scene/messages";
import { Scene } from "../../scene/Scene";
import { deepDispose } from "../utils/deepDispose";
import { THREE_ATTRIBUTE_NAMES } from "./constants";
import { createTexture } from "./createTexture";
import {
  GLTFCubicSplineInterpolant,
  GLTFCubicSplineQuaternionInterpolant,
} from "./CubicSplineInterpolation";
import { getCustomMeshData } from "./getCustomMeshData";

type NodeId = string;
type MeshId = string;
type PrimitiveId = string;
type MaterialId = string;
type SkinId = string;

/**
 * Instants of a {@link Scene} for the render thread.
 * This class is responsible for creating and disposing Three.js objects.
 */
export class RenderScene extends Scene {
  #csm: CSM | null = null;
  #toMainThread: (message: SceneMessage) => void;

  root = new Object3D();
  mixer = new AnimationMixer(this.root);

  #animationsEnabled = false;

  materialObjects = new Map<MaterialId, MeshStandardMaterial>();
  primitiveObjects = new Map<PrimitiveId, ThreeMesh | SkinnedMesh>();
  meshObjects = new Map<MeshId, Object3D>();
  instancedMeshObjects = new Map<NodeId, Object3D>();
  nodeObjects = new Map<NodeId, Object3D | Bone>();
  skeletonObjects = new Map<SkinId, Skeleton>();
  animationObjects = new Map<NodeId, AnimationAction>();

  static DEFAULT_MATERIAL = new MeshStandardMaterial();

  constructor(toMainThread: (message: SceneMessage) => void) {
    super();

    this.#toMainThread = toMainThread;
  }

  get csm() {
    return this.#csm;
  }

  set csm(csm: CSM | null) {
    this.#csm?.dispose();
    this.#csm = csm;

    for (const material of this.materialObjects.values()) {
      csm?.setupMaterial(material);
    }
  }

  onmessage({ subject, data }: SceneMessage) {
    switch (subject) {
      case "create_buffer": {
        this.buffer.create(data.json, data.id);
        break;
      }

      case "dispose_buffer": {
        const buffer = this.buffer.store.get(data);
        if (buffer) buffer.dispose();
        break;
      }

      case "create_accessor": {
        this.createAccessor(data.json, data.id);
        break;
      }

      case "dispose_accessor": {
        const accessor = this.accessor.store.get(data);
        if (accessor) accessor.dispose();
        break;
      }

      case "create_texture": {
        this.texture.create(data.json, data.id);
        break;
      }

      case "dispose_texture": {
        const texture = this.texture.store.get(data);
        if (texture) texture.dispose();
        break;
      }

      case "create_material": {
        const { object: material } = this.material.create(data.json, data.id);

        const object = new MeshStandardMaterial();
        this.materialObjects.set(data.id, object);

        this.csm?.setupMaterial(object);

        material.addEventListener("dispose", () => {
          object.map?.dispose();
          object.normalMap?.dispose();
          object.metalnessMap?.dispose();
          object.roughnessMap?.dispose();
          object.aoMap?.dispose();
          object.emissiveMap?.dispose();
          object.dispose();

          this.materialObjects.delete(data.id);
        });

        this.updateMaterial(data.id, data.json);
        break;
      }

      case "change_material": {
        this.updateMaterial(data.id, data.json);

        const material = this.material.store.get(data.id);
        if (!material) throw new Error("Material not found");
        this.material.applyJSON(material, data.json);
        break;
      }

      case "dispose_material": {
        const material = this.material.store.get(data);
        if (material) material.dispose();
        break;
      }

      case "create_primitive": {
        this.createPrimitive(data.json, data.id);
        break;
      }

      case "change_primitive": {
        this.updatePrimitive(data.id, data.json);

        const primitive = this.primitive.store.get(data.id);
        if (!primitive) throw new Error("Primitive not found");
        this.primitive.applyJSON(primitive, data.json);
        break;
      }

      case "dispose_primitive": {
        const primitive = this.primitive.store.get(data);
        if (primitive) primitive.dispose();
        break;
      }

      case "create_mesh": {
        const { object: mesh } = this.mesh.create(data.json, data.id);

        const object = new Object3D();
        this.meshObjects.set(data.id, object);

        mesh.addEventListener("dispose", () => {
          this.meshObjects.delete(data.id);
        });

        this.updateMesh(data.id, data.json);
        break;
      }

      case "change_mesh": {
        this.updateMesh(data.id, data.json);

        const mesh = this.mesh.store.get(data.id);
        if (!mesh) throw new Error("Mesh not found");
        this.mesh.applyJSON(mesh, data.json);
        break;
      }

      case "dispose_mesh": {
        const mesh = this.mesh.store.get(data);
        if (mesh) mesh.dispose();
        break;
      }

      case "create_node": {
        const { object: node } = this.node.create(data.json, data.id);

        this.createNodeObject(node);
        break;
      }

      case "change_node": {
        this.updateNode(data.id, data.json);
        break;
      }

      case "dispose_node": {
        const node = this.node.store.get(data);
        if (node) node.dispose();
        break;
      }

      case "create_skin": {
        const { object: skin } = this.skin.create(data.json, data.id);
        this.updateSkin(skin);
        break;
      }

      case "dispose_skin": {
        const skin = this.skin.store.get(data);

        if (skin) {
          const joints = skin.listJoints();

          skin.dispose();

          // Update joint objects
          joints.forEach((node) => {
            this.updateNodeObject(node);
          });
        }
        break;
      }

      case "create_animation": {
        this.animation.create(data.json, data.id);

        const animation = this.animation.store.get(data.id);
        if (!animation) throw new Error("Animation not found");

        const tracks: KeyframeTrack[] = [];

        animation.listChannels().forEach((channel) => {
          // Get keyframe track type
          const path = channel.getTargetPath();
          if (!path) return;

          let threePath: string;
          let TypedKeyframeTrack:
            | typeof NumberKeyframeTrack
            | typeof VectorKeyframeTrack
            | typeof QuaternionKeyframeTrack;

          switch (path) {
            case "weights": {
              TypedKeyframeTrack = NumberKeyframeTrack;
              threePath = "morphTargetInfluences";
              break;
            }

            case "rotation": {
              TypedKeyframeTrack = QuaternionKeyframeTrack;
              threePath = "quaternion";
              break;
            }

            case "translation": {
              TypedKeyframeTrack = VectorKeyframeTrack;
              threePath = "position";
              break;
            }

            case "scale": {
              TypedKeyframeTrack = VectorKeyframeTrack;
              threePath = "scale";
              break;
            }

            default:
              throw new Error(`Unknown target path: ${path}`);
          }

          // Get interpolation mode
          const sampler = channel.getSampler();
          if (!sampler) return;

          let interpolationMode: typeof InterpolateLinear | typeof InterpolateDiscrete | undefined;

          switch (sampler.getInterpolation()) {
            case "LINEAR": {
              interpolationMode = InterpolateLinear;
              break;
            }

            case "STEP": {
              interpolationMode = InterpolateDiscrete;
              break;
            }

            case "CUBICSPLINE": {
              interpolationMode = undefined;
              break;
            }

            default:
              throw new Error("Unknown interpolation");
          }

          // Get target object ids
          const target = channel.getTargetNode();
          if (!target) return;

          const targetid = this.node.getId(target);
          if (!targetid) return;

          const targetObject = this.nodeObjects.get(targetid);
          if (!targetObject) return;

          const targetIds = [];
          if (path === "weights") {
            targetObject.traverse((child) => {
              if ("morphTargetInfluences" in child) targetIds.push(child.uuid);
            });
          } else {
            targetIds.push(targetObject.uuid);
          }

          // Get input and output data
          const inputAccessor = sampler.getInput();
          if (!inputAccessor) return;

          const outputAccessor = sampler.getOutput();
          if (!outputAccessor) return;

          const inputArray = inputAccessor.getArray();
          if (!inputArray) return;

          const outputArray = outputAccessor.getArray();
          if (!outputArray) return;

          // Create keyframe track
          targetIds.forEach((id) => {
            const track = new TypedKeyframeTrack(
              `${id}.${threePath}`,
              inputArray,
              outputArray,
              interpolationMode
            );

            tracks.push(track);

            // Create a custom interpolant for cubic spline interpolation
            // The built in three.js cubic interpolant is not compatible with the glTF spec
            if (sampler.getInterpolation() === "CUBICSPLINE") {
              // @ts-ignore
              track.createInterpolant = function InterpolantFactoryMethodGLTFCubicSpline(
                result: any
              ) {
                // A CUBICSPLINE keyframe in glTF has three output values for each input value,
                // representing inTangent, splineVertex, and outTangent. As a result, track.getValueSize()
                // must be divided by three to get the interpolant's sampleSize argument.
                const InterpolantType =
                  this instanceof QuaternionKeyframeTrack
                    ? GLTFCubicSplineQuaternionInterpolant
                    : GLTFCubicSplineInterpolant;

                return new InterpolantType(
                  this.times,
                  this.values,
                  this.getValueSize() / 3,
                  result
                );
              };
              // @ts-ignore
              track.createInterpolant.isInterpolantFactoryMethodGLTFCubicSpline = true;
            }
          });
        });

        const clip = new AnimationClip(animation.getName(), undefined, tracks);

        const action = this.mixer.clipAction(clip);
        if (this.#animationsEnabled) action.play();

        this.animationObjects.set(data.id, action);
        break;
      }

      case "change_animation": {
        const animation = this.animation.store.get(data.id);
        if (!animation) throw new Error("Animation not found");
        this.animation.applyJSON(animation, data.json);
        break;
      }

      case "dispose_animation": {
        const animation = this.animation.store.get(data);
        if (animation) animation.dispose();
        break;
      }
    }
  }

  createAccessor(json: Partial<AccessorJSON>, accessorId?: string) {
    if (accessorId) {
      const accessor = this.accessor.store.get(accessorId);
      if (accessor) return { id: accessorId, accessor };
    }

    const { id, object: accessor } = this.accessor.create(json, accessorId);

    if (!accessorId) {
      const fullJSON = this.accessor.toJSON(accessor);
      this.#toMainThread({ subject: "create_accessor", data: { id, json: fullJSON } });
    }

    return { id, accessor };
  }

  updateNode(id: string, json: Partial<NodeJSON>) {
    const node = this.node.store.get(id);
    if (!node) throw new Error("Node not found");

    const object = this.nodeObjects.get(id);
    if (!object) throw new Error("Object not found");

    if (json.name) object.name = json.name;

    if (json.translation) object.position.fromArray(json.translation);
    if (json.rotation) object.quaternion.fromArray(json.rotation);
    if (json.scale) object.scale.fromArray(json.scale);

    let oldMesh: Mesh | null = null;
    let newMesh: Mesh | null = null;

    if (json.mesh !== undefined) {
      oldMesh = node.getMesh();

      // Remove old mesh
      object.children.forEach((child) => {
        const id = this.getMeshId(child) ?? this.getInstancedMeshNodeId(child);
        const isMesh = id !== null;
        if (isMesh) object.remove(child);
      });

      // Add new mesh
      if (json.mesh) {
        newMesh = this.mesh.store.get(json.mesh) ?? null;
        if (!newMesh) throw new Error("Mesh not found");

        const meshObject = this.meshObjects.get(json.mesh);
        if (!meshObject) throw new Error("Mesh object not found");

        const weights = node.getWeights();

        const instanceCount = newMesh.listParents().filter((p) => p instanceof Node).length;
        const shouldClone = instanceCount > 1 || weights.length > 0;

        if (shouldClone) {
          const instance = meshObject.clone();

          // Apply weights
          weights.forEach((weight, i) => {
            instance.traverse((child) => {
              if (child instanceof ThreeMesh) {
                if ("morphTargetInfluences" in child) {
                  if (!child.morphTargetInfluences) return;
                  child.morphTargetInfluences[i] = weight;
                }
              }
            });
          });

          this.instancedMeshObjects.set(id, instance);
          object.add(instance);
        } else {
          object.add(meshObject);
        }
      }
    }

    node.setMesh(newMesh);

    if (json.skin !== undefined) {
      const oldSkin = node.getSkin();

      // Remove old skin
      node.setSkin(null);

      const newSkin = json.skin ? this.skin.store.get(json.skin) : null;
      if (newSkin === undefined) throw new Error("Skin not found");

      // Add new skin
      node.setSkin(newSkin);

      // Update skin
      if (oldSkin) this.updateSkin(oldSkin);
      if (newSkin) this.updateSkin(newSkin);
    }

    if (json.children) {
      // Remove old children
      object.children.forEach((child) => {
        const nodeId = this.getNodeId(child);
        const isNode = nodeId !== null;
        if (isNode) {
          object.remove(child);
          this.root.add(child);
        }
      });

      // Add new children
      json.children.forEach((childId) => {
        const nodeObject = this.nodeObjects.get(childId);
        if (!nodeObject) throw new Error("Node object not found");

        object.add(nodeObject);
      });
    }

    // Apply node JSON
    this.node.applyJSON(node, json);

    // Update primitive objects if mesh changed
    if (oldMesh) {
      oldMesh.listPrimitives().forEach((primitive) => this.updatePrimitiveObject(primitive));
    }

    if (newMesh) {
      newMesh.listPrimitives().forEach((primitive) => this.updatePrimitiveObject(primitive));
    }
  }

  #isNodeJoint(node: Node) {
    return node.listParents().some((parent) => parent instanceof Skin);
  }

  updateNodeObject(node: Node) {
    const id = this.node.getId(node);
    if (!id) throw new Error("Node not found");

    const previousObject = this.nodeObjects.get(id);

    if (previousObject) {
      // Check if node is a joint
      const isJoint = this.#isNodeJoint(node);

      if (isJoint !== previousObject instanceof Bone) {
        this.createNodeObject(node);
      }
    } else {
      this.createNodeObject(node);
    }
  }

  createNodeObject(node: Node) {
    const id = this.node.getId(node);
    if (!id) throw new Error("Node not found");

    const previousObject = this.nodeObjects.get(id);

    const parent = previousObject?.parent;
    const children = previousObject?.children;

    // Remove previous object
    if (previousObject) {
      previousObject.removeFromParent();
      this.nodeObjects.delete(id);
    }

    // Create new object
    const isJoint = this.#isNodeJoint(node);
    const object = isJoint ? new Bone() : new Object3D();
    object.name = node.getName();

    this.root.add(object);

    // Add previous parent and children
    if (parent) parent.add(object);
    if (children) children.forEach((child) => object.add(child));

    this.nodeObjects.set(id, object);

    node.addEventListener("dispose", () => {
      this.nodeObjects.delete(id);
    });
  }

  updateSkin(skin: Skin) {
    const id = this.skin.getId(skin);
    if (!id) throw new Error("Skin not found");

    // Update joint objects
    skin.listJoints().forEach((node) => {
      this.updateNodeObject(node);
    });

    // Update skinned mesh objects
    const parents = skin.listParents().filter((p): p is Node => p instanceof Node);

    parents.forEach((node) => {
      const mesh = node.getMesh();
      if (!mesh) return;

      mesh.listPrimitives().forEach((primitive) => {
        this.updatePrimitiveObject(primitive);
      });
    });

    // Get bones
    const bones = skin.listJoints().map((node) => {
      const id = this.node.getId(node);
      if (!id) throw new Error("Node id not found");

      const object = this.nodeObjects.get(id);
      if (!object) throw new Error("Node object not found");
      if (!(object instanceof Bone)) throw new Error("Node object is not a bone");

      return object;
    });

    // Apply inverse bind matrices
    const inverseBindMatrices = skin.getInverseBindMatrices();
    if (inverseBindMatrices) {
      const array = inverseBindMatrices.getArray();
      if (array) {
        bones.forEach((bone, i) => {
          bone.matrix.fromArray(array, i * 16);
        });
      }
    }

    // Remove old skeleton
    const oldSkeleton = this.skeletonObjects.get(id);
    if (oldSkeleton) {
      oldSkeleton.dispose();
      this.skeletonObjects.delete(id);
    }

    // Create skeleton
    const skeleton = new Skeleton(bones);
    this.skeletonObjects.set(id, skeleton);

    // Bind skeleton to mesh
    parents.forEach((node) => {
      const mesh = node.getMesh();
      if (!mesh) return;

      mesh.listPrimitives().forEach((primitive) => {
        const id = this.primitive.getId(primitive);
        if (!id) throw new Error("Primitive id not found");

        const object = this.primitiveObjects.get(id);
        if (!object) throw new Error("Primitive object not found");
        if (!(object instanceof SkinnedMesh)) return;

        object.bind(skeleton, object.matrixWorld);
      });
    });

    skin.addEventListener("dispose", () => {
      skeleton.dispose();
    });
  }

  updateMesh(id: string, json: Partial<MeshJSON>) {
    const mesh = this.mesh.store.get(id);
    if (!mesh) throw new Error("Mesh not found");

    const object = this.meshObjects.get(id);
    if (!object) throw new Error("Mesh object not found");

    if (json.primitives) {
      const oldPrimitives = mesh.listPrimitives();

      // Remove previous primitives
      oldPrimitives.forEach((primitive) => {
        const primitiveId = this.primitive.getId(primitive);
        if (!primitiveId) throw new Error("Primitive not found");

        const primitiveObject = this.primitiveObjects.get(primitiveId);
        if (!primitiveObject) throw new Error("Primitive object not found");

        mesh.removePrimitive(primitive);
        object.remove(primitiveObject);
      });

      // Add new primitives
      json.primitives.forEach((primitiveId) => {
        const primitive = this.primitive.store.get(primitiveId);
        if (!primitive) throw new Error("Primitive not found");

        mesh.addPrimitive(primitive);

        const primitiveObject = this.primitiveObjects.get(primitiveId);
        if (!primitiveObject) throw new Error("Primitive object not found");

        // Apply weights
        mesh.getWeights().forEach((weight, i) => {
          if (!primitiveObject.morphTargetInfluences) return;
          primitiveObject.morphTargetInfluences[i] = weight;
        });

        object.add(primitiveObject);
      });

      // Update both new and old primitive objects
      oldPrimitives.forEach((primitive) => this.updatePrimitiveObject(primitive));
      mesh.listPrimitives().forEach((primitive) => this.updatePrimitiveObject(primitive));
    }

    if (json.extras) {
      if (json.extras.customMesh) {
        // Remove old primitives
        mesh.listPrimitives().forEach((primitive) => {
          mesh.removePrimitive(primitive);

          const primitiveId = this.primitive.getId(primitive);
          if (!primitiveId) throw new Error("Primitive not found");

          const indices = primitive.getIndices();
          if (indices) {
            const isIndicesUsed =
              indices.listParents().filter((parent) => parent instanceof Primitive).length > 1;

            const indicesId = this.accessor.getId(indices);
            if (!indicesId) throw new Error("Accessor not found");

            if (!isIndicesUsed)
              this.#toMainThread({ subject: "dispose_accessor", data: indicesId });
          }

          primitive.listAttributes().forEach((attribute) => {
            const isAttributeUsed =
              attribute.listParents().filter((parent) => parent instanceof Primitive).length > 1;

            const attributeId = this.accessor.getId(attribute);
            if (!attributeId) throw new Error("Accessor not found");

            if (!isAttributeUsed)
              this.#toMainThread({ subject: "dispose_accessor", data: attributeId });
          });

          this.#toMainThread({ subject: "dispose_primitive", data: primitiveId });
        });

        // Create acessors
        const { positions, normals, indices } = getCustomMeshData(json.extras.customMesh);

        const { id: positionsId } = this.createAccessor({
          array: new Float32Array(positions),
          type: "VEC3",
          componentType: 5126,
        });

        const { id: normalsId } = this.createAccessor({
          array: new Float32Array(normals),
          type: "VEC3",
          componentType: 5126,
        });

        const { id: indicesId } = this.createAccessor({
          array: new Uint16Array(indices),
          type: "SCALAR",
          componentType: 5121,
        });

        // Create new primitive
        const { id: primitiveId, primitive } = this.createPrimitive({
          attributes: { POSITION: positionsId, NORMAL: normalsId },
          indices: indicesId,
        });

        // Add to scene
        const primitiveObject = this.primitiveObjects.get(primitiveId);
        if (!primitiveObject) throw new Error("Primitive object not found");

        mesh.addPrimitive(primitive);

        // Add primitive to mesh
        this.#toMainThread({
          subject: "change_mesh",
          data: { id, json: { primitives: [primitiveId] } },
        });
      }
    }
  }

  createPrimitive(json: Partial<PrimitiveJSON>, primitiveId?: string) {
    if (primitiveId) {
      const primitive = this.primitive.store.get(primitiveId);
      if (primitive) return { id: primitiveId, primitive };
    }

    const { id, object: primitive } = this.primitive.create(json, primitiveId);
    const fullJSON = this.primitive.toJSON(primitive);

    if (!primitiveId) {
      // Update main thread if primitive is new
      this.#toMainThread({ subject: "create_primitive", data: { id, json: fullJSON } });
    }

    this.updatePrimitiveObject(primitive);

    this.updatePrimitive(id, fullJSON);

    return { id, primitive };
  }

  #isPrimitiveSkinnedMesh(primitive: Primitive) {
    return primitive
      .listParents()
      .filter((parent): parent is Mesh => parent instanceof Mesh)
      .flatMap((mesh) =>
        mesh.listParents().filter((parent): parent is Node => parent instanceof Node)
      )
      .some((node) => node.getSkin() !== null);
  }

  updatePrimitiveObject(primitive: Primitive) {
    const id = this.primitive.getId(primitive);
    if (!id) throw new Error("Primitive id not found");

    const previousObject = this.primitiveObjects.get(id);

    if (previousObject) {
      // Check if primitive is skinned mesh
      const isSkinnedMesh = this.#isPrimitiveSkinnedMesh(primitive);

      if (isSkinnedMesh !== previousObject instanceof SkinnedMesh) {
        this.createPrimitiveObject(primitive);
      }
    } else {
      this.createPrimitiveObject(primitive);
    }
  }

  createPrimitiveObject(primitive: Primitive) {
    const id = this.primitive.getId(primitive);
    if (!id) throw new Error("Primitive id not found");

    const previousObject = this.primitiveObjects.get(id);

    const parent = previousObject?.parent;
    const children = previousObject?.children;
    const geometry = previousObject?.geometry;
    const material = previousObject?.material;

    // Remove previous object
    if (previousObject) {
      previousObject.removeFromParent();
      this.primitiveObjects.delete(id);
    }

    // Create new object
    const isSkinnedMesh = this.#isPrimitiveSkinnedMesh(primitive);
    const object = isSkinnedMesh
      ? new SkinnedMesh(geometry, material)
      : new ThreeMesh(geometry, material);

    if (object instanceof SkinnedMesh) {
      const normalized = object.geometry.attributes.skinWeight.normalized;
      if (!normalized) object.normalizeSkinWeights();
    }

    object.name = primitive.getName();
    object.geometry.morphTargetsRelative = true;
    object.castShadow = true;
    object.receiveShadow = true;

    // Add previous parent and children
    if (parent) parent.add(object);
    if (children) children.forEach((child) => object.add(child));

    this.primitiveObjects.set(id, object);

    primitive.addEventListener("dispose", () => {
      object.removeFromParent();
      object.geometry.dispose();
      this.primitiveObjects.delete(id);
    });
  }

  updatePrimitive(id: string, json: Partial<PrimitiveJSON>) {
    const primitive = this.primitive.store.get(id);
    if (!primitive) throw new Error(`Primitive not found ${id}`);

    const object = this.primitiveObjects.get(id);
    if (!object) throw new Error("Primitive object not found");

    if (json.material !== undefined) {
      // Remove previous material
      object.material = RenderScene.DEFAULT_MATERIAL;

      // Add new material
      if (json.material) {
        const materialObject = this.materialObjects.get(json.material);
        if (!materialObject) throw new Error("Material not found");

        object.material = materialObject;
      }
    }

    if (json.indices !== undefined) {
      if (!json.indices) {
        object.geometry.setIndex(null);
      } else {
        const accessor = this.accessor.store.get(json.indices);
        if (!accessor) throw new Error("Accessor not found");

        const attribute = accessorToAttribute(accessor);
        object.geometry.setIndex(attribute);
      }
    }

    if (json.attributes) {
      // Remove previous attributes
      Object.values(THREE_ATTRIBUTE_NAMES).forEach((name) => {
        object.geometry.deleteAttribute(name);
      });

      // Add new attributes
      Object.entries(json.attributes).forEach(([name, accessorId]) => {
        const threeName = THREE_ATTRIBUTE_NAMES[name as keyof typeof THREE_ATTRIBUTE_NAMES];

        if (accessorId === null) {
          object.geometry.deleteAttribute(threeName);
        } else {
          const accessor = this.accessor.store.get(accessorId);
          if (!accessor) throw new Error("Accessor not found");

          const attribute = accessorToAttribute(accessor);

          if (!attribute) object.geometry.deleteAttribute(threeName);
          else object.geometry.setAttribute(threeName, attribute);
        }
      });
    }

    if (json.targets) {
      // Reset morph influences
      if (json.targets.length === 0) object.morphTargetInfluences = undefined;
      else object.morphTargetInfluences = [];

      // Remove previous morph attributes
      Object.values(THREE_ATTRIBUTE_NAMES).forEach((name) => {
        delete object.geometry.morphAttributes[name];
      });

      // Add new morph attributes
      json.targets.forEach((target) => {
        Object.entries(target.attributes).forEach(([name, accessorId]) => {
          const threeName = THREE_ATTRIBUTE_NAMES[name as keyof typeof THREE_ATTRIBUTE_NAMES];

          if (!accessorId) {
            object.geometry.deleteAttribute(threeName);
          } else {
            const accessor = this.accessor.store.get(accessorId);
            if (!accessor) throw new Error("Accessor not found");

            const attribute = accessorToAttribute(accessor);

            if (!attribute) delete object.geometry.morphAttributes[threeName];
            else {
              const morphAttributes = object.geometry.morphAttributes[threeName];
              if (!morphAttributes) object.geometry.morphAttributes[threeName] = [attribute];
              else morphAttributes.push(attribute);
            }
          }
        });
      });
    }
  }

  async updateMaterial(id: string, json: Partial<MaterialJSON>) {
    const material = this.material.store.get(id);
    if (!material) throw new Error("Material not found");

    const object = this.materialObjects.get(id);
    if (!object) throw new Error("Material object not found");

    if (json.doubleSided !== undefined) {
      object.side = json.doubleSided ? DoubleSide : FrontSide;
    }

    if (json.baseColorFactor) {
      object.color.fromArray(json.baseColorFactor);
    }

    const getTextureData = (
      textureId: string | null | undefined,
      textureInfo: TextureInfoJSON | null | undefined,
      currentTexture: Texture | null,
      currentInfo: TextureInfo | null
    ) => {
      if (!material) throw new Error("Material not found");

      const currentInfoJSON = currentInfo ? TextureInfoUtils.toJSON(currentInfo) : null;
      let texture: Texture | null = currentTexture;
      let info: TextureInfoJSON | null = currentInfoJSON;

      if (textureId !== undefined) {
        if (textureId === null) texture = null;
        else {
          const foundTexture = this.texture.store.get(textureId);
          if (!foundTexture) throw new Error("Texture not found");
          texture = foundTexture;
        }
      }

      if (textureInfo !== undefined) info = textureInfo;

      return { texture, info };
    };

    if (json.baseColorTexture !== undefined || json.baseColorTextureInfo !== undefined) {
      // Remove previous texture
      if (object.map) object.map.dispose();

      // Create new texture
      const { texture, info } = getTextureData(
        json.baseColorTexture,
        json.baseColorTextureInfo,
        material.getBaseColorTexture(),
        material.getBaseColorTextureInfo()
      );

      const textureObject = await createTexture(texture, info);
      object.map = textureObject;

      if (textureObject) textureObject.encoding = sRGBEncoding;
    }

    if (
      json.metallicRoughnessTexture !== undefined ||
      json.metallicRoughnessTextureInfo !== undefined
    ) {
      // Remove previous texture
      if (object.metalnessMap) object.metalnessMap.dispose();

      // Create new texture
      const { texture, info } = getTextureData(
        json.metallicRoughnessTexture,
        json.metallicRoughnessTextureInfo,
        material.getMetallicRoughnessTexture(),
        material.getMetallicRoughnessTextureInfo()
      );

      const textureObject = await createTexture(texture, info);
      object.metalnessMap = textureObject;
      object.roughnessMap = textureObject;
    }

    if (json.metallicFactor !== undefined) {
      object.metalness = json.metallicFactor;
    }

    if (json.roughnessFactor !== undefined) {
      object.roughness = json.roughnessFactor;
    }

    if (json.normalTexture !== undefined || json.normalTextureInfo !== undefined) {
      // Remove previous texture
      if (object.normalMap) object.normalMap.dispose();

      // Create new texture
      const { texture, info } = getTextureData(
        json.normalTexture,
        json.normalTextureInfo,
        material.getNormalTexture(),
        material.getNormalTextureInfo()
      );

      const textureObject = await createTexture(texture, info);
      object.normalMap = textureObject;
    }

    if (json.normalScale !== undefined) {
      object.normalScale.set(json.normalScale, json.normalScale);
    }

    if (json.occlusionTexture !== undefined || json.occlusionTextureInfo !== undefined) {
      // Remove previous texture
      if (object.aoMap) object.aoMap.dispose();

      // Create new texture
      const { texture, info } = getTextureData(
        json.occlusionTexture,
        json.occlusionTextureInfo,
        material.getOcclusionTexture(),
        material.getOcclusionTextureInfo()
      );

      const textureObject = await createTexture(texture, info);
      object.aoMap = textureObject;
    }

    if (json.occlusionStrength !== undefined) {
      object.aoMapIntensity = json.occlusionStrength;
    }

    if (json.emissiveFactor !== undefined) {
      object.emissive.fromArray(json.emissiveFactor);
    }

    if (json.emissiveTexture !== undefined || json.emissiveTextureInfo !== undefined) {
      // Remove previous texture
      if (object.emissiveMap) object.emissiveMap.dispose();

      // Create new texture
      const { texture, info } = getTextureData(
        json.emissiveTexture,
        json.emissiveTextureInfo,
        material.getEmissiveTexture(),
        material.getEmissiveTextureInfo()
      );

      const textureObject = await createTexture(texture, info);
      object.emissiveMap = textureObject;

      if (textureObject) textureObject.encoding = sRGBEncoding;
    }

    if (json.alphaMode !== undefined || json.alphaCutoff !== undefined) {
      const cutoff = json.alphaCutoff !== undefined ? json.alphaCutoff : material.getAlphaCutoff();

      switch (json.alphaMode) {
        case "OPAQUE": {
          object.transparent = false;
          object.depthWrite = true;
          object.alphaTest = 0;
          break;
        }

        case "MASK": {
          object.transparent = false;
          object.depthWrite = true;
          object.alphaTest = cutoff;
          break;
        }

        case "BLEND": {
          object.transparent = true;
          object.depthWrite = false;
          object.alphaTest = 0;
          break;
        }
      }
    }

    object.needsUpdate = true;
  }

  getInstancedMeshNodeId(object: Object3D): string | null {
    for (const [id, clonedMeshObject] of this.instancedMeshObjects.entries()) {
      let found = false;
      clonedMeshObject.traverse((child) => {
        if (child === object) found = true;
      });
      if (found) return id;
    }

    return null;
  }

  getPrimitiveId(object: Object3D): string | null {
    for (const [id, primitiveObject] of this.primitiveObjects.entries()) {
      if (primitiveObject === object) return id;
    }

    return null;
  }

  getPrimitiveMeshId(primitive: Primitive): string | null {
    for (const [id, mesh] of this.mesh.store.entries()) {
      if (mesh.listPrimitives().includes(primitive)) return id;
    }

    return null;
  }

  getMeshId(object: Object3D): string | null {
    for (const [id, meshObject] of this.meshObjects.entries()) {
      if (meshObject === object) return id;
    }

    return null;
  }

  getMeshNodeId(mesh: Mesh): string | null {
    for (const [id, node] of this.node.store.entries()) {
      if (node.getMesh() === mesh) return id;
    }

    return null;
  }

  getNodeId(object: Object3D): string | null {
    for (const [id, nodeObject] of this.nodeObjects.entries()) {
      if (nodeObject === object) return id;
    }

    return null;
  }

  getObjectNodeId(object: Object3D): string | null {
    // Check primitives
    const primitiveId = this.getPrimitiveId(object);
    if (primitiveId) {
      const primitive = this.primitive.store.get(primitiveId);
      if (!primitive) throw new Error("Primitive not found");

      const meshId = this.getPrimitiveMeshId(primitive);
      if (!meshId) return null;

      const mesh = this.mesh.store.get(meshId);
      if (!mesh) throw new Error("Mesh not found");

      const nodeId = this.getMeshNodeId(mesh);
      if (!nodeId) return null;

      return nodeId;
    }

    // Check cloned meshes
    const clonedMeshId = this.getInstancedMeshNodeId(object);
    if (clonedMeshId) return clonedMeshId;

    return null;
  }

  toggleAnimations(enabled: boolean) {
    this.#animationsEnabled = enabled;

    if (enabled) this.animationObjects.forEach((clip) => clip.play());
    else this.animationObjects.forEach((clip) => clip.reset());
  }

  destroy() {
    this.root.removeFromParent();
    deepDispose(this.root);
  }
}

function accessorToAttribute(accessor: Accessor): BufferAttribute | null {
  const array = accessor.getArray();
  if (!array) return null;

  return new BufferAttribute(array, accessor.getElementSize(), accessor.getNormalized());
}
