import { useClient } from "@unavi/react-client";
import { WorldMetadata } from "@wired-protocol/types";
import { useEffect } from "react";

import Overlay from "@/app/play/Overlay";
import { useLoadUser } from "@/src/play/hooks/useLoadUser";
import LoadingScreen from "@/src/play/ui/LoadingScreen";

import FileDrop from "./FileDrop";
import { WorldUriId } from "./types";

interface Props {
  id: WorldUriId;
  metadata: WorldMetadata;
}

export default function ClientApp({ id, metadata }: Props) {
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
      {loadingProgress == 1 ? <Overlay id={id} metadata={metadata} /> : null}

      <FileDrop />

      <LoadingScreen
        text={metadata.info?.name}
        image={metadata.info?.image}
        loadingProgress={loadingProgress}
        loadingText={loadingText}
      />
    </>
  );
}
