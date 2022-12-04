import {
  AppId,
  PublicationMainFocus,
  PublicationMetadata,
  PublicationMetadataMedia,
  PublicationMetadataVersions,
} from "lens";
import { nanoid } from "nanoid";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";

import { useLens } from "../../../client/lens/hooks/useLens";
import { trpc } from "../../../client/trpc";
import { env } from "../../../env/client.mjs";
import Button from "../../../ui/Button";
import ButtonFileInput from "../../../ui/ButtonFileInput";
import TextArea from "../../../ui/TextArea";
import TextField from "../../../ui/TextField";
import { useEditorStore } from "../../store";
import { cropImage } from "../../utils/cropImage";

function cdnModelURL(id: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/published/${id}/model.glb`;
}

function cdnImageURL(id: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/published/${id}/image.jpg`;
}

interface Props {
  onClose: () => void;
}

export default function UpdatePage({ onClose }: Props) {
  const name = useEditorStore((state) => state.name);
  const description = useEditorStore((state) => state.description);
  const publicationId = useEditorStore((state) => state.publicationId);

  const { handle } = useLens();
  const router = useRouter();
  const id = router.query.id as string;

  const { data: imageURL } = trpc.project.image.useQuery(
    { id },
    {
      enabled: id !== undefined,
      trpc: {},
    }
  );

  const { mutateAsync: saveProject } = trpc.project.save.useMutation();

  const { mutateAsync: createModelUploadUrl } =
    trpc.publication.modelUploadURL.useMutation();

  const { mutateAsync: createImageUploadUrl } =
    trpc.publication.imageUploadURL.useMutation();

  const { mutateAsync: createMetadataUploadUrl } =
    trpc.publication.metadataUploadURL.useMutation();

  const [imageFile, setImageFile] = useState<File>();
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (imageFile || !imageURL) return;
    cropImage(imageURL).then((file) => setImageFile(file));
  }, [imageFile, imageURL]);

  async function handlePublish() {
    if (loading) return;

    if (!publicationId) throw new Error("No publication id");

    try {
      setLoading(true);
      const promises: Promise<void>[] = [];

      // Save project
      promises.push(
        saveProject({
          id,
          name,
          description,
        })
      );

      // Export scene and upload to S3
      promises.push(
        new Promise((resolve, reject) => {
          async function upload() {
            const { engine } = useEditorStore.getState();
            if (!engine) throw new Error("Engine not found");
            if (!publicationId) throw new Error("No publication id");

            // Export scene to glb
            const glb = await engine.export();
            const body = new Blob([glb], { type: "model/gltf-binary" });

            // Upload to S3
            const url = await createModelUploadUrl({ id: publicationId });
            const response = await fetch(url, {
              method: "PUT",
              body,
              headers: {
                "Content-Type": "model/gltf-binary",
                "x-amz-acl": "public-read",
              },
            });

            if (!response.ok) reject();
            else resolve();
          }

          upload();
        })
      );

      // Upload image to S3
      promises.push(
        new Promise((resolve, reject) => {
          async function upload() {
            if (!publicationId) throw new Error("No publication id");

            if (!imageFile) {
              reject();
              return;
            }

            // Get image
            const res = await fetch(URL.createObjectURL(imageFile));
            const body = await res.blob();

            // Upload to S3
            const url = await createImageUploadUrl({ id: publicationId });
            const response = await fetch(url, {
              method: "PUT",
              body,
              headers: {
                "Content-Type": "image/jpeg",
                "x-amz-acl": "public-read",
              },
            });

            if (!response.ok) reject();
            else resolve();
          }

          upload();
        })
      );

      // Upload metadata to S3
      promises.push(
        new Promise((resolve, reject) => {
          async function upload() {
            if (!publicationId) throw new Error("No publication id");

            const modelURL = cdnModelURL(publicationId);
            const imageURL = cdnImageURL(publicationId);

            const media: PublicationMetadataMedia[] = [
              {
                item: imageURL,
                type: "image/jpeg",
                altTag: "preview image",
              },
              {
                item: modelURL,
                type: "model/gltf-binary",
                altTag: "model",
              },
            ];

            const metadata: PublicationMetadata = {
              version: PublicationMetadataVersions.two,
              metadata_id: nanoid(),
              description,
              locale: "en-US",
              tags: ["3d", "gltf", "space", "wired"],
              mainContentFocus: PublicationMainFocus.Image,
              external_url: `https://thewired.space/user/${handle}`,
              name,
              image: imageURL,
              imageMimeType: "image/jpeg",
              media,
              attributes: [],
              appId: AppId.Space,
            };

            // Upload to S3
            const url = await createMetadataUploadUrl({ id: publicationId });
            const response = await fetch(url, {
              method: "PUT",
              body: JSON.stringify(metadata),
              headers: {
                "Content-Type": "application/json",
                "x-amz-acl": "public-read",
              },
            });

            if (response.ok) resolve();
            else reject();
          }

          upload();
        })
      );

      await Promise.all(promises);

      setLoading(false);
      onClose();
    } catch (error) {
      console.error(error);
      setLoading(false);
    }
  }

  const image = imageFile ? URL.createObjectURL(imageFile) : null;

  return (
    <div className="space-y-8">
      <div className="flex flex-col items-center space-y-1">
        <h1 className="flex justify-center text-4xl font-bold">Update Space</h1>
      </div>

      <div className="space-y-4">
        <TextField
          onChange={(e) => {
            const value = e.target.value;
            useEditorStore.setState({ name: value });
          }}
          title="Name"
          outline
          defaultValue={name}
        />

        <TextArea
          onChange={(e) => {
            const value = e.target.value;
            useEditorStore.setState({ description: value });
          }}
          autoComplete="off"
          title="Description"
          outline
          defaultValue={description}
        />

        <div className="space-y-4">
          <div className="text-lg font-bold">Image</div>

          <div className="relative aspect-card h-full w-full rounded-xl bg-primaryContainer">
            {image && (
              <img
                src={image}
                alt="picture preview"
                className="h-full w-full rounded-xl object-cover"
              />
            )}
          </div>

          <ButtonFileInput
            title="Cover Picture"
            accept="image/*"
            onChange={(e) => {
              const file = e.target.files?.[0];
              if (!file) return;

              cropImage(URL.createObjectURL(file)).then((file) =>
                setImageFile(file)
              );
            }}
          >
            Upload Image
          </ButtonFileInput>
        </div>
      </div>

      <div className="flex justify-end">
        <Button onClick={handlePublish} variant="filled" loading={loading}>
          Submit
        </Button>
      </div>
    </div>
  );
}
