import { Dispatch, SetStateAction, useRef, useState } from "react";
import { useQueryClient } from "react-query";
import {
  uploadImageToIpfs,
  useIpfsFile,
  useProfile,
  ImageSources,
} from "ceramic";

import { Button, Dialog, ImageUpload, TextField } from "./base";

interface Props {
  id: string;
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
}

export function ProfileSettingsDialog({ id, open, setOpen }: Props) {
  const nameRef = useRef<HTMLInputElement>();
  const descriptionRef = useRef<HTMLInputElement>();

  const [imageFile, setImageFile] = useState<File>();

  const queryClient = useQueryClient();
  const { profile, merge } = useProfile(id);
  const image = useIpfsFile(profile?.image?.original.src);

  async function handleSave() {
    const name = nameRef.current.value;
    const description = descriptionRef.current.value;

    if (imageFile) {
      const cid = await uploadImageToIpfs(imageFile);

      const img = new Image();
      img.src = URL.createObjectURL(imageFile);

      img.onload = async () => {
        const imageObject: ImageSources = {
          original: {
            src: cid,
            height: img.height,
            width: img.width,
            mimeType: imageFile.type,
            size: imageFile.size,
          },
        };

        await merge({
          name,
          description,
          image: imageObject,
        });
      };
    } else {
      await merge({
        name,
        description,
      });
    }

    queryClient.invalidateQueries(`basicProfile-${id}`);
    setOpen(false);
  }

  return (
    <Dialog open={open} setOpen={setOpen}>
      <div className="space-y-6">
        <h1 className="text-3xl flex justify-center">Edit Profile</h1>

        <div className="h-32 w-32">
          <ImageUpload setImageFile={setImageFile} defaultValue={image} />
        </div>

        <div className="space-y-4">
          <TextField
            title="Name"
            inputRef={nameRef}
            defaultValue={profile?.name}
          />
          <TextField
            title="Bio"
            inputRef={descriptionRef}
            defaultValue={profile?.description}
          />
        </div>

        <Button onClick={handleSave}>
          <div>Save</div>
        </Button>
      </div>
    </Dialog>
  );
}
