import { useClient } from "@wired-labs/react-client";
import { ERC721Metadata } from "contracts";
import { useEffect } from "react";

import { useLoadUser } from "../../../src/play/hooks/useLoadUser";
import LoadingScreen from "../../../src/play/ui/LoadingScreen";
import Overlay from "../../../src/play/ui/Overlay";
import FileDrop from "./FileDrop";

interface Props {
  spaceId: number;
  metadata: ERC721Metadata;
}

export default function ClientApp({ spaceId, metadata }: Props) {
  const { engine, loadingProgress, loadingText } = useClient();

  useLoadUser();

  useEffect(() => {
    if (!engine) return;

    // Expose engine to window for debugging
    (window as any).engine = engine;

    return () => {
      (window as any).engine = undefined;
    };
  }, [engine]);

  return (
    <>
      {loadingProgress == 1 ? <Overlay id={spaceId} /> : null}

      <FileDrop />

      <LoadingScreen
        text={metadata.name}
        image={metadata.image}
        loadingProgress={loadingProgress}
        loadingText={loadingText}
      />
    </>
  );
}
