import { nanoid } from "nanoid";
import { useRouter } from "next/router";
import { useRef, useState } from "react";

import { uploadFileToIpfs } from "../../helpers/ipfs/fetch";
import { useCreatePost } from "../../helpers/lens/hooks/useCreatePost";
import { useProfileByHandle } from "../../helpers/lens/hooks/useProfileByHandle";
import { useLensStore } from "../../helpers/lens/store";
import { AppId, Metadata, MetadataVersions } from "../../helpers/lens/types";
import { crop } from "../../helpers/utils/crop";
import Button from "../base/Button";
import FileUpload from "../base/FileUpload";
import TextArea from "../base/TextArea";
import TextField from "../base/TextField";

export default function AvatarUploadPage() {
  const nameRef = useRef<HTMLInputElement>(null);
  const descriptionRef = useRef<HTMLTextAreaElement>(null);

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
        name: nameRef.current?.value ?? "",
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

      router.push(`/user/${handle}/avatars`);
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
          inputRef={nameRef}
          autoComplete="off"
          title="Name"
          defaultValue="My Avatar"
        />
        <TextArea
          textAreaRef={descriptionRef}
          autoComplete="off"
          title="Description"
          defaultValue=""
        />

        <div className="space-y-4">
          <div className="text-lg font-bold">Image</div>

          {imageFile && (
            <div className="w-1/3 aspect-vertical">
              <img
                src={URL.createObjectURL(imageFile)}
                alt="cover picture preview"
                className="object-cover rounded-xl h-full w-full border"
              />
            </div>
          )}

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
