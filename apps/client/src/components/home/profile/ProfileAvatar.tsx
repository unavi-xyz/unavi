import React, { Suspense, useEffect, useRef, useState } from "react";
import { Canvas } from "@react-three/fiber";
import { ContactShadows, OrbitControls } from "@react-three/drei";
import { Vector3 } from "three";
import { useAvatar, useIpfsFile } from "ceramic";

const Avatar = React.lazy(() =>
  import("3d").then((module) => ({ default: module.Avatar }))
);

interface Props {
  avatarId?: string;
}

export default function ProfileAvatar({ avatarId }: Props) {
  const walkWeight = useRef(0);
  const jumpWeight = useRef(0);

  const { avatar } = useAvatar(avatarId);
  const vrmFile = useIpfsFile(avatar?.vrm);

  const [vrmUrl, setVrmUrl] = useState<string>();

  useEffect(() => {
    if (!vrmFile) {
      setVrmUrl(undefined);
      return;
    }

    const url = URL.createObjectURL(vrmFile);
    setVrmUrl(url);
  }, [vrmFile]);

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
          {vrmUrl && (
            <Avatar
              src={vrmUrl}
              animationsSrc="/models/animations.fbx"
              walkWeight={walkWeight}
              jumpWeight={jumpWeight}
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
