import { Dispatch, SetStateAction, useContext, useRef, useState } from "react";
import { useQueryClient } from "react-query";
import {
  uploadFileToIpfs,
  useProfile,
  ImageSources,
  IpfsContext,
  useIpfsImage,
} from "ceramic";

import { Button, Dialog, ImageUpload, TextField } from "../base";

interface Props {
  id: string;
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
}

export function ProfileSettingsDialog({ id, open, setOpen }: Props) {
  const nameRef = useRef<HTMLInputElement>();
  const descriptionRef = useRef<HTMLTextAreaElement>();

  const { ipfs } = useContext(IpfsContext);

  const { profile, merge } = useProfile(id);
  const image = useIpfsImage(profile?.image?.original.src);
  const queryClient = useQueryClient();

  const [imageFile, setImageFile] = useState<File>();
  const [loading, setLoading] = useState(false);

  async function handleSave() {
    if (loading) return;
    setLoading(true);

    const name = nameRef.current.value;
    const description = descriptionRef.current.value;

    if (imageFile) {
      const cid = await uploadFileToIpfs(ipfs, imageFile);

      const imageElement = new Image();
      imageElement.src = URL.createObjectURL(imageFile);

      imageElement.onload = async () => {
        const imageObject: ImageSources = {
          original: {
            src: cid,
            height: imageElement.height,
            width: imageElement.width,
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
    setLoading(false);
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

          <div className="flex flex-col space-y-3">
            <label className="block text-lg pointer-events-none">Bio</label>
            <textarea
              ref={descriptionRef}
              defaultValue={profile?.description}
              maxLength={420}
              rows={8}
              className="w-full border p-2 leading-tight rounded"
            />
          </div>
        </div>

        <Button loading={loading} onClick={handleSave}>
          Save
        </Button>
      </div>
    </Dialog>
  );
}
