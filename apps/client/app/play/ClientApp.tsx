import { useClient } from "@wired-labs/react-client";
import { useEffect } from "react";

import { useLoadUser } from "@/src/play/hooks/useLoadUser";
import LoadingScreen from "@/src/play/ui/LoadingScreen";
import Overlay from "@/src/play/ui/Overlay";
import { SpaceMetadata } from "@/src/server/helpers/readSpaceMetadata";

import FileDrop from "./FileDrop";

interface Props {
  metadata: SpaceMetadata;
}

export default function ClientApp({ metadata }: Props) {
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
      {loadingProgress == 1 ? <Overlay metadata={metadata} /> : null}

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
