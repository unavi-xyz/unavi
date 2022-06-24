import { nanoid } from "nanoid";
import { useRouter } from "next/router";
import { useRef, useState } from "react";

import { uploadFileToIpfs } from "../../lib/ipfs/fetch";
import { useCreatePost } from "../../lib/lens/hooks/useCreatePost";
import { useProfileByHandle } from "../../lib/lens/hooks/useProfileByHandle";
import { useLensStore } from "../../lib/lens/store";
import { AppId, Metadata, MetadataVersions } from "../../lib/lens/types";
import Button from "../../ui/base/Button";
import Card from "../../ui/base/Card";
import FileUpload from "../../ui/base/FileUpload";
import TextArea from "../../ui/base/TextArea";
import TextField from "../../ui/base/TextField";
import { crop } from "../../utils/crop";
import AvatarCanvas from "../layouts/AvatarLayout/AvatarCanvas";

export default function AvatarUploadPage() {
  const descriptionRef = useRef<HTMLTextAreaElement>(null);

  const [name, setName] = useState<string>("My Avatar");
  const [imageFile, setImageFile] = useState<File>();
  const [vrmFile, setVrmFile] = useState<File>();
  const [loading, setLoading] = useState(false);

  const router = useRouter();
  const handle = useLensStore((state) => state.handle);
  const profile = useProfileByHandle(handle);
  const createPost = useCreatePost(profile?.id);

  const disableSubmit = !imageFile || !vrmFile || !handle;

  async function handleSubmit() {
    if (loading || disableSubmit) return;

    setLoading(true);

    try {
      //upload image to IPFS
      const cropped = await crop(URL.createObjectURL(imageFile), 3 / 5);
      const imageURI = await uploadFileToIpfs(cropped);

      //upload vrm to IPFS
      const vrmURI = await uploadFileToIpfs(vrmFile);

      //create metadata
      const metadata: Metadata = {
        version: MetadataVersions.one,
        metadata_id: nanoid(),
        name,
        description: descriptionRef.current?.value ?? "",
        content: vrmURI,
        image: imageURI,
        imageMimeType: imageFile.type,
        attributes: [],
        animation_url: undefined,
        external_url: "https://thewired.space",
        media: [{ item: imageURI, type: imageFile.type }],
        appId: AppId.avatar,
      };

      //create post
      await createPost(metadata);

      router.push(`/user/${handle}`);
    } catch (error) {
      console.error(error);
    }

    setLoading(false);
  }

  return (
    <div className="space-y-8">
      <div className="flex flex-col items-center space-y-1">
        <h1 className="text-3xl flex justify-center">Upload Avatar</h1>
        <p className="text-lg flex justify-center">Mint a new avatar NFT</p>
      </div>

      <div className="space-y-4">
        <TextField
          autoComplete="off"
          title="Name"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />

        <TextArea
          textAreaRef={descriptionRef}
          autoComplete="off"
          title="Description"
          defaultValue=""
        />

        <div className="space-y-4">
          <div className="text-lg font-bold">Image</div>

          <FileUpload
            color="SurfaceVariant"
            title="Image"
            accept="image/*"
            onChange={(e) => {
              const file = e.target.files?.[0];
              if (file) setImageFile(file);
            }}
          />
        </div>

        <div className="space-y-4">
          <div className="text-lg font-bold">VRM File</div>

          <FileUpload
            color="SurfaceVariant"
            title="VRM"
            accept=".vrm"
            onChange={(e) => {
              const file = e.target.files?.[0];
              if (file) setVrmFile(file);
            }}
          />
        </div>
      </div>

      {(imageFile || vrmFile) && (
        <div className="flex space-x-4">
          {imageFile && (
            <Card
              text={name}
              image={URL.createObjectURL(imageFile)}
              aspect="vertical"
            />
          )}

          <div className="w-full">
            {vrmFile && <AvatarCanvas url={URL.createObjectURL(vrmFile)} />}
          </div>
        </div>
      )}

      <div className="flex justify-end">
        <Button
          onClick={handleSubmit}
          variant="filled"
          disabled={disableSubmit}
          loading={loading}
        >
          Submit
        </Button>
      </div>
    </div>
  );
}
