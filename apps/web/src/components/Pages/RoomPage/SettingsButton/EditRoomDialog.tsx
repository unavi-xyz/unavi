import { Dispatch, SetStateAction, useEffect, useRef, useState } from "react";
import { deleteRoom, editRoom, removeRoomFromSpace, useRoom } from "ceramic";

import Button from "../../../base/Button";
import Dialog from "../../../base/Dialog/Dialog";
import ImageUpload from "../../../base/ImageUpload";
import TextField from "../../../base/TextField/TextField";

interface Props {
  spaceId: string;
  roomId: string;
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
}
export default function EditRoomDialog({
  spaceId,
  roomId,
  open,
  setOpen,
}: Props) {
  const { room } = useRoom(roomId);

  const name = useRef<HTMLInputElement>();
  const description = useRef<HTMLInputElement>();

  const [imageFile, setImageFile] = useState<File>();

  useEffect(() => {
    setImageFile(undefined);
  }, [open]);

  async function handleCreate() {
    await editRoom(
      roomId,
      name.current.value,
      description.current.value,
      imageFile
    );
    setOpen(false);
  }

  async function handleDelete() {
    await deleteRoom(roomId);
    await removeRoomFromSpace(spaceId, roomId);
    setOpen(false);
  }

  return (
    <Dialog open={open} setOpen={setOpen}>
      <div className="flex flex-col space-y-4">
        <h1 className="text-3xl flex justify-center">Edit room</h1>

        <div className="h-48">
          <ImageUpload setImageFile={setImageFile} defaultValue={room?.image} />
        </div>

        <TextField title="Name" inputRef={name} defaultValue={room?.name} />
        <TextField
          title="Description"
          inputRef={description}
          defaultValue={room?.description}
        />

        <div className="flex space-x-2">
          <Button onClick={() => setOpen(false)} color="gray">
            <span className="text-xl">Cancel</span>
          </Button>
          <Button onClick={handleCreate}>
            <span className="text-xl">Save</span>
          </Button>
        </div>

        <Button onClick={handleDelete} color="red">
          <span className="text-xl">Delete Room</span>
        </Button>
      </div>
    </Dialog>
  );
}
