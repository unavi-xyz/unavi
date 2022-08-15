import {
  AnimationClip,
  NumberKeyframeTrack,
  Object3D,
  QuaternionKeyframeTrack,
  VectorKeyframeTrack,
} from "three";

import { INTERPOLATION, PATH_PROPERTIES } from "../constants";
import { GLTF } from "../schemaTypes";
import {
  GLTFCubicSplineInterpolant,
  GLTFCubicSplineQuaternionInterpolant,
} from "./CubicSplineInterpolation";
import { getNormalizedComponentScale } from "./getNormalizedComponentScale";
import { AccessorResult } from "./loadAccessor";

export async function loadAnimation(
  index: number,
  json: GLTF,
  loadAccessor: (index: number) => Promise<AccessorResult>,
  loadNode: (index: number) => Promise<Object3D>
): Promise<AnimationClip> {
  if (json.animations === undefined) {
    throw new Error("No animations found");
  }

  const animationDef = json.animations[index];

  // Load channels
  const channelPromises = animationDef.channels.map(async (channel) => {
    const sampler = animationDef.samplers[channel.sampler];

    if (channel.target.node === undefined) {
      throw new Error(`Animation target has no node`);
    }

    const nodePromise = loadNode(channel.target.node);
    const inputPromise = loadAccessor(sampler.input);
    const outputPromise = loadAccessor(sampler.output);

    const [node, input, output] = await Promise.all([nodePromise, inputPromise, outputPromise]);

    if (!output) {
      throw new Error(`Animation output has no output`);
    }

    if (!input) {
      throw new Error(`Animation input has no input`);
    }

    node.updateMatrix();
    node.matrixAutoUpdate = true;

    let outputArray = Array.from(output.array);

    if (output.normalized) {
      const scale = getNormalizedComponentScale(outputArray.constructor as any);
      const scaled = new Float32Array(outputArray.length);

      outputArray.forEach((value, index) => {
        scaled[index] = value * scale;
      });

      outputArray = Array.from(scaled);
    }

    let TypedKeyframeTrack:
      | typeof NumberKeyframeTrack
      | typeof QuaternionKeyframeTrack
      | typeof VectorKeyframeTrack;

    switch (channel.target.path) {
      case "weights":
        TypedKeyframeTrack = NumberKeyframeTrack;
        break;
      case "rotation":
        TypedKeyframeTrack = QuaternionKeyframeTrack;
        break;
      case "translation":
      case "scale":
      default:
        TypedKeyframeTrack = VectorKeyframeTrack;
    }

    const names: string[] = [];

    if (channel.target.path === "weights") {
      node.traverse((child) => {
        if ("morphTargetInfluences" in child) {
          names.push(child.uuid);
        }
      });
    } else {
      names.push(node.uuid);
    }

    const interpolationType = sampler.interpolation ?? "LINEAR";
    const interpolation = INTERPOLATION[interpolationType];

    const channelTracks = names.map((name) => {
      const track = new TypedKeyframeTrack(
        `${name}.${PATH_PROPERTIES[channel.target.path]}`,
        Array.from(input.array),
        outputArray,
        interpolation
      );

      // Override interpolation with custom factory method
      // The built in three.js cubic interpolation does not work with glTF
      if (sampler.interpolation === "CUBICSPLINE") {
        // @ts-ignore
        track.createInterpolant = function InterpolantFactoryMethodGLTFCubicSpline(result) {
          // A CUBICSPLINE keyframe in glTF has three output values for each input value,
          // representing inTangent, splineVertex, and outTangent. As a result, track.getValueSize()
          // must be divided by three to get the interpolant's sampleSize argument.

          const interpolantType =
            this instanceof QuaternionKeyframeTrack
              ? GLTFCubicSplineQuaternionInterpolant
              : GLTFCubicSplineInterpolant;

          return new interpolantType(this.times, this.values, this.getValueSize() / 3, result);
        };

        // Mark as CUBICSPLINE. `track.getInterpolation()` doesn't support custom interpolants.
        // @ts-ignore
        track.createInterpolant.isInterpolantFactoryMethodGLTFCubicSpline = true;
      }

      return track;
    });

    return channelTracks;
  });

  const tracks = await Promise.all(channelPromises);
  const flattened = tracks.flat();

  // Create animation
  const name = animationDef.name ?? `animation_${index}`;
  const animationClip = new AnimationClip(name, undefined, flattened);

  return animationClip;
}
