import { ClientContext } from "@unavi/react-client";
import { useContext } from "react";

import { usePlayStore } from "@/app/play/store";
import { useAuth } from "@/src/client/AuthProvider";

import TextField from "../../../ui/TextField";
import { toHex } from "../../../utils/toHex";

export default function NameSettings() {
  const nickname = usePlayStore((state) => state.nickname);
  const { user } = useAuth();
  const { playerId } = useContext(ClientContext);

  const guestName =
    playerId == null || playerId === undefined
      ? "Guest"
      : `Guest ${toHex(playerId)}`;

  if (user?.address) return null;

  return (
    <TextField
      label="Name"
      name="name"
      placeholder={guestName}
      value={nickname ?? ""}
      onChange={(e) => {
        usePlayStore.setState({
          didChangeName: true,
          nickname: e.target.value,
        });
      }}
      className="text-center"
    />
  );
}
