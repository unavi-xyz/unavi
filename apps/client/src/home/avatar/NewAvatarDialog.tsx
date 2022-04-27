import { Dispatch, SetStateAction, useContext, useRef, useState } from "react";
import { useRouter } from "next/router";

import { Button, Dialog, ImageUpload, TextField } from "../../components/base";
import { createAvatar, IpfsContext } from "ceramic";

interface Props {
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
}

export default function NewAvatarDialog({ open, setOpen }: Props) {
  const router = useRouter();

  const { ipfs } = useContext(IpfsContext);

  const inputRef = useRef<HTMLInputElement>();
  const nameRef = useRef<HTMLInputElement>();
  const descriptionRef = useRef<HTMLInputElement>();

  const [imageFile, setImageFile] = useState<File>();

  async function handleCreate() {
    const name = nameRef.current.value;
    const description = descriptionRef.current.value;
    const vrmfile = inputRef.current.files[0];

    if (!vrmfile) return;

    const id = await createAvatar(ipfs, name, description, imageFile, vrmfile);

    router.push(`/avatar/${id}`);
  }

  return (
    <Dialog open={open} setOpen={setOpen}>
      <div className="space-y-6">
        <h1 className="text-3xl flex justify-center">Upload an Avatar</h1>

        <div className="space-y-4">
          <div className="h-32">
            <ImageUpload setImageFile={setImageFile} />
          </div>

          <input ref={inputRef} type="file" accept=".vrm" />

          <TextField
            title="Name"
            inputRef={nameRef}
            defaultValue="New avatar"
          />
          <TextField title="Description" inputRef={descriptionRef} />
        </div>

        <Button onClick={handleCreate}>Create</Button>
      </div>
    </Dialog>
  );
}
