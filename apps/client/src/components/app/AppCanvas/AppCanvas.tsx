import { useEffect } from "react";
import { Physics } from "@react-three/cannon";
import { Canvas } from "@react-three/fiber";
import { useContextBridge } from "@react-three/drei";
import { useRouter } from "next/router";
import { QueryClientProvider } from "react-query";
import { IpfsContext } from "ceramic";
import { Player } from "3d";

import { queryClient } from "../../../helpers/constants";
import { appManager } from "../helpers/store";
import { SocketContext } from "../SocketProvider";
import World from "./World";

export default function AppCanvas() {
  const router = useRouter();
  const spaceId = router.query.space as string;

  const ContextBridge = useContextBridge(IpfsContext, SocketContext);

  useEffect(() => {
    appManager.setSpaceId(spaceId);
  }, [spaceId]);

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
