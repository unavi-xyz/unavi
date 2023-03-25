import { useClient } from "@wired-labs/react-client";
import { ERC721Metadata } from "contracts";

import { useLoadUser } from "../../../src/play/hooks/useLoadUser";
import LoadingScreen from "../../../src/play/ui/LoadingScreen";
import Overlay from "../../../src/play/ui/Overlay";

interface Props {
  spaceId: number;
  metadata: ERC721Metadata;
}

export default function ClientApp({ spaceId, metadata }: Props) {
  const { loadingProgress, loadingText } = useClient();

  useLoadUser();

  return (
    <>
      {loadingProgress == 1 ? <Overlay id={spaceId} /> : null}

      <LoadingScreen
        text={metadata.name}
        image={metadata.image}
        loadingProgress={loadingProgress}
        loadingText={loadingText}
      />
    </>
  );
}
