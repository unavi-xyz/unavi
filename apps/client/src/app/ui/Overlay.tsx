import Link from "next/link";
import { useRouter } from "next/router";
import { useEffect, useRef, useState } from "react";
import { IoMdArrowRoundBack, IoMdSettings } from "react-icons/io";
import { MdMic, MdMicOff } from "react-icons/md";

import { useSession } from "../../client/auth/useSession";
import { trpc } from "../../client/trpc";
import Dialog from "../../ui/Dialog";
import { useIsMobile } from "../../utils/useIsMobile";
import { LocalStorageKey } from "../constants";
import { sendToHost } from "../hooks/useHost";
import { useSetAvatar } from "../hooks/useSetAvatar";
import { useAppStore } from "../store";
import ChatBox from "./ChatBox";
import MobileChatBox from "./MobileChatBox";
import Settings from "./Settings";

export default function Overlay() {
  const engine = useAppStore((state) => state.engine);
  const [openUserPage, setOpenUserPage] = useState(false);
  const [muted, setMuted] = useState(true);
  const hasProducedAudio = useRef(false);

  const isMobile = useIsMobile();
  const setAvatar = useSetAvatar();
  const utils = trpc.useContext();
  const router = useRouter();
  const { data: session } = useSession();

  const id = router.query.id as string;

  useEffect(() => {
    if (!session?.address) return;
    utils.social.profile.byAddress.prefetch({ address: session.address });
  }, [session, utils]);

  async function handleClose() {
    setOpenUserPage(false);
    if (!engine) return;

    const { didChangeName, didChangeAvatar, nickname, avatar, playerId, players } =
      useAppStore.getState();

    if (!players || playerId === null) return;
    const playerName = players.names.get(playerId);
    if (!playerName) return;

    if (didChangeName) {
      useAppStore.setState({ didChangeName: false });

      // Update name
      playerName.nickname = nickname;

      // Save to local storage
      if (nickname) localStorage.setItem(LocalStorageKey.Name, nickname);
      else localStorage.removeItem(LocalStorageKey.Name);

      // Publish name change
      sendToHost({ subject: "set_name", data: nickname });
    }

    if (didChangeAvatar) {
      useAppStore.setState({ didChangeAvatar: false });

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
    const { engine } = useAppStore.getState();
    if (!engine) throw new Error("Engine not found");

    // Toggle mic
    const isMuted = !muted;
    useAppStore.setState({ micPaused: isMuted });
    setMuted(isMuted);

    // If first time using mic, request permission
    if (!isMuted && !hasProducedAudio.current) {
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: true,
      });

      const track = stream.getAudioTracks()[0];
      if (!track) throw new Error("No audio track found");

      const { producerTransport } = useAppStore.getState();
      if (!producerTransport) throw new Error("Producer transport not found");

      const producer = await producerTransport.produce({ track });

      useAppStore.setState({ producer, producedTrack: track });
      hasProducedAudio.current = true;
    }

    const { producer } = useAppStore.getState();

    if (isMuted) producer?.pause();
    else producer?.resume();
  }

  return (
    <>
      <Dialog open={openUserPage} onClose={handleClose}>
        <Settings />
      </Dialog>

      <div className="absolute top-0 left-0 z-20 p-4">
        <Link href={`/space/${id}`}>
          <div className="rounded-full bg-white/60 p-3 text-2xl text-neutral-900 shadow backdrop-blur-xl transition hover:bg-white/70 hover:shadow-md active:opacity-80">
            <IoMdArrowRoundBack />
          </div>
        </Link>
      </div>

      <div className="absolute top-0 right-0 z-20 space-x-2 p-4">
        <button
          onClick={handleMic}
          className="rounded-full bg-white/60 p-3 text-2xl text-neutral-900 shadow backdrop-blur-xl transition hover:bg-white/70 hover:shadow-md active:opacity-80"
        >
          {muted ? <MdMicOff className="text-red-700" /> : <MdMic />}
        </button>
        <button
          onClick={() => setOpenUserPage(true)}
          className="rounded-full bg-white/60 p-3 text-2xl text-neutral-900 shadow backdrop-blur-xl transition hover:bg-white/70 hover:shadow-md active:opacity-80"
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
