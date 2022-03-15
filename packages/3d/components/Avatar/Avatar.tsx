import { MutableRefObject, Suspense, useEffect, useState } from "react";
import { useFrame } from "@react-three/fiber";
import { useFBX } from "@react-three/drei";
import { AnimationAction, AnimationMixer } from "three";

import { useMixamoAnimation } from "./useMixamoAnimation";
import { useVRM } from "./useVRM";

type Animations = {
  idle: AnimationAction;
  walk: AnimationAction;
  run: AnimationAction;
  jump: AnimationAction;
};

interface Props {
  src: string;
  animationsSrc: string;
  walkWeight: MutableRefObject<number>;
  jumpWeight: MutableRefObject<number>;
}

export function Avatar({ src, animationsSrc, walkWeight, jumpWeight }: Props) {
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
      animations.jump.weight = jumpWeight.current;

      animations.walk.weight = walkWeight.current;
      animations.idle.weight = 1 - walkWeight.current;
    }
  });

  if (!vrm?.scene) return null;

  return (
    <Suspense fallback={null}>
      <primitive object={vrm.scene} />
    </Suspense>
  );
}
