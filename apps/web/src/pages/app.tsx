import { useEffect } from "react";
import { Canvas } from "@react-three/fiber";
import { Physics } from "@react-three/cannon";
import { useRouter } from "next/router";
import { Player, World } from "3d";
import { useAuth, useRoom } from "ceramic";

import AppLayout from "../layouts/AppLayout";

export default function App() {
  const router = useRouter();
  const roomId = router.query.room as string;

  const { room } = useRoom(roomId);
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
        <World scene={room.scene} />
        <Player />
      </Physics>
    </Canvas>
  );
}

App.Layout = AppLayout;
