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
import { useMemo, useRef, useState } from "react";
import { useSigner } from "wagmi";

import { useCreatePost } from "../client/lens/hooks/useCreatePost";
import { useLens } from "../client/lens/hooks/useLens";
import { useProfileByHandle } from "../client/lens/hooks/useProfileByHandle";
import { trpc } from "../client/trpc";
import { cropImage } from "../editor/utils/cropImage";
import { env } from "../env/client.mjs";
import Button from "../ui/Button";
import FileInput from "../ui/FileInput";
import TextArea from "../ui/TextArea";
import TextField from "../ui/TextField";

function cdnModelURL(id: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/published/${id}/model.glb`;
}

function cdnImageURL(id: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/published/${id}/image.jpg`;
}

function cdnMetadataURL(id: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/published/${id}/metadata.json`;
}

export default function CreateAvatarPage() {
  const router = useRouter();
  const { handle, client } = useLens();
  const { data: signer } = useSigner();
  const { openConnectModal } = useConnectModal();
  const profile = useProfileByHandle(handle);
  const createPost = useCreatePost(profile?.id);

  const nameRef = useRef<HTMLInputElement>(null);
  const descriptionRef = useRef<HTMLTextAreaElement>(null);

  const [loading, setLoading] = useState(false);
  const [imageFile, setImageFile] = useState<File>();
  const [vrmFile, setVrmFile] = useState<File>();

  const { mutateAsync: createModelUploadUrl } =
    trpc.publication.modelUploadURL.useMutation();

  const { mutateAsync: createImageUploadUrl } =
    trpc.publication.imageUploadURL.useMutation();

  const { mutateAsync: createMetadataUploadUrl } =
    trpc.publication.metadataUploadURL.useMutation();

  const { mutateAsync: createPublication } =
    trpc.publication.create.useMutation();

  const { mutateAsync: linkPublication } = trpc.publication.link.useMutation();

  const imageURL = useMemo(() => {
    if (!imageFile) return;
    return URL.createObjectURL(imageFile);
  }, [imageFile]);

  async function handleCreate() {
    if (loading || !imageFile || !vrmFile) return;

    if (!signer) {
      if (openConnectModal) openConnectModal();
      return;
    }

    const name = nameRef.current?.value ?? "";
    const description = descriptionRef.current?.value ?? "";

    try {
      setLoading(true);
      const promises: Promise<void>[] = [];

      // Create database publication
      const publicationId = await createPublication({ type: "AVATAR" });

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
                "Content-Type": imageFile.type,
                "x-amz-acl": "public-read",
              },
            });

            if (!response.ok) reject();
            else resolve();
          }

          upload();
        })
      );

      // Upload VRM to S3
      promises.push(
        new Promise((resolve, reject) => {
          async function upload() {
            if (!vrmFile) {
              reject();
              return;
            }

            // Get VRM
            const res = await fetch(URL.createObjectURL(vrmFile));
            const body = await res.blob();

            // Upload to S3
            const url = await createModelUploadUrl({ id: publicationId });
            const response = await fetch(url, {
              method: "PUT",
              body,
              headers: {
                "Content-Type": vrmFile.type,
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
              tags: ["vrm", "avatar", "wired"],
              mainContentFocus: PublicationMainFocus.Image,
              external_url: `https://thewired.space/user/${handle}`,
              name,
              image: imageURL,
              imageMimeType: "image/jpeg",
              media,
              attributes: [],
              appId: AppId.Avatar,
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

      if (!profile) {
        router.push("/create/avatars");
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
              sources: [AppId.Avatar],
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
        router.push("/create/avatars");
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
      router.push(`/avatar/${publication.id}`);
    } catch (err) {
      console.error(err);
      setLoading(false);
    }
  }

  return (
    <div className="space-y-4">
      <div className="text-center text-3xl font-bold">New Avatar</div>

      <TextField
        inputRef={nameRef}
        title="Name"
        defaultValue="My Avatar"
        outline
      />

      <TextArea
        textAreaRef={descriptionRef}
        autoComplete="off"
        title="Description"
        outline
        rows={4}
      />

      <div className="text-lg font-bold">Image</div>

      {imageURL && (
        <div className="flex justify-center">
          <div className="aspect-vertical w-1/3 overflow-hidden rounded-xl">
            <img src={imageURL} alt="avatar image" className="h-full w-full" />
          </div>
        </div>
      )}

      <FileInput
        accept="image/*"
        onChange={(e) => {
          const file = e.target.files?.[0];
          if (!file) return;

          cropImage(URL.createObjectURL(file), 3 / 5, false).then(setImageFile);
        }}
      />

      <div className="text-lg font-bold">VRM File</div>

      <FileInput
        accept=".vrm"
        onChange={(e) => {
          const file = e.target.files?.[0];
          if (file) setVrmFile(file);
        }}
      />

      <div className="flex justify-end">
        <Button
          variant="filled"
          onClick={handleCreate}
          loading={loading}
          disabled={loading || !imageFile || !vrmFile}
        >
          Create
        </Button>
      </div>
    </div>
  );
}
