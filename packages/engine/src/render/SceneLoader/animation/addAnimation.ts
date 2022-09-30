import {
  AnimationClip,
  AnimationMixer,
  InterpolateDiscrete,
  InterpolateLinear,
  NumberKeyframeTrack,
  QuaternionKeyframeTrack,
  VectorKeyframeTrack,
} from "three";

import { AnimationJSON } from "../../../scene";
import { SceneMap } from "../types";
import {
  GLTFCubicSplineInterpolant,
  GLTFCubicSplineQuaternionInterpolant,
} from "./CubicSplineInterpolation";

export function addAnimation(
  animation: AnimationJSON,
  map: SceneMap,
  mixer: AnimationMixer
) {
  const tracks: Array<
    NumberKeyframeTrack | VectorKeyframeTrack | QuaternionKeyframeTrack
  > = [];

  animation.channels.forEach((channel) => {
    // Get keyframe track type
    let threePath: string;
    let TypedKeyframeTrack:
      | typeof NumberKeyframeTrack
      | typeof VectorKeyframeTrack
      | typeof QuaternionKeyframeTrack;
    switch (channel.path) {
      case "weights":
        TypedKeyframeTrack = NumberKeyframeTrack;
        threePath = "morphTargetInfluences";
        break;
      case "rotation":
        TypedKeyframeTrack = QuaternionKeyframeTrack;
        threePath = "quaternion";
        break;
      case "translation":
        TypedKeyframeTrack = VectorKeyframeTrack;
        threePath = "position";
        break;
      case "scale":
        TypedKeyframeTrack = VectorKeyframeTrack;
        threePath = "scale";
        break;
      default:
        throw new Error(`Unknown target path: ${channel.path}`);
    }

    // Get interpolation type
    let interpolationMode:
      | typeof InterpolateLinear
      | typeof InterpolateDiscrete
      | undefined;
    switch (channel.sampler.interpolation) {
      case "LINEAR":
        interpolationMode = InterpolateLinear;
        break;
      case "STEP":
        interpolationMode = InterpolateDiscrete;
        break;
      case "CUBICSPLINE":
        interpolationMode = undefined;
        break;
      default:
        throw new Error(
          `Unknown interpolation: ${channel.sampler.interpolation}`
        );
    }

    // Get input and output data
    const inputId = channel.sampler.inputId;
    const outputId = channel.sampler.outputId;
    const input = map.accessors.get(inputId);
    const output = map.accessors.get(outputId);
    if (!input || !output) throw new Error("Accessor not found");
    const inputArray = Array.from(input.array);
    const outputArray = Array.from(output.array);

    // Get target object ids
    const target = map.objects.get(channel.targetId);
    if (!target) throw new Error(`Target not found: ${channel.targetId}`);

    const targetIds = [];
    if (channel.path === "weights") {
      target.traverse((child) => {
        if ("morphTargetInfluences" in child) targetIds.push(child.uuid);
      });
    } else {
      targetIds.push(target.uuid);
    }

    targetIds.forEach((targetId) => {
      // Create keyframe track
      const track = new TypedKeyframeTrack(
        `${targetId}.${threePath}`,
        inputArray,
        outputArray,
        interpolationMode
      );

      // Create a custom interpolant for cubic spline interpolation
      // The built in three.js cubic interpolant is not compatible with the glTF spec
      if (channel.sampler.interpolation === "CUBICSPLINE") {
        // @ts-ignore
        track.createInterpolant =
          function InterpolantFactoryMethodGLTFCubicSpline(result: any) {
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
        track.createInterpolant.isInterpolantFactoryMethodGLTFCubicSpline =
          true;
      }

      // Add keyframe track to animation clip
      tracks.push(track);
    });
  });

  const clip = new AnimationClip(animation.name, undefined, tracks);
  mixer.clipAction(clip).play();

  map.animations.set(animation.id, clip);
}
