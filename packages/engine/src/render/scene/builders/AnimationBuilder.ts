import {
  AnimationClip,
  InterpolateDiscrete,
  InterpolateLinear,
  NumberKeyframeTrack,
  QuaternionKeyframeTrack,
  VectorKeyframeTrack,
} from "three";

import { AnimationJSON } from "../../../scene";
import { subscribe } from "../../../utils/subscribe";
import { RenderScene } from "../RenderScene";
import {
  GLTFCubicSplineInterpolant,
  GLTFCubicSplineQuaternionInterpolant,
} from "../utils/CubicSplineInterpolation";
import { Builder } from "./Builder";

/**
 * @internal
 * Handles the conversion of animations to Three.js objects.
 */
export class AnimationBuilder extends Builder<AnimationJSON, AnimationClip> {
  constructor(scene: RenderScene) {
    super(scene);
  }

  add(json: Partial<AnimationJSON>, id: string) {
    const previousObject = this.getObject(id);
    if (previousObject) throw new Error(`Animation with id ${id} already exists.`);

    const { object: animation } = this.scene.animation.create(json, id);

    const object = new AnimationClip(undefined, undefined, []);
    this.setObject(id, object);

    subscribe(animation, "Name", (value) => {
      object.name = value;
    });

    subscribe(animation, "Channels", (channels) => {
      channels.forEach((channel) => {
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

        const targetid = this.scene.node.getId(target);
        if (!targetid) return;

        const targetObject = this.scene.builders.node.getObject(targetid);
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

          object.tracks.push(track);

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

              return new InterpolantType(this.times, this.values, this.getValueSize() / 3, result);
            };
            // @ts-ignore
            track.createInterpolant.isInterpolantFactoryMethodGLTFCubicSpline = true;
          }
        });
      });

      return () => {
        object.tracks = [];
      };
    });

    animation.addEventListener("dispose", () => {
      this.setObject(id, null);
    });

    return animation;
  }

  remove(id: string) {
    this.scene.animation.store.get(id)?.dispose();
  }

  update(json: Partial<AnimationJSON>, id: string) {
    const animation = this.scene.animation.store.get(id);
    if (!animation) throw new Error(`Animation with id ${id} does not exist.`);

    this.scene.animation.applyJSON(animation, json);
  }
}
