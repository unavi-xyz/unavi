import { Dispatch, SetStateAction, useRef } from "react";
import { useRouter } from "next/router";
import { useQueryClient } from "react-query";
import { mergeTile, removeRoomFromProfile, unpinTile, useRoom } from "ceramic";

import { Button, Dialog, TextField } from "./base";

interface Props {
  id: string;
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
}

export function RoomSettingsDialog({ id, open, setOpen }: Props) {
  const router = useRouter();

  const name = useRef<HTMLInputElement>();
  const description = useRef<HTMLInputElement>();

  const queryClient = useQueryClient();
  const { room } = useRoom(id);

  async function handleSave() {
    await mergeTile(
      id,
      { name: name.current.value, description: description.current.value },
      true
    );
    queryClient.invalidateQueries(`room-${id}`);
    setOpen(false);
  }

  async function handleDelete() {
    await unpinTile(id);
    await removeRoomFromProfile(id);
    router.push("/");
  }

  return (
    <Dialog open={open} setOpen={setOpen}>
      <div className="space-y-6">
        <h1 className="text-3xl flex justify-center">Settings</h1>

        <div className="space-y-4">
          <TextField title="Name" inputRef={name} defaultValue={room?.name} />
          <TextField
            title="Description"
            inputRef={description}
            defaultValue={room?.description}
          />
        </div>

        <Button onClick={handleSave}>
          <div>Save</div>
        </Button>

        <Button color="red" onClick={handleDelete}>
          <div>Delete</div>
        </Button>
      </div>
    </Dialog>
  );
}
