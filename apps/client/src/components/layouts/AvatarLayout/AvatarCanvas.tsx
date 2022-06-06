import { OrbitControls, Sky } from "@react-three/drei";
import { Canvas, useThree } from "@react-three/fiber";
import { useEffect, useRef } from "react";

import { Avatar } from "@wired-xr/avatar";

import { useIpfsUrl } from "../../../helpers/ipfs/useIpfsUrl";
import { ANIMATIONS_URL } from "../../app/OtherPlayer";
import Spinner from "../../base/Spinner";

interface Props {
  uri: string;
}

export default function AvatarCanvas({ uri }: Props) {
  const controlsRef = useRef<any>(null);

  const url = useIpfsUrl(uri);

  if (!url)
    return (
      <div className="flex justify-center items-center h-full rounded-3xl animate-pulse bg-surfaceVariant">
        <Spinner />
      </div>
    );

  return (
    <Canvas
      shadows
      camera={{
        position: [0, 3, 3],
      }}
      className="rounded-3xl select-none"
    >
      <ambientLight intensity={0.2} />
      <directionalLight
        castShadow
        position={[1, 2, 2]}
        shadow-mapSize-width={8192}
        shadow-mapSize-height={8192}
      />

      <CameraMover />

      <Sky inclination={0} />
      <OrbitControls
        ref={controlsRef}
        makeDefault
        enablePan={false}
        autoRotate
        autoRotateSpeed={0.5}
        target={[0, 1, 0]}
      />

      <mesh
        receiveShadow
        rotation={[Math.PI / -2, 0, 0]}
        position={[0, 0.01, 0]}
      >
        <planeBufferGeometry args={[100, 100]} />
        <meshStandardMaterial color="#EADDFF" />
      </mesh>

      <group rotation={[0, Math.PI, 0]}>
        <Avatar
          src={url}
          animationsSrc={ANIMATIONS_URL}
          animationWeights={{
            idle: { current: 1 },
            walk: { current: 0 },
            run: { current: 0 },
            jump: { current: 0 },
          }}
        />
      </group>
    </Canvas>
  );
}

function CameraMover() {
  const { camera } = useThree();

  useEffect(() => {
    camera.position.set(0, 1.6, 2);
  }, [camera]);

  return null;
}
