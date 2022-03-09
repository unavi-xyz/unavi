import { Canvas } from "@react-three/fiber";
import { Physics } from "@react-three/cannon";
import { Sky } from "@react-three/drei";
import { useRouter } from "next/router";
import { Player } from "3d";
import { useAuth } from "ceramic";

import AppLayout from "../layouts/AppLayout";
import { useEffect } from "react";

export default function App() {
  const router = useRouter();
  const roomId = router.query.room as string;

  const { authenticated, connect } = useAuth();

  useEffect(() => {
    if (!authenticated) connect();
  }, [authenticated, connect]);

  if (!authenticated) {
    return <div></div>;
  }

  return (
    <Canvas>
      <Physics>
        <ambientLight intensity={0.1} />
        <directionalLight color="red" position={[1, 2, 5]} />

        <mesh position={[0, 0, -4]}>
          <boxGeometry />
          <meshStandardMaterial />
        </mesh>

        <Sky />

        <Player />
      </Physics>
    </Canvas>
  );
}

App.Layout = AppLayout;
