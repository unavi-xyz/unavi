import { useState } from "react";
import { IoMdPerson } from "react-icons/io";

import { trpc } from "../../client/trpc";
import { env } from "../../env/client.mjs";
import Dialog from "../../ui/Dialog";
import { usePointerLocked } from "../hooks/usePointerLocked";
import { useAppStore } from "../store";
import { LocalStorageKey } from "./constants";
import UserPage from "./UserPage";

function getAvatarURL(fileId: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/temp/${fileId}`;
}

export default function UserButton() {
  const [openUserPage, setOpenUserPage] = useState(false);

  const isPointerLocked = usePointerLocked();

  const { mutateAsync: createTempUpload } = trpc.useMutation(
    "public.get-temp-upload"
  );

  async function handleClose() {
    setOpenUserPage(false);

    const engine = useAppStore.getState().engine;
    if (!engine) throw new Error("Engine not found");

    const { displayName, customAvatar, didChangeName, didChangeAvatar } =
      useAppStore.getState();

    // Name
    if (didChangeName) {
      useAppStore.setState({ didChangeName: false });

      // Publish display name
      engine.setName(displayName);

      // Save to local storage
      if (displayName) localStorage.setItem(LocalStorageKey.Name, displayName);
      else localStorage.removeItem(LocalStorageKey.Name);
    }

    // Avatar
    if (didChangeAvatar && customAvatar) {
      useAppStore.setState({ didChangeAvatar: false });

      // Get avatar file
      const body = await fetch(customAvatar).then((res) => res.blob());
      const { url, fileId } = await createTempUpload();

      // Upload to S3
      const res = await fetch(url, {
        method: "PUT",
        body,
        headers: {
          "Content-Type": body.type,
          "x-amz-acl": "public-read",
        },
      });

      if (!res.ok) throw new Error("Failed to upload avatar");

      // Publish avatar
      const avatarURL = getAvatarURL(fileId);
      engine.setAvatar(avatarURL);

      // Save to local storage
      localStorage.setItem(LocalStorageKey.Avatar, avatarURL);
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
