import { useEffect } from "react";
import { useThree } from "@react-three/fiber";
import { OrbitControls } from "@react-three/drei";

const GRID_SIZE = 20;

export default function Scene() {
  const { camera } = useThree();

  useEffect(() => {
    camera.position.set(9, 9, 9);
  }, [camera]);

  return (
    <group>
      <OrbitControls
        addEventListener={undefined}
        hasEventListener={undefined}
        removeEventListener={undefined}
        dispatchEvent={undefined}
      />
      <gridHelper args={[GRID_SIZE, GRID_SIZE]} />
    </group>
  );
}
