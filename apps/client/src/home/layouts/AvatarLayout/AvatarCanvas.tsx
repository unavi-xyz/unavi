import { Triplet } from "@react-three/cannon";
import { OrbitControls, Sky } from "@react-three/drei";
import { Canvas, useThree } from "@react-three/fiber";
import { useEffect, useRef } from "react";

interface Props {
  url: string;
  background?: boolean;
  cameraPosition?: Triplet;
}

export default function AvatarCanvas({
  url,
  background = true,
  cameraPosition = [0, 1.6, 1.6],
}: Props) {
  const controlsRef = useRef<any>(null);

  return (
    <Canvas
      shadows
      camera={{
        position: [0, 3, 3],
      }}
      className="rounded-xl select-none"
    >
      <ambientLight intensity={0.2} />
      <directionalLight
        castShadow
        position={[1, 2, 2]}
        shadow-mapSize-width={8192}
        shadow-mapSize-height={8192}
      />

      <CameraMover pos={cameraPosition} />
      <OrbitControls
        ref={controlsRef}
        makeDefault
        enablePan={false}
        autoRotate
        autoRotateSpeed={0.5}
        target={[0, 1, 0]}
      />

      {background && (
        <group>
          <Sky inclination={0} />
          <mesh
            receiveShadow
            rotation={[Math.PI / -2, 0, 0]}
            position={[0, 0.01, 0]}
          >
            <planeBufferGeometry args={[100, 100]} />
            <meshStandardMaterial color="#EADDFF" />
          </mesh>
        </group>
      )}

      <group rotation={[0, Math.PI, 0]}>
        {/* <Avatar
          src={url}
          animationsSrc="/models/animations.fbx"
          animationWeights={{
            idle: { current: 1 },
            walk: { current: 0 },
            run: { current: 0 },
            jump: { current: 0 },
          }}
        /> */}
      </group>
    </Canvas>
  );
}

function CameraMover({ pos }: { pos: Triplet }) {
  const { camera } = useThree();

  useEffect(() => {
    camera.position.fromArray(pos);
  }, [camera, pos]);

  return null;
}
