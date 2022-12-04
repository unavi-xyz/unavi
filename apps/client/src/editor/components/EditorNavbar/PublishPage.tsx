import { useConnectModal } from "@rainbow-me/rainbowkit";
import {
  AppId,
  GetPublicationsDocument,
  GetPublicationsQuery,
  GetPublicationsQueryVariables,
  PublicationMainFocus,
  PublicationMetadata,
  PublicationMetadataMedia,
  PublicationMetadataVersions,
} from "lens";
import { nanoid } from "nanoid";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";
import { useSigner } from "wagmi";

import { useCreatePost } from "../../../client/lens/hooks/useCreatePost";
import { useLens } from "../../../client/lens/hooks/useLens";
import { useProfileByHandle } from "../../../client/lens/hooks/useProfileByHandle";
import { trpc } from "../../../client/trpc";
import { env } from "../../../env/client.mjs";
import Button from "../../../ui/Button";
import ButtonFileInput from "../../../ui/ButtonFileInput";
import TextArea from "../../../ui/TextArea";
import TextField from "../../../ui/TextField";
import { useSave } from "../../hooks/useSave";
import { useEditorStore } from "../../store";
import { cropImage } from "../../utils/cropImage";

function cdnModelURL(id: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/published/${id}/model.glb`;
}

function cdnImageURL(id: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/published/${id}/image.jpg`;
}

function cdnMetadataURL(id: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/published/${id}/metadata.json`;
}

export default function PublishPage() {
  const router = useRouter();
  const id = router.query.id as string;

  const name = useEditorStore((state) => state.name);
  const description = useEditorStore((state) => state.description);

  const { handle, client } = useLens();
  const { data: signer } = useSigner();
  const { openConnectModal } = useConnectModal();
  const profile = useProfileByHandle(handle);
  const createPost = useCreatePost(profile?.id);
  const { save } = useSave();

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

  const { mutateAsync: createPublication } =
    trpc.publication.create.useMutation();

  const { mutateAsync: linkPublication } = trpc.publication.link.useMutation();

  const [imageFile, setImageFile] = useState<File>();
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (imageFile || !imageURL) return;
    cropImage(imageURL).then((file) => setImageFile(file));
  }, [imageFile, imageURL]);

  async function handlePublish() {
    if (loading) return;

    if (!signer) {
      if (openConnectModal) openConnectModal();
      return;
    }

    try {
      setLoading(true);
      const promises: Promise<void>[] = [];

      // Save project
      await save();

      // Create database publication
      const publicationId = await createPublication();

      // Export scene and upload to S3
      promises.push(
        new Promise((resolve, reject) => {
          async function upload() {
            const { engine } = useEditorStore.getState();
            if (!engine) throw new Error("Engine not found");

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

      const contentURI = cdnMetadataURL(publicationId);

      // Create lens publication
      const success = await createPost(contentURI);

      if (!success) {
        setLoading(false);
        return;
      }

      // Save project, linking it to the publication
      promises.push(
        saveProject({
          id,
          name,
          description,
          publicationId,
        })
      );

      if (!profile) {
        router.push(`/user/${handle}`);
        return;
      }

      // Wait 3 seconds for the publication to be indexed
      await new Promise((resolve) => setTimeout(resolve, 3000));

      // This is a hack to get the publicationId
      // Get latest publications
      const { data } = await client
        .query<GetPublicationsQuery, GetPublicationsQueryVariables>(
          GetPublicationsDocument,
          {
            request: {
              profileId: profile.id,
              limit: 4, // We use 4 just in case other publications were created at the same time
              sources: [AppId.Space],
            },
          },
          {
            fetchOptions: { cache: "reload" },
            requestPolicy: "network-only",
          }
        )
        .toPromise();

      // Find the publication we just created by comparing metadata
      const publication = data?.publications.items.find(
        (item) =>
          item.metadata.name === name &&
          item.metadata.description === description
      );

      if (!publication) {
        router.push(`/user/${handle}`);
        return;
      }

      // Link lens publication id to database publication id
      promises.push(
        linkPublication({
          lensId: publication.id,
          publicationId,
        })
      );

      await Promise.all(promises);

      // Redirect to space
      router.push(`/space/${publication.id}`);
    } catch (error) {
      console.error(error);
      setLoading(false);
    }
  }

  const image = imageFile ? URL.createObjectURL(imageFile) : null;

  return (
    <div className="space-y-8">
      <div className="flex flex-col items-center space-y-1">
        <h1 className="flex justify-center text-4xl font-bold">
          Publish Space
        </h1>
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
          rows={4}
          defaultValue={description}
        />

        <div className="space-y-4">
          <div className="text-lg font-bold">Image</div>

          <div className="relative aspect-card h-full w-full rounded-xl bg-primaryContainer">
            {image && (
              <img
                src={image}
                alt="picture preview"
                className="h-full w-full rounded-xl"
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
          {signer ? "Submit" : "Sign In"}
        </Button>
      </div>
    </div>
  );
}
