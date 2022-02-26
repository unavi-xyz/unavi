import { Dispatch, SetStateAction, useRef, useState } from "react";
import { addRoomToSpace, createRoom } from "ceramic";

import Button from "../Button";
import Dialog from "../Dialog/Dialog";
import ImageUpload from "../ImageUpload";
import TextField from "../TextField/TextField";

interface Props {
  spaceId: string;
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
}
export default function CreateRoomDialog({ spaceId, open, setOpen }: Props) {
  const name = useRef<HTMLInputElement>();
  const description = useRef<HTMLInputElement>();

  const [imageFile, setImageFile] = useState<File>();

  async function handleCreate() {
    const streamId = await createRoom(
      name.current.value,
      description.current.value,
      imageFile
    );

    await addRoomToSpace(spaceId, streamId);

    setOpen(false);
  }

  return (
    <Dialog open={open} setOpen={setOpen}>
      <div className="flex flex-col space-y-4">
        <h1 className="text-3xl flex justify-center">Create a room</h1>

        <div className="h-48">
          <ImageUpload setImageFile={setImageFile} />
        </div>

        <TextField title="Name" inputRef={name} />
        <TextField title="Description" inputRef={description} />

        <Button text="Create" onClick={handleCreate} />
      </div>
    </Dialog>
  );
}
