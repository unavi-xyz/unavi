import { useEngine, useMic, useWebSocket } from "@wired-labs/react-client";
import Link from "next/link";
import { useRef, useState } from "react";
import { IoMdArrowRoundBack, IoMdSettings } from "react-icons/io";
import { MdMic, MdMicOff } from "react-icons/md";

import { usePlayStore } from "../../../app/play/[id]/store";
import DialogContent, { DialogRoot } from "../../ui/Dialog";
import { toHex } from "../../utils/toHex";
import { useIsMobile } from "../../utils/useIsMobile";
import { LocalStorageKey } from "../constants";
import CrosshairTooltip, { CrosshairAction } from "../CrosshairTooltip";
import { useSetAvatar } from "../hooks/useSetAvatar";
import Stats from "../Stats";
import ChatBox from "./ChatBox";
import MobileChatBox from "./MobileChatBox";
import Settings from "./Settings";

interface Props {
  id: number;
  action: CrosshairAction;
}

export default function Overlay({ id, action }: Props) {
  const hasProducedAudio = useRef(false);
  const [openSettings, setOpenSettings] = useState(false);

  const engine = useEngine();
  const isMobile = useIsMobile();
  const setAvatar = useSetAvatar();
  const { send } = useWebSocket();
  const { micEnabled, setMicEnabled, setMicTrack } = useMic();

  async function handleClose() {
    setOpenSettings(false);
    if (!engine) return;

    const { didChangeName, didChangeAvatar, nickname, avatar } = usePlayStore.getState();

    if (didChangeName) {
      usePlayStore.setState({ didChangeName: false });

      // Save to local storage
      if (nickname) localStorage.setItem(LocalStorageKey.Name, nickname);
      else localStorage.removeItem(LocalStorageKey.Name);

      // Publish name change
      send({ subject: "set_name", data: nickname });
    }

    if (didChangeAvatar) {
      usePlayStore.setState({ didChangeAvatar: false });

      // Update engine
      engine.render.send({ subject: "set_user_avatar", data: avatar });

      if (avatar) {
        // Upload avatar
        setAvatar(avatar);
      } else {
        // Remove avatar
        send({ subject: "set_avatar", data: null });
        localStorage.removeItem(LocalStorageKey.Avatar);
      }
    }
  }

  async function toggleMic() {
    if (!engine) return;

    // If first time using mic, request permission
    if (!micEnabled && !hasProducedAudio.current) {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });

      const track = stream.getAudioTracks()[0];
      if (!track) throw new Error("No audio track found");

      setMicTrack(track);

      hasProducedAudio.current = true;
    }

    setMicEnabled(!micEnabled);
  }

  return (
    <>
      <DialogRoot
        open={openSettings}
        onOpenChange={(open) => {
          if (!open) handleClose();
        }}
      >
        <DialogContent open={openSettings} autoFocus={false} title="Settings">
          <Settings onClose={handleClose} />
        </DialogContent>
      </DialogRoot>

      <div className="fixed top-0 left-0 z-20 p-4">
        <Link href={`/space/${toHex(id)}`} className="rounded-full">
          <div className="rounded-full bg-white/80 p-3 text-2xl text-neutral-900 shadow backdrop-blur-xl transition hover:bg-white/90 hover:shadow-md active:scale-95">
            <IoMdArrowRoundBack />
          </div>
        </Link>
      </div>

      <div className="fixed top-16 left-0 z-20 p-4">
        <Stats />
      </div>

      <div className="crosshair" />
      <CrosshairTooltip action={action} />

      <div className="fixed top-0 right-0 z-20 space-x-2 p-4">
        <button
          onClick={toggleMic}
          className="rounded-full bg-white/80 p-3 text-2xl text-neutral-900 shadow backdrop-blur-xl transition hover:bg-white/90 hover:shadow-md active:scale-95"
        >
          {micEnabled ? <MdMic /> : <MdMicOff className="text-red-700" />}
        </button>
        <button
          onClick={() => setOpenSettings(true)}
          className="rounded-full bg-white/80 p-3 text-2xl text-neutral-900 shadow backdrop-blur-xl transition hover:bg-white/90 hover:shadow-md active:scale-95"
        >
          <IoMdSettings />
        </button>
      </div>

      <div className="fixed bottom-0 left-0 z-20 p-4">
        {isMobile ? (
          <MobileChatBox />
        ) : (
          <div className="w-96">
            <ChatBox />
          </div>
        )}
      </div>
    </>
  );
}
