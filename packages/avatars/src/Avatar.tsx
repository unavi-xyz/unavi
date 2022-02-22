import { useEffect, useRef, useState } from "react";
import { useFrame, useThree } from "@react-three/fiber";
import { useFBX } from "@react-three/drei";
import { AnimationAction, AnimationMixer } from "three";

import useVRM from "./hooks/useVRM";
import useMixamoAnimation from "./hooks/useMixamoAnimation";

export enum ANIMATIONS {
  idle = "idle",
  walk = "walk",
  run = "run",
  jump = "jump",
}

type Animations = {
  [ANIMATIONS.idle]: AnimationAction;
  [ANIMATIONS.walk]: AnimationAction;
  [ANIMATIONS.run]: AnimationAction;
  [ANIMATIONS.jump]: AnimationAction;
};

interface Props {
  firstPerson?: boolean;
  animation?: ANIMATIONS;
}

export function Avatar({
  firstPerson = false,
  animation = ANIMATIONS.idle,
}: Props) {
  const active = useRef<AnimationAction>();

  const [animations, setAnimations] = useState<Animations>();
  const [mixer, setMixer] = useState<AnimationMixer>();

  const fbx = useFBX("models/animations.fbx");
  const vrm = useVRM("models/Tira.vrm");

  useMixamoAnimation(fbx, vrm);

  const { camera } = useThree();

  useEffect(() => {
    if (!firstPerson || !vrm?.firstPerson) return;

    vrm.firstPerson.setup();
    camera.layers.enable(vrm.firstPerson.firstPersonOnlyLayer);
  }, [vrm]);

  useEffect(() => {
    if (!fbx) return;

    const scale = 0.01;
    fbx.scale.set(scale, scale, scale);
    fbx.rotation.y = Math.PI;

    const mixer = new AnimationMixer(fbx);
    setMixer(mixer);

    const idle = mixer.clipAction(fbx.animations[0]);
    const run = mixer.clipAction(fbx.animations[1]);
    const walk = mixer.clipAction(fbx.animations[2]);
    const jump = mixer.clipAction(fbx.animations[3]);

    const obj: Animations = {
      [ANIMATIONS.idle]: idle,
      [ANIMATIONS.walk]: walk,
      [ANIMATIONS.run]: run,
      [ANIMATIONS.jump]: jump,
    };

    setAnimations(obj);
  }, [fbx]);

  useEffect(() => {
    if (!mixer || !animations) return;
    fadeAnimation(animations[animation]);
  }, [animation, mixer]);

  function fadeAnimation(newAction: AnimationAction) {
    const time = 0.2;

    if (active.current) {
      const prev = active.current;
      prev.warp(1, 0, time).fadeOut(time);
      setTimeout(() => {
        prev.stop();
      }, time * 1000);
    }

    newAction.play().warp(0, 1, time).fadeIn(time);
    active.current = newAction;
  }

  useFrame(({ clock }) => {
    if (!mixer) return;
    const delta = clock.getDelta() * 10;
    mixer.update(delta);
  });

  return <group>{vrm && <primitive object={vrm.scene} />}</group>;
}
