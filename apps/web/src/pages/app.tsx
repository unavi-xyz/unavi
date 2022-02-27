import { Canvas } from "@react-three/fiber";
import { Physics } from "@react-three/cannon";
import { useRouter } from "next/router";
import { Player } from "3d";

export default function App() {
  const router = useRouter();
  const roomId = router.query.room as string;

  return (
    <Canvas>
      <Physics>
        <ambientLight intensity={0.1} />
        <directionalLight color="red" position={[1, 2, 5]} />

        <mesh position={[0, 0, -4]}>
          <boxGeometry />
          <meshStandardMaterial />
        </mesh>

        <Player />
      </Physics>
    </Canvas>
  );
}
