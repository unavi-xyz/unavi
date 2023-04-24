import { useClient } from "@unavi/react-client";
import { useEffect, useState } from "react";

import { usePlayStore } from "@/app/play/store";

import DialogContent, { DialogRoot } from "../../../ui/Dialog";
import { LocalStorageKey } from "../../constants";
import { useSetAvatar } from "../../hooks/useSetAvatar";
import AvatarBrowser from "./AvatarBrowser";
import Settings from "./Settings";

export type SettingsPage = "Settings" | "Browse Avatars";

interface Props {
  open: boolean;
  setOpen: (value: boolean) => void;
}

export default function SettingsDialog({ open, setOpen }: Props) {
  const [page, setPage] = useState<SettingsPage>("Settings");

  const setAvatar = useSetAvatar();
  const { engine, send } = useClient();

  useEffect(() => {
    if (!open) return;
    setPage("Settings");
  }, [open]);

  async function handleClose() {
    setOpen(false);

    if (!engine) return;

    const { didChangeName, didChangeAvatar, nickname, avatar } = usePlayStore.getState();

    if (didChangeName) {
      usePlayStore.setState({ didChangeName: false });

      // Save to local storage
      if (nickname) localStorage.setItem(LocalStorageKey.Name, nickname);
      else localStorage.removeItem(LocalStorageKey.Name);

      // Publish name change
      send({ type: "set_name", data: nickname });
    }

    if (didChangeAvatar) setAvatar(avatar);
  }

  return (
    <DialogRoot
      open={open}
      onOpenChange={(value) => {
        if (!value) handleClose();
      }}
    >
      <DialogContent
        open={open}
        autoFocus={false}
        title={page}
        size={page === "Browse Avatars" ? "large" : "normal"}
      >
        {page === "Browse Avatars" ? (
          <AvatarBrowser setPage={setPage} onClose={handleClose} />
        ) : (
          <Settings setPage={setPage} onClose={handleClose} />
        )}
      </DialogContent>
    </DialogRoot>
  );
}
