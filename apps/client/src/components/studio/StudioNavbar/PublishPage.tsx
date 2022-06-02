import produce from "immer";
import { nanoid } from "nanoid";
import { useRouter } from "next/router";
import { useRef, useState } from "react";

import { uploadFileToIpfs } from "../../../helpers/ipfs/fetch";
import { useCreatePost } from "../../../helpers/lens/hooks/useCreatePost";
import { useProfileByHandle } from "../../../helpers/lens/hooks/useProfileByHandle";
import { useLensStore } from "../../../helpers/lens/store";
import { AppId, Metadata, MetadataVersions } from "../../../helpers/lens/types";
import { getFileByPath } from "../../../helpers/studio/filesystem";
import { useProject } from "../../../helpers/studio/hooks/useProject";
import { useStudioStore } from "../../../helpers/studio/store";
import { crop } from "../../../helpers/utils/crop";
import Button from "../../base/Button";
import FileUpload from "../../base/FileUpload";
import TextArea from "../../base/TextArea";
import TextField from "../../base/TextField";

export default function PublishPage() {
  const project = useProject();
  const nameRef = useRef<HTMLInputElement>(null);
  const descriptionRef = useRef<HTMLTextAreaElement>(null);

  const handle = useLensStore((state) => state.handle);
  const rootHandle = useStudioStore((state) => state.rootHandle);
  const [imageFile, setImageFile] = useState<File>();
  const [loading, setLoading] = useState(false);

  const router = useRouter();
  const profile = useProfileByHandle(handle);
  const createPost = useCreatePost(profile?.id);

  const disableSubmit = !imageFile || !handle;

  async function handleSubmit() {
    if (loading || disableSubmit || !project?.scene || !rootHandle) return;

    setLoading(true);

    try {
      //upload image to IPFS
      const cropped = await crop(URL.createObjectURL(imageFile), 5 / 3);
      const imageURI = await uploadFileToIpfs(cropped);

      //upload scene assets to IPFS
      const finalScene = await produce(project.scene, async (draft) => {
        await Promise.all(
          Object.entries(draft.assets).map(async ([key, asset]) => {
            const fileHandle = await getFileByPath(asset.uri, rootHandle);
            if (!fileHandle) return;
            const file = await fileHandle.getFile();
            if (!file) return;

            const uri = await uploadFileToIpfs(file);
            draft.assets[key].uri = uri;
          })
        );
      });

      const metadata: Metadata = {
        version: MetadataVersions.one,
        metadata_id: nanoid(),
        name: nameRef.current?.value ?? "",
        description: descriptionRef.current?.value ?? "",
        content: JSON.stringify(finalScene),
        image: imageURI,
        imageMimeType: imageFile.type,
        attributes: [],
        animation_url: undefined,
        external_url: "https://thewired.space",
        media: [{ item: imageURI, type: imageFile.type }],
        appId: AppId.space,
      };

      await createPost(metadata);

      router.push(`/user/${handle}`);
    } catch (err) {
      console.error(err);
    }

    setLoading(false);
  }

  return (
    <div className="space-y-8">
      <div className="flex flex-col items-center space-y-1">
        <h1 className="text-3xl flex justify-center">Publish</h1>
        <p className="text-lg flex justify-center">Mint a new space NFT</p>
      </div>

      <div className="space-y-4">
        <TextField
          inputRef={nameRef}
          autoComplete="off"
          title="Name"
          defaultValue={project?.name}
        />
        <TextArea
          textAreaRef={descriptionRef}
          autoComplete="off"
          title="Description"
          defaultValue={project?.description}
        />

        <div className="space-y-4">
          <div className="text-lg font-bold">Image</div>

          {imageFile && (
            <div className="w-full aspect-card">
              <img
                src={URL.createObjectURL(imageFile)}
                alt="cover picture preview"
                className="object-cover rounded-xl h-full w-full border"
              />
            </div>
          )}

          <FileUpload
            color="SurfaceVariant"
            title="Cover Picture"
            accept="image/*"
            onChange={(e) => {
              const file = e.target.files?.[0];
              if (file) setImageFile(file);
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
