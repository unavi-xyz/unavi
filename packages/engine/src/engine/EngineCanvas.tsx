import { useContextBridge } from "@react-three/drei";
import { Canvas } from "@react-three/fiber";
import { Context } from "urql";

import { IpfsContext } from "@wired-xr/ipfs";
import { LensContext } from "@wired-xr/lens";

import { NetworkingContext } from "../networking";

interface Props {
  children: React.ReactNode;
}

export function EngineCanvas({ children }: Props) {
  const ContextBridge = useContextBridge(NetworkingContext, LensContext, IpfsContext, Context);

  return (
    <Canvas shadows>
      <ContextBridge>{children}</ContextBridge>
    </Canvas>
  );
}
