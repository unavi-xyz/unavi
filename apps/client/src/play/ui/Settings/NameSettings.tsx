import { useClientStore } from "@unavi/engine";

import { usePlayStore } from "@/app/play/playStore";
import { useAuth } from "@/src/client/AuthProvider";
import { toHex } from "@/src/utils/toHex";

import TextField from "../../../ui/TextField";

export default function NameSettings() {
  const nickname = usePlayStore((state) => state.uiName);
  const playerId = useClientStore((state) => state.playerId);
  const { user } = useAuth();

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
      onChange={(e) => usePlayStore.setState({ uiName: e.target.value })}
      className="text-center"
    />
  );
}
