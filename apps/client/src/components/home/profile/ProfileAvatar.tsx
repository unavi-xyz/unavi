import React, { Suspense, useEffect, useRef, useState } from "react";
import { Canvas } from "@react-three/fiber";
import { ContactShadows, OrbitControls } from "@react-three/drei";
import { Vector3 } from "three";
import { useAvatar, useIpfsFile } from "ceramic";
import { AnimationWeights } from "3d";

const Avatar = React.lazy(() =>
  import("3d").then((module) => ({ default: module.Avatar }))
);

interface Props {
  avatarId?: string;
}

export default function ProfileAvatar({ avatarId }: Props) {
  const oneRef = useRef(1);
  const zeroRef = useRef(0);

  const animationWeights: AnimationWeights = {
    idle: oneRef,
    walk: zeroRef,
    run: zeroRef,
    jump: zeroRef,
  };

  const { avatar } = useAvatar(avatarId);
  const { url } = useIpfsFile(avatar?.vrm);

  return (
    <div className="w-full h-full">
      <Canvas
        mode="concurrent"
        camera={{ position: [0, 1.2, -1.6] }}
        className="rounded-3xl"
      >
        <OrbitControls
          makeDefault
          enablePan={false}
          target={new Vector3(0, 0.9, 0)}
          enableDamping
          dampingFactor={0.05}
          maxPolarAngle={Math.PI / 1.9}
        />

        <Suspense fallback={null}>
          {url && (
            <Avatar
              src={url}
              animationsSrc="/models/animations.fbx"
              animationWeights={animationWeights}
            />
          )}
        </Suspense>

        <ContactShadows width={2} height={2} blur={8} />

        <directionalLight intensity={0.9} position={new Vector3(10, 30, -20)} />
        <ambientLight intensity={0.1} />
      </Canvas>
    </div>
  );
}
