import { useState } from "react";
import { IoMdPerson } from "react-icons/io";

import { trpc } from "../../client/trpc";
import { env } from "../../env/client.mjs";
import Dialog from "../../ui/Dialog";
import { usePointerLocked } from "../hooks/usePointerLocked";
import { useAppStore } from "../store";
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

    const { displayName, customAvatar } = useAppStore.getState();

    // Publish display name
    engine.setName(displayName);

    // Upload avatar to s3
    if (customAvatar) {
      const body = await fetch(customAvatar).then((res) => res.blob());

      const { url, fileId } = await createTempUpload();

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
