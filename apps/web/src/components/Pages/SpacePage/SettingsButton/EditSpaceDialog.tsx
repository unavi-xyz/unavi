import { Dispatch, SetStateAction, useEffect, useRef, useState } from "react";
import { editSpace, useSpace } from "ceramic";

import Button from "../../../base/Button";
import Dialog from "../../../base/Dialog/Dialog";
import ImageUpload from "../../../base/ImageUpload";
import TextField from "../../../base/TextField/TextField";

interface Props {
  spaceId: string;
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
}
export default function EditSpaceDialog({ spaceId, open, setOpen }: Props) {
  const { space } = useSpace(spaceId);

  const name = useRef<HTMLInputElement>();
  const description = useRef<HTMLInputElement>();

  const [imageFile, setImageFile] = useState<File>();

  useEffect(() => {
    if (!name.current) return;
    name.current.value = space?.name;
    description.current.value = space?.description;
  }, [space]);

  async function handleCreate() {
    await editSpace(
      spaceId,
      name.current.value,
      description.current.value,
      imageFile
    );

    setOpen(false);
  }

  return (
    <Dialog open={open} setOpen={setOpen}>
      <div className="flex flex-col space-y-4">
        <h1 className="text-3xl flex justify-center">Edit space</h1>

        <div className="w-28 h-28">
          <ImageUpload
            setImageFile={setImageFile}
            defaultValue={space?.image}
          />
        </div>

        <TextField title="Name" inputRef={name} defaultValue={space?.name} />
        <TextField
          title="Description"
          inputRef={description}
          defaultValue={space?.description}
        />

        <div className="flex space-x-2">
          <Button onClick={() => setOpen(false)} color="gray">
            <span className="text-xl">Cancel</span>
          </Button>
          <Button onClick={handleCreate}>
            <span className="text-xl">Save</span>
          </Button>
        </div>
      </div>
    </Dialog>
  );
}
