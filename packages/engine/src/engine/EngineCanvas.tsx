import { useContextBridge } from "@react-three/drei";
import { Canvas } from "@react-three/fiber";
import { Context } from "urql";

import { EthersContext } from "@wired-xr/ethers";
import { IpfsContext } from "@wired-xr/ipfs";
import { LensContext } from "@wired-xr/lens";

import { NetworkingContext } from "../networking";

interface Props {
  children: React.ReactNode;
}

export function EngineCanvas({ children }: Props) {
  const ContextBridge = useContextBridge(
    NetworkingContext,
    LensContext,
    EthersContext,
    IpfsContext,
    Context
  );

  return (
    <Canvas shadows>
      <ContextBridge>{children}</ContextBridge>
    </Canvas>
  );
}
