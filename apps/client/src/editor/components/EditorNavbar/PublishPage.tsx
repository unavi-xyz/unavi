import { WorldMetadata } from "@wired-protocol/types";
import { ERC721Metadata, ERC721MetadataSchema } from "contracts";
import { nanoid } from "nanoid";
import Link from "next/link";
import { useEffect, useState } from "react";
import { toast } from "react-hot-toast";
import useSWR from "swr";

import { GetFileDownloadResponse } from "@/app/api/projects/[id]/files/[file]/types";
import { linkProject } from "@/app/api/projects/[id]/link/helper";
import { publishProject } from "@/app/api/projects/[id]/publish/helper";
import { MAX_DESCRIPTION_LENGTH, MAX_TITLE_LENGTH } from "@/app/api/projects/constants";
import { copyProjectToModel } from "@/app/api/spaces/[id]/model/copy-project/helper";
import { getSpaceModelFileUpload } from "@/app/api/spaces/[id]/model/files/[file]/helper";
import { createSpaceModel } from "@/app/api/spaces/[id]/model/helper";
import {
  getSpaceNFTFileDownload,
  getSpaceNFTFileUpload,
} from "@/app/api/spaces/[id]/nft/files/[file]/helper";
import { useAuth } from "@/src/client/AuthProvider";
import { env } from "@/src/env.mjs";

import { fetcher } from "../../../play/utils/fetcher";
import { Project } from "../../../server/helpers/fetchProject";
import Button from "../../../ui/Button";
import ImageInput from "../../../ui/ImageInput";
import TextArea from "../../../ui/TextArea";
import TextField from "../../../ui/TextField";
import { bytesToDisplay } from "../../../utils/bytesToDisplay";
import { cdnURL, S3Path } from "../../../utils/s3Paths";
import { useSave } from "../../hooks/useSave";
import { cropImage } from "../../utils/cropImage";
import { parseError } from "../../utils/parseError";
import { useEditor } from "../Editor";

interface Props {
  project: Project;
}

export default function PublishPage({ project }: Props) {
  const { engine, title: editorTitle, image } = useEditor();

  const { user } = useAuth();
  const { save } = useSave(project);

  const [title, setTitle] = useState(editorTitle);
  const [description, setDescription] = useState(project.description);

  const { data: imageDownload } = useSWR<GetFileDownloadResponse>(
    () => `/api/projects/${project.id}/files/image`,
    fetcher,
    { revalidateOnFocus: false, revalidateOnReconnect: false }
  );

  const [imageFile, setImageFile] = useState<File>();
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (imageFile) return;
    if (image) cropImage(image).then(setImageFile);
    else if (imageDownload) cropImage(imageDownload.url).then(setImageFile);
  }, [imageFile, image, imageDownload]);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();

    if (loading || !engine || !user) return;

    const toastId = nanoid();

    async function publish() {
      if (!engine) throw new Error("Engine not found");
      if (!user) throw new Error("Session not found");

      toast.loading("Preparing space...", { id: toastId });

      // Start saving
      const savePromise = save();

      // Publish project
      const { spaceId, nftId } = await publishProject(project.id);

      // Create space model
      const { modelId } = await createSpaceModel(spaceId);
      const imageURL = cdnURL(S3Path.spaceModel(modelId).image);
      const modelURL = cdnURL(S3Path.spaceModel(modelId).model);

      async function uploadWorldMetadata() {
        const metadata: WorldMetadata = {
          info: {
            name: title.trimEnd(),
            description: description.trimEnd(),
            authors: user?.username
              ? [`${user.username}@${env.NEXT_PUBLIC_DEPLOYED_URL}`]
              : undefined,
            image: imageURL,
            host: env.NEXT_PUBLIC_DEFAULT_HOST,
          },
          model: modelURL,
        };

        const url = await getSpaceModelFileUpload(spaceId, "metadata");

        const response = await fetch(url, {
          method: "PUT",
          body: JSON.stringify(metadata),
          headers: { "Content-Type": "application/json", "x-amz-acl": "public-read" },
        });

        if (!response.ok) throw new Error("Failed to upload metadata");
      }

      async function uploadModel() {
        if (!engine) throw new Error("Engine not found");

        // Finish saving
        await savePromise;

        const [url, optimizedModel] = await Promise.all([
          getSpaceModelFileUpload(spaceId, "model"),
          engine.scene.export({ optimize: true }),
        ]);

        const response = await fetch(url, {
          method: "PUT",
          body: optimizedModel,
          headers: { "Content-Type": "model/gltf-binary", "x-amz-acl": "public-read" },
        });

        if (!response.ok) throw new Error("Failed to upload model");

        console.info("📦 Published model size:", bytesToDisplay(optimizedModel.byteLength));
      }

      async function uploadImage() {
        if (!imageFile) throw new Error("Image not found");

        // Get image
        const res = await fetch(URL.createObjectURL(imageFile));
        const imageBlob = await res.blob();

        // Upload to S3
        const url = await getSpaceModelFileUpload(spaceId, "image");

        const response = await fetch(url, {
          method: "PUT",
          body: imageBlob,
          headers: { "Content-Type": "image/jpeg", "x-amz-acl": "public-read" },
        });

        if (!response.ok) throw new Error("Failed to upload image");
      }

      // If space has an NFT, update the metadata
      async function uploadNftMetadata() {
        if (!nftId) return;

        const currentMetadataURL = await getSpaceNFTFileDownload(spaceId, "metadata");
        const currentMetadataRes = await fetch(currentMetadataURL);
        const currentMetadata = ERC721MetadataSchema.parse(await currentMetadataRes.json());

        const erc721metadata: ERC721Metadata = {
          ...currentMetadata,
          name: title.trimEnd(),
          description: description.trimEnd(),
          image: imageURL,
          animation_url: modelURL,
        };

        // Upload to S3
        const url = await getSpaceNFTFileUpload(spaceId, "metadata");

        const response = await fetch(url, {
          method: "PUT",
          body: JSON.stringify(erc721metadata),
          headers: {
            "Content-Type": "application/json",
            "x-amz-acl": "public-read",
          },
        });

        if (!response.ok) throw new Error("Failed to upload image");
      }

      toast.loading("Uploading metadata...", { id: toastId });

      await Promise.all([
        copyProjectToModel(spaceId, { projectId: project.id }),
        uploadWorldMetadata(),
        uploadModel(),
        uploadImage(),
        uploadNftMetadata(),
        linkProject(project.id, { spaceId }),
      ]);

      toast.success(
        (t) => (
          <span className="space-x-4">
            <span>Space published!</span>

            <Link
              href={`/space/${spaceId}`}
              onClick={() => toast.dismiss(t.id)}
              className="rounded-full bg-neutral-200 px-4 py-1 font-semibold transition hover:bg-neutral-300 active:opacity-80"
            >
              View
            </Link>
          </span>
        ),
        { id: toastId, duration: 20000 }
      );
    }

    setLoading(true);

    try {
      await publish();
    } catch (err) {
      toast.error(parseError(err, "Failed to publish."), { id: toastId });
      console.error(err);
    }

    setLoading(false);
  }

  const imageUrl = imageFile ? URL.createObjectURL(imageFile) : undefined;

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <TextField
        label="Title"
        name="title"
        autoComplete="off"
        maxLength={MAX_TITLE_LENGTH}
        defaultValue={title}
        disabled={loading}
        onChange={(e) => setTitle(e.target.value)}
      />

      <TextArea
        label="Description"
        name="description"
        autoComplete="off"
        rows={4}
        maxLength={MAX_DESCRIPTION_LENGTH}
        defaultValue={description}
        disabled={loading}
        onChange={(e) => setDescription(e.target.value)}
      />

      <ImageInput
        name="Image"
        src={imageUrl}
        disabled={loading}
        onChange={(e) => {
          const file = e.target.files?.[0];
          if (!file) return;
          cropImage(URL.createObjectURL(file)).then((file) => setImageFile(file));
        }}
        className="h-72 w-full"
      />

      <div className="flex justify-end">
        <Button disabled={loading} type="submit">
          Submit
        </Button>
      </div>
    </form>
  );
}
