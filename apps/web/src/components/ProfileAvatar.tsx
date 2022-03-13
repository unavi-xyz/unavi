import React, { Suspense, useRef } from "react";
import { Canvas } from "@react-three/fiber";
import { ContactShadows, OrbitControls } from "@react-three/drei";
import { Vector3 } from "three";

const Avatar = React.lazy(() =>
  import("3d").then((module) => ({ default: module.Avatar }))
);

export default function ProfileAvatar() {
  const walkWeight = useRef(0);
  const jumpWeight = useRef(0);

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
          <Avatar
            src="/models/Latifa.vrm"
            animationsSrc="/models/animations.fbx"
            walkWeight={walkWeight}
            jumpWeight={jumpWeight}
          />
        </Suspense>

        <ContactShadows width={2} height={2} blur={8} />

        <directionalLight />
        <ambientLight intensity={0.1} />
      </Canvas>
    </div>
  );
}
