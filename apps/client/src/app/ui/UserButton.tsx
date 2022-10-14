import { useState } from "react";
import { IoMdPerson } from "react-icons/io";

import { useLens } from "../../client/lens/hooks/useLens";
import Dialog from "../../ui/Dialog";
import { LocalStorageKey } from "../constants";
import { usePointerLocked } from "../hooks/usePointerLocked";
import { useSetAvatar } from "../hooks/useSetAvatar";
import { useAppStore } from "../store";
import UserPage from "./UserPage";

export default function UserButton() {
  const [openUserPage, setOpenUserPage] = useState(false);

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

  const opacityClass = isPointerLocked ? "opacity-0" : "opacity-100";

  return (
    <>
      <Dialog open={openUserPage} onClose={handleClose}>
        <UserPage />
      </Dialog>

      <div
        className={`flex items-center justify-center transition ${opacityClass}`}
      >
        <button
          onClick={() => setOpenUserPage(true)}
          className="aspect-square rounded-full bg-surface p-3 text-3xl shadow transition hover:shadow-lg"
        >
          <IoMdPerson />
        </button>
      </div>
    </>
  );
}
