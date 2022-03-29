import { Physics } from "@react-three/cannon";
import { Canvas } from "@react-three/fiber";
import { useContextBridge } from "@react-three/drei";
import { useRouter } from "next/router";
import { QueryClientProvider } from "react-query";
import { IpfsContext } from "ceramic";
import { Player } from "3d";

import World from "./World";
import { queryClient } from "../../../helpers/constants";
import { MultiplayerContext } from "../MultiplayerProvider";
import { useContext, useEffect } from "react";

export default function AppCanvas() {
  const router = useRouter();
  const spaceId = router.query.space as string;

  const { setRoomId } = useContext(MultiplayerContext);

  const ContextBridge = useContextBridge(IpfsContext, MultiplayerContext);

  useEffect(() => {
    setRoomId(spaceId);
  }, [setRoomId, spaceId]);

  return (
    <Canvas mode="concurrent">
      <QueryClientProvider client={queryClient}>
        <ContextBridge>
          <Physics>
            <Player />
            <World spaceId={spaceId} />
          </Physics>
        </ContextBridge>
      </QueryClientProvider>
    </Canvas>
  );
}
