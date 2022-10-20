import Link from "next/link";
import { useRouter } from "next/router";
import { useState } from "react";
import { IoMdArrowRoundBack, IoMdPerson } from "react-icons/io";
import { MdMic, MdMicOff } from "react-icons/md";

import { useLens } from "../../client/lens/hooks/useLens";
import Dialog from "../../ui/Dialog";
import Tooltip from "../../ui/Tooltip";
import { LocalStorageKey } from "../constants";
import { usePointerLocked } from "../hooks/usePointerLocked";
import { useSetAvatar } from "../hooks/useSetAvatar";
import { useAppStore } from "../store";
import UserPage from "./UserPage";

export default function UserButton() {
  const router = useRouter();
  const id = router.query.id as string;

  const [openUserPage, setOpenUserPage] = useState(false);
  const [muted, setMuted] = useState(true);

  const isPointerLocked = usePointerLocked();
  const { handle } = useLens();

  const setAvatar = useSetAvatar();

  async function handleClose() {
    setOpenUserPage(false);

    const engine = useAppStore.getState().engine;
    if (!engine) throw new Error("Engine not found");

    const { displayName, customAvatar, didChangeName, didChangeAvatar } =
      useAppStore.getState();

    // If no lens handle, use name
    if (!handle && didChangeName) {
      useAppStore.setState({ didChangeName: false });

      // Publish display name
      engine.setName(displayName);

      // Save to local storage
      if (displayName) localStorage.setItem(LocalStorageKey.Name, displayName);
      else localStorage.removeItem(LocalStorageKey.Name);
    }

    // Avatar
    if (didChangeAvatar && customAvatar) {
      setAvatar(customAvatar);
    } else if (didChangeAvatar) {
      engine.setAvatar(null);
      localStorage.removeItem(LocalStorageKey.Avatar);
    }
  }

  function handleMic() {
    setMuted((prev) => !prev);
  }

  const opacityClass = isPointerLocked ? "opacity-0" : "opacity-100";

  return (
    <>
      <Dialog open={openUserPage} onClose={handleClose}>
        <UserPage />
      </Dialog>

      <div
        className={`flex items-center justify-center space-x-4 transition ${opacityClass}`}
      >
        <Tooltip text="Leave">
          <Link href={`/space/${id}`}>
            <div className="aspect-square cursor-pointer rounded-full bg-surface p-3 text-2xl shadow transition hover:shadow-lg">
              <IoMdArrowRoundBack />
            </div>
          </Link>
        </Tooltip>

        <Tooltip text="Identity">
          <button
            onClick={() => setOpenUserPage(true)}
            className="aspect-square rounded-full bg-surface p-3 text-2xl shadow transition hover:shadow-lg"
          >
            <IoMdPerson />
          </button>
        </Tooltip>

        <Tooltip text={muted ? "Unmute" : "Mute"}>
          <button
            onClick={handleMic}
            className="aspect-square rounded-full bg-surface p-3 text-2xl shadow transition hover:shadow-lg"
          >
            {muted ? <MdMicOff /> : <MdMic />}
          </button>
        </Tooltip>
      </div>
    </>
  );
}
