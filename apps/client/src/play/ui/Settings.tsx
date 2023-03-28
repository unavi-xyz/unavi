import { ClientContext } from "@wired-labs/react-client";
import { useContext } from "react";

import { usePlayStore } from "../../../app/play/[id]/store";
import { useSession } from "../../client/auth/useSession";
import TextField from "../../ui/TextField";
import { toHex } from "../../utils/toHex";
import AccountSettings from "./AccountSettings";
import AvatarSettings from "./AvatarSettings";

interface Props {
  onClose: () => void;
}

export default function Settings({ onClose }: Props) {
  const nickname = usePlayStore((state) => state.nickname);

  const { playerId } = useContext(ClientContext);
  const { data: session } = useSession();

  const guestName =
    playerId == null || playerId === undefined ? "Guest" : `Guest ${toHex(playerId)}`;

  return (
    <div className="space-y-4">
      {!session?.address && (
        <TextField
          name="Name"
          placeholder={guestName}
          value={nickname ?? ""}
          onChange={(e) => {
            usePlayStore.setState({ didChangeName: true, nickname: e.target.value });
          }}
          className="h-full w-full rounded-lg bg-neutral-200/50 px-4 py-2 text-center text-neutral-900 placeholder:text-neutral-400"
        />
      )}

      <AvatarSettings />

      <AccountSettings onClose={onClose} />
    </div>
  );
}
