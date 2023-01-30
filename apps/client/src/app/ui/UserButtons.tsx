import Link from "next/link";
import { useRouter } from "next/router";
import { useEffect, useRef, useState } from "react";
import { IoMdArrowRoundBack, IoMdPerson } from "react-icons/io";
import { MdMic, MdMicOff } from "react-icons/md";

import { useAppStore } from "../../app/store";
import { useSession } from "../../client/auth/useSession";
import { trpc } from "../../client/trpc";
import Dialog from "../../ui/Dialog";
import Tooltip from "../../ui/Tooltip";
import { LocalStorageKey } from "../constants";
import { sendToHost } from "../hooks/useHost";
import { usePointerLocked } from "../hooks/usePointerLocked";
import { useSetAvatar } from "../hooks/useSetAvatar";
import UserPage from "./UserPage";

export default function UserButtons() {
  const router = useRouter();
  const id = router.query.id as string;

  const [openUserPage, setOpenUserPage] = useState(false);
  const [muted, setMuted] = useState(true);
  const hasProducedAudio = useRef(false);

  const engine = useAppStore((state) => state.engine);
  const isPointerLocked = usePointerLocked();
  const setAvatar = useSetAvatar();
  const utils = trpc.useContext();
  const { data: session } = useSession();

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
    }

    if (didChangeAvatar) {
      useAppStore.setState({ didChangeAvatar: false });
      // Update engine
      // engine.render.send({ subject: "set_user_avatar", data: customAvatar });

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

  const opacityClass = isPointerLocked ? "opacity-0" : "opacity-100";
  const mutedClass = muted ? "text-red-600" : "text-black";

  return (
    <>
      <Dialog open={openUserPage} onClose={handleClose}>
        <UserPage />
      </Dialog>

      <div className={`flex items-center justify-center space-x-4 transition ${opacityClass}`}>
        <Tooltip text="Leave">
          <Link href={`/space/${id}`}>
            <div className="aspect-square cursor-pointer rounded-full bg-white p-3 text-2xl shadow transition hover:shadow-lg">
              <IoMdArrowRoundBack />
            </div>
          </Link>
        </Tooltip>

        <Tooltip text="Identity">
          <button
            onClick={() => setOpenUserPage(true)}
            className="aspect-square rounded-full bg-white p-3 text-2xl shadow transition hover:shadow-lg"
          >
            <IoMdPerson />
          </button>
        </Tooltip>

        <Tooltip text={muted ? "Unmute" : "Mute"}>
          <button
            onClick={handleMic}
            className={`${mutedClass} aspect-square rounded-full bg-white p-3 text-2xl shadow transition hover:shadow-lg`}
          >
            {muted ? <MdMicOff /> : <MdMic />}
          </button>
        </Tooltip>
      </div>
    </>
  );
}
