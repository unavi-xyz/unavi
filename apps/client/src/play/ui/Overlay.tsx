import Link from "next/link";
import { useRef, useState } from "react";
import { IoMdArrowRoundBack, IoMdSettings } from "react-icons/io";
import { MdMic, MdMicOff } from "react-icons/md";

import DialogContent, { DialogRoot } from "../../ui/Dialog";
import { numberToHexDisplay } from "../../utils/numberToHexDisplay";
import { useIsMobile } from "../../utils/useIsMobile";
import { LocalStorageKey } from "../constants";
import { sendToHost } from "../hooks/useHost";
import { useSetAvatar } from "../hooks/useSetAvatar";
import { usePlayStore } from "../store";
import ChatBox from "./ChatBox";
import MobileChatBox from "./MobileChatBox";
import Settings from "./Settings";

interface Props {
  id: number;
}

export default function Overlay({ id }: Props) {
  const engine = usePlayStore((state) => state.engine);
  const [openSettings, setOpenSettings] = useState(false);
  const [muted, setMuted] = useState(true);
  const hasProducedAudio = useRef(false);

  const isMobile = useIsMobile();
  const setAvatar = useSetAvatar();

  async function handleClose() {
    setOpenSettings(false);
    if (!engine) return;

    const { didChangeName, didChangeAvatar, nickname, avatar, playerId, players } =
      usePlayStore.getState();

    if (!players || playerId === null) return;
    const playerName = players.names.get(playerId);
    if (!playerName) return;

    if (didChangeName) {
      usePlayStore.setState({ didChangeName: false });

      // Update name
      playerName.nickname = nickname;

      // Save to local storage
      if (nickname) localStorage.setItem(LocalStorageKey.Name, nickname);
      else localStorage.removeItem(LocalStorageKey.Name);

      // Publish name change
      sendToHost({ subject: "set_name", data: nickname });
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
        sendToHost({ subject: "set_avatar", data: null });
        localStorage.removeItem(LocalStorageKey.Avatar);
      }
    }
  }

  async function handleMic() {
    const { engine } = usePlayStore.getState();
    if (!engine) throw new Error("Engine not found");

    // Toggle mic
    const isMuted = !muted;
    usePlayStore.setState({ micPaused: isMuted });
    setMuted(isMuted);

    // If first time using mic, request permission
    if (!isMuted && !hasProducedAudio.current) {
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: true,
      });

      const track = stream.getAudioTracks()[0];
      if (!track) throw new Error("No audio track found");

      const { producerTransport } = usePlayStore.getState();
      if (!producerTransport) throw new Error("Producer transport not found");

      const producer = await producerTransport.produce({ track });

      usePlayStore.setState({ producer, producedTrack: track });
      hasProducedAudio.current = true;
    }

    const { producer } = usePlayStore.getState();

    if (isMuted) producer?.pause();
    else producer?.resume();
  }

  return (
    <>
      <DialogRoot
        open={openSettings}
        onOpenChange={(open) => {
          if (!open) handleClose();
        }}
      >
        <DialogContent open={openSettings} title="Settings">
          <Settings onClose={handleClose} />
        </DialogContent>
      </DialogRoot>

      <div className="absolute top-0 left-0 z-20 p-4">
        <Link href={`/space/${numberToHexDisplay(id)}`} className="rounded-full">
          <div className="rounded-full bg-white/80 p-3 text-2xl text-neutral-900 shadow backdrop-blur-xl transition hover:bg-white/90 hover:shadow-md active:scale-95">
            <IoMdArrowRoundBack />
          </div>
        </Link>
      </div>

      <div className="absolute top-0 right-0 z-20 space-x-2 p-4">
        <button
          onClick={handleMic}
          className="rounded-full bg-white/80 p-3 text-2xl text-neutral-900 shadow backdrop-blur-xl transition hover:bg-white/90 hover:shadow-md active:scale-95"
        >
          {muted ? <MdMicOff className="text-red-700" /> : <MdMic />}
        </button>
        <button
          onClick={() => setOpenSettings(true)}
          className="rounded-full bg-white/80 p-3 text-2xl text-neutral-900 shadow backdrop-blur-xl transition hover:bg-white/90 hover:shadow-md active:scale-95"
        >
          <IoMdSettings />
        </button>
      </div>

      <div className="absolute bottom-0 left-0 z-20 p-4">
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
