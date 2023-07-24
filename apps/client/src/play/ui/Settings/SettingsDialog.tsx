import { useClientStore } from "@unavi/engine";
import { useEffect, useState } from "react";

import { usePlayStore } from "@/app/play/playStore";

import DialogContent, { DialogRoot } from "../../../ui/Dialog";
import { LocalStorageKey } from "../../constants";
import { setAvatar } from "../../utils/setAvatar";
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

  useEffect(() => {
    if (!open) return;
    setPage("Settings");
  }, [open]);

  async function handleClose() {
    setOpen(false);

    const { uiName: name, uiAvatar: avatar } = usePlayStore.getState();

    setAvatar(avatar);

    // Save name to local storage
    if (name) {
      localStorage.setItem(LocalStorageKey.Name, name);
    } else {
      localStorage.removeItem(LocalStorageKey.Name);
    }

    // Set name
    useClientStore.getState().setName(name);
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
          <div className="space-y-6">
            <AvatarSettings setPage={setPage} />
            <NameSettings />
            <AccountSettings onClose={handleClose} />
          </div>
        )}
      </DialogContent>
    </DialogRoot>
  );
}
