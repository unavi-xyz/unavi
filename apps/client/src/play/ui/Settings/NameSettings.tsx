import { ClientContext } from "@wired-labs/react-client";
import { useContext } from "react";

import { usePlayStore } from "@/app/play/store";

import { useSession } from "../../../client/auth/useSession";
import TextField from "../../../ui/TextField";
import { toHex } from "../../../utils/toHex";

export default function NameSettings() {
  const nickname = usePlayStore((state) => state.nickname);
  const { data: session } = useSession();
  const { playerId } = useContext(ClientContext);

  const guestName =
    playerId == null || playerId === undefined ? "Guest" : `Guest ${toHex(playerId)}`;

  if (session?.address) return null;

  return (
    <TextField
      label="Name"
      name="name"
      placeholder={guestName}
      value={nickname ?? ""}
      onChange={(e) => {
        usePlayStore.setState({ didChangeName: true, nickname: e.target.value });
      }}
      className="text-center"
    />
  );
}
