import { Dispatch, SetStateAction, useContext, useRef, useState } from "react";
import { useRouter } from "next/router";
import { useQueryClient } from "react-query";
import {
  editSpace,
  IpfsContext,
  removeFromSpaces,
  unpinTile,
  useAuth,
  useIpfsImage,
  useSpace,
} from "ceramic";

import { Button, Dialog, ImageUpload, TextField } from "../../base";

interface Props {
  id: string;
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
}

export function SpaceSettingsDialog({ id, open, setOpen }: Props) {
  const router = useRouter();

  const nameRef = useRef<HTMLInputElement>();
  const descriptionRef = useRef<HTMLTextAreaElement>();

  const { ipfs } = useContext(IpfsContext);

  const [imageFile, setImageFile] = useState<File>();
  const [loadingSave, setLoadingSave] = useState(false);
  const [loadingsDelete, setLoadingsDelete] = useState(false);

  const queryClient = useQueryClient();
  const { viewerId } = useAuth();
  const { space } = useSpace(id);
  const image = useIpfsImage(space?.image);

  async function handleSave() {
    if (loadingSave) return;
    setLoadingSave(true);

    const name = nameRef.current.value;
    const description = descriptionRef.current.value;

    await editSpace(ipfs, id, name, description, imageFile);

    queryClient.invalidateQueries(`space-${id}`);
    setOpen(false);
    setLoadingSave(false);
  }

  async function handleDelete() {
    if (loadingsDelete) return;
    setLoadingsDelete(true);

    await unpinTile(id);
    await removeFromSpaces(id);

    queryClient.invalidateQueries(`user-spaces-${viewerId}`);
    router.back();
  }

  return (
    <Dialog open={open} setOpen={setOpen}>
      <div className="space-y-6">
        <h1 className="text-3xl flex justify-center">Settings</h1>

        <div className="h-32">
          <ImageUpload setImageFile={setImageFile} defaultValue={image} />
        </div>

        <div className="space-y-4">
          <TextField
            title="Name"
            inputRef={nameRef}
            defaultValue={space?.name}
          />

          <div className="flex flex-col space-y-3">
            <label className="block text-lg pointer-events-none">
              Description
            </label>
            <textarea
              ref={descriptionRef}
              defaultValue={space?.description}
              maxLength={420}
              rows={8}
              className="w-full border p-2 leading-tight rounded"
            />
          </div>
        </div>

        <div className="space-y-2">
          <Button loading={loadingSave} onClick={handleSave}>
            Save
          </Button>
          <Button loading={loadingsDelete} color="red" onClick={handleDelete}>
            Delete
          </Button>
        </div>
      </div>
    </Dialog>
  );
}
