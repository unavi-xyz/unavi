import { useEngine, useLoading } from "@wired-labs/react-client";
import { ERC721Metadata } from "contracts";

import { useAvatarEquip } from "../../../src/play/hooks/useAvatarEquip";
import { useLoadUser } from "../../../src/play/hooks/useLoadUser";
import { useSetAvatar } from "../../../src/play/hooks/useSetAvatar";
import LoadingScreen from "../../../src/play/ui/LoadingScreen";
import Overlay from "../../../src/play/ui/Overlay";
import { usePlayStore } from "./store";

interface Props {
  spaceId: number;
  metadata: ERC721Metadata;
}

export default function ClientApp({ spaceId, metadata }: Props) {
  const avatar = usePlayStore((state) => state.avatar);

  const engine = useEngine();
  const setAvatar = useSetAvatar();
  const { progress, text } = useLoading();
  const equipAction = useAvatarEquip(engine, avatar, setAvatar);

  useLoadUser();

  return (
    <>
      {progress == 1 ? <Overlay id={spaceId} action={equipAction} /> : null}

      <LoadingScreen
        text={metadata.name}
        image={metadata.image}
        loadingProgress={progress}
        loadingText={text}
      />
    </>
  );
}
