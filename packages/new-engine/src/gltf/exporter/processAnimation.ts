import {
  AnimationClip,
  BufferAttribute,
  InterleavedBufferAttribute,
  InterpolateDiscrete,
  InterpolateLinear,
  Object3D,
  SkinnedMesh,
} from "three";

import { PATH_PROPERTIES_REVERSE } from "../constants";
import { Animation, AnimationChannel, AnimationSampler, GLTF } from "../schemaTypes";

export function processAnimation(
  clip: AnimationClip,
  root: Object3D,
  json: GLTF,
  processNode: (node: Object3D) => number,
  processAccessor: (attribute: BufferAttribute | InterleavedBufferAttribute) => number | null
) {
  const tracks = clip.tracks;

  const channels: AnimationChannel[] = [];
  const samplers: AnimationSampler[] = [];

  tracks.forEach((track) => {
    const nodeName = track.name.split(".")[0];
    const pathName = track.name.split(".")[1];
    const path: "scale" | "translation" | "rotation" | "weights" | undefined =
      // @ts-ignore
      PATH_PROPERTIES_REVERSE[pathName];

    if (path === undefined) throw new Error(`Unknown path ${pathName}`);

    let node: Object3D | undefined;
    root.traverse((object) => {
      if (object.uuid === nodeName || object.name === nodeName) {
        node = object;
      }
    });

    if (!node) throw new Error(`Unknown node ${nodeName}`);

    const inputItemSize = 1;
    let outputItemSize = track.values.length / track.times.length;

    if (path === "weights" && "morphTargetInfluences" in node) {
      //@ts-ignore
      outputItemSize /= node.morphTargetInfluences.length;
    }

    // Interpolation
    let interpolation: "LINEAR" | "STEP" | "CUBICSPLINE" | undefined;
    switch (track.getInterpolation()) {
      case InterpolateLinear:
        interpolation = "LINEAR";
        break;
      case InterpolateDiscrete:
        interpolation = "STEP";
        break;
      default:
        // See if there is a custom cubic spline interpolation
        // @ts-ignore
        if (track.createInterpolantisInterpolantFactoryMethodGLTFCubicSpline === true) {
          interpolation = "CUBICSPLINE";
          // itemSize of CUBICSPLINE keyframe is 9
          // (VEC3 * 3: inTangent, splineVertex, and outTangent)
          // but needs to be stored as VEC3 so dividing by 3 here.
          outputItemSize /= 3;
          break;
        }

        // Fallback to LINEAR
        interpolation = "LINEAR";
    }

    // Sampler
    const input = processAccessor(new BufferAttribute(track.times, inputItemSize));
    const output = processAccessor(new BufferAttribute(track.values, outputItemSize));

    if (input === null || output === null) throw new Error("Invalid accessor");

    const sampler: AnimationSampler = {
      input,
      output,
      interpolation,
    };
    const samplerIndex = samplers.push(sampler) - 1;

    // Channel
    const nodeIndex = processNode(node);
    const channel: AnimationChannel = {
      sampler: samplerIndex,
      target: {
        node: nodeIndex,
        path,
      },
    };
    channels.push(channel);
  });

  const animationDef: Animation = {
    channels,
    samplers,
  };

  if (!json.animations) json.animations = [];
  const index = json.animations.push(animationDef) - 1;
  animationDef.name = clip.name || `clip_${json.animations.length}`;
  return index;
}
