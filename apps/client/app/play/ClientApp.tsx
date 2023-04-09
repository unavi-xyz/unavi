import { useClient } from "@wired-labs/react-client";
import { useEffect } from "react";

import Overlay from "@/app/play/Overlay";
import { useLoadUser } from "@/src/play/hooks/useLoadUser";
import LoadingScreen from "@/src/play/ui/LoadingScreen";
import { SpaceMetadata } from "@/src/server/helpers/readSpaceMetadata";

import FileDrop from "./FileDrop";
import { SpaceUriId } from "./types";

interface Props {
  id: SpaceUriId;
  metadata: SpaceMetadata;
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
        text={metadata.title}
        image={metadata.image}
        loadingProgress={loadingProgress}
        loadingText={loadingText}
      />
    </>
  );
}
