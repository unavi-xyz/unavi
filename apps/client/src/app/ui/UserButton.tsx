import { useState } from "react";
import { IoMdPerson } from "react-icons/io";

import Dialog from "../../ui/Dialog";
import { usePointerLocked } from "../hooks/usePointerLocked";
import { useAppStore } from "../store";
import UserPage from "./UserPage";

export default function UserButton() {
  const [openUserPage, setOpenUserPage] = useState(false);

  const isPointerLocked = usePointerLocked();

  const opacityClass = isPointerLocked ? "opacity-0" : "opacity-100";

  function handleClose() {
    setOpenUserPage(false);

    // Publish display name
    const engine = useAppStore.getState().engine;
    if (!engine) throw new Error("Engine not found");

    const { displayName } = useAppStore.getState();
    engine.setName(displayName);
  }

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
