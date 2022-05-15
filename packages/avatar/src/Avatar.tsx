import { useFBX } from "@react-three/drei";
import { useFrame } from "@react-three/fiber";
import { MutableRefObject, Suspense, useEffect, useState } from "react";
import { AnimationAction, AnimationMixer } from "three";

import { useMixamoAnimation } from "./useMixamoAnimation";
import { useVRM } from "./useVRM";

type Animations = {
  idle: AnimationAction;
  walk: AnimationAction;
  run: AnimationAction;
  jump: AnimationAction;
};

export type AnimationWeights = Record<
  keyof Animations,
  MutableRefObject<number>
>;

interface Props {
  src: string;
  animationsSrc: string;
  animationWeights: AnimationWeights;
}

export function Avatar({ src, animationsSrc, animationWeights }: Props) {
  const vrm = useVRM(src);
  const fbx = useFBX(animationsSrc);

  useMixamoAnimation(fbx, vrm);

  const [animations, setAnimations] = useState<Animations>();
  const [mixer, setMixer] = useState<AnimationMixer>();

  useEffect(() => {
    if (!fbx) return;

    const scale = 0.01;
    fbx.scale.set(scale, scale, scale);
    fbx.rotation.y = Math.PI;

    const newMixer = new AnimationMixer(fbx);
    setMixer(newMixer);

    const idle = newMixer.clipAction(fbx.animations[0]);
    const run = newMixer.clipAction(fbx.animations[1]);
    const walk = newMixer.clipAction(fbx.animations[2]);
    const jump = newMixer.clipAction(fbx.animations[3]);

    idle.play();
    walk.play();
    jump.play();

    const obj: Animations = {
      idle,
      walk,
      run,
      jump,
    };
    setAnimations(obj);
  }, [fbx]);

  useFrame((_, delta) => {
    if (!mixer) return;
    mixer.update(delta);

    if (animations) {
      animations.jump.weight = animationWeights.jump.current;
      animations.walk.weight = animationWeights.walk.current;
      animations.run.weight = animationWeights.walk.current;
      animations.idle.weight = animationWeights.idle.current;
    }
  });

  if (!vrm?.scene) return null;

  return (
    <Suspense fallback={null}>
      <primitive object={vrm.scene} />
    </Suspense>
  );
}
