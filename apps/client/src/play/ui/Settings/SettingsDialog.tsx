import { useClientStore } from "@unavi/react-client";
import { useEffect, useState } from "react";

import { usePlayStore } from "@/app/play/store";

import DialogContent, { DialogRoot } from "../../../ui/Dialog";
import { LocalStorageKey } from "../../constants";
import { useSetAvatar } from "../../hooks/useSetAvatar";
import AccountSettings from "./AccountSettings";
import AvatarBrowser from "./AvatarBrowser";
import AvatarSettings from "./AvatarSettings";
import NameSettings from "./NameSettings";

export type SettingsPage = "Settings" | "Browse Avatars";

interface Props {
  open: boolean;
  setOpen: (value: boolean) => void;
}

export default function SettingsDialog({ open, setOpen }: Props) {
  const [page, setPage] = useState<SettingsPage>("Settings");

  const setAvatar = useSetAvatar();

  useEffect(() => {
    if (!open) return;
    setPage("Settings");
  }, [open]);

  async function handleClose() {
    setOpen(false);

    const { didChangeName, didChangeAvatar, nickname, avatar } =
      usePlayStore.getState();

    if (didChangeName) {
      usePlayStore.setState({ didChangeName: false });

      // Save to local storage
      if (nickname) localStorage.setItem(LocalStorageKey.Name, nickname);
      else localStorage.removeItem(LocalStorageKey.Name);

      // Publish name change
      useClientStore
        .getState()
        .sendWebSockets({ data: nickname, id: "xyz.unavi.world.user.name" });
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
        autoFocus={false}
        title={page}
        size={page === "Browse Avatars" ? "large" : "normal"}
      >
        {page === "Browse Avatars" ? (
          <AvatarBrowser setPage={setPage} onClose={handleClose} />
        ) : (
          <div className="space-y-4">
            <NameSettings />
            <AvatarSettings setPage={setPage} />
            <AccountSettings onClose={handleClose} />
          </div>
        )}
      </DialogContent>
    </DialogRoot>
  );
}
