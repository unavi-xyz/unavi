import { WorldMetadata } from "@wired-protocol/types";
import { MdConstruction } from "react-icons/md";

import { useAuth } from "@/src/client/AuthProvider";
import { HOME_SERVER } from "@/src/constants";
import Tooltip from "@/src/ui/Tooltip";

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
      {mode === PlayMode.Play ? (
        <PlayOverlay id={id} metadata={metadata} />
      ) : (
        <BuildOverlay />
      )}

      {isAuthor ? (
        <div className="fixed bottom-0 right-0 z-20 space-x-2 p-4">
          <Tooltip text="Toggle Build Mode" side="left">
            <button
              onClick={() => {
                if (mode === PlayMode.Play) {
                  usePlayStore.setState({ mode: PlayMode.Build });
                } else {
                  usePlayStore.setState({ mode: PlayMode.Play });
                }
              }}
              className={`h-[52px] w-[52px] rounded-full text-2xl backdrop-blur-lg transition active:scale-95 ${
                mode === PlayMode.Build
                  ? "bg-white text-black hover:bg-white/90"
                  : "bg-black/50 text-white hover:bg-black/70"
              }`}
            >
              <MdConstruction className="w-full" />
            </button>
          </Tooltip>
        </div>
      ) : null}

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
