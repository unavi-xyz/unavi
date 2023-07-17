import { WorldMetadata } from "@wired-protocol/types";

import { useAuth } from "@/src/client/AuthProvider";
import { HOME_SERVER } from "@/src/constants";
import EditModeButton from "@/src/play/ui/editor/EditModeButton";

import ChatBox from "../../src/play/ui/ChatBox";
import MobileChatBox from "../../src/play/ui/MobileChatBox";
import { useIsMobile } from "../../src/utils/useIsMobile";
import BuildOverlay from "./BuildOverlay";
import PlayOverlay from "./PlayOverlay";
import { usePlayStore } from "./store";
import { PlayMode, WorldUriId } from "./types";

interface Props {
  id: WorldUriId;
  metadata: WorldMetadata;
}

export default function Overlay({ id, metadata }: Props) {
  const mode = usePlayStore((state) => state.mode);

  const isMobile = useIsMobile();
  const { user } = useAuth();

  const isAuthor = Boolean(
    metadata.info?.authors &&
      user?.username &&
      metadata.info.authors.includes(`@${user.username}:${HOME_SERVER}`)
  );

  return (
    <>
      {mode === PlayMode.Play ? <PlayOverlay id={id} /> : <BuildOverlay />}

      {isAuthor ? <EditModeButton /> : null}

      <div className="fixed bottom-0 left-0 z-20 p-4">
        {isMobile ? (
          <MobileChatBox />
        ) : (
          <div className="w-80">
            <ChatBox />
          </div>
        )}
      </div>
    </>
  );
}
