import { nanoid } from "nanoid";
import { useRouter } from "next/router";
import { useRef, useState } from "react";

// import { useLocalSpace } from "../../../helpers/indexeddb/LocalSpace/hooks/useLocalSpace";
import { uploadFileToIpfs } from "../../../helpers/ipfs/fetch";
import { useCreatePost } from "../../../helpers/lens/hooks/useCreatePost";
import { useProfileByHandle } from "../../../helpers/lens/hooks/useProfileByHandle";
import { useLensStore } from "../../../helpers/lens/store";
import { AppId, Metadata, MetadataVersions } from "../../../helpers/lens/types";
import { useStudioStore } from "../../../helpers/studio/store";
import { crop } from "../../../helpers/utils/crop";
import Button from "../../base/Button";
import FileUpload from "../../base/FileUpload";
import TextArea from "../../base/TextArea";
import TextField from "../../base/TextField";

export default function PublishPage() {
  return null;

  // const nameRef = useRef<HTMLInputElement>(null);
  // const descriptionRef = useRef<HTMLTextAreaElement>(null);
  // const id = useStudioStore((state) => state.id);
  // const localSpace = useLocalSpace(id);
  // const handle = useLensStore((state) => state.handle);
  // const profile = useProfileByHandle(handle);
  // const router = useRouter();
  // const [imageFile, setImageFile] = useState<File>();
  // const [loading, setLoading] = useState(false);
  // const createPost = useCreatePost(profile?.id);
  // const image = imageFile ?? localSpace?.image ?? localSpace?.generatedImage;
  // async function handleSubmit() {
  //   if (!profile || !localSpace || loading || !image) return;
  //   setLoading(true);
  //   try {
  //     const cropped = await crop(URL.createObjectURL(image), 5 / 3);
  //     const imageURI = await uploadFileToIpfs(cropped);
  //     const metadata: Metadata = {
  //       version: MetadataVersions.one,
  //       metadata_id: nanoid(),
  //       name: nameRef.current?.value ?? "",
  //       description: descriptionRef.current?.value ?? "",
  //       content: JSON.stringify(localSpace.scene),
  //       image: imageURI,
  //       imageMimeType: image.type,
  //       attributes: [],
  //       animation_url: undefined,
  //       external_url: "https://thewired.space",
  //       media: [{ item: imageURI, type: image.type }],
  //       appId: AppId.space,
  //     };
  //     await createPost(metadata);
  //     router.push(`/user/${handle}`);
  //   } catch (err) {
  //     console.error(err);
  //   }
  //   setLoading(false);
  // }
  // return (
  //   <div className="space-y-8">
  //     <div className="flex flex-col items-center space-y-1">
  //       <h1 className="text-3xl flex justify-center">Publish</h1>
  //       <p className="text-lg flex justify-center">Mint a new space NFT</p>
  //     </div>
  //     <div className="w-full space-y-4">
  //       <div className="aspect-card rounded-2xl">
  //         {image && (
  //           <img
  //             src={URL.createObjectURL(image)}
  //             alt="space image"
  //             className="w-full h-full object-cover rounded-2xl"
  //           />
  //         )}
  //       </div>
  //       <div className="w-full space-y-1">
  //         <div className="text-lg font-bold">Image</div>
  //         <FileUpload
  //           title="Cover Picture"
  //           accept="image/*"
  //           color="SurfaceVariant"
  //           onChange={(e) => {
  //             const file = e.target.files?.[0];
  //             if (file) setImageFile(file);
  //           }}
  //         />
  //       </div>
  //       <TextField
  //         inputRef={nameRef}
  //         title="Name"
  //         defaultValue={localSpace?.name}
  //       />
  //       <TextArea
  //         textAreaRef={descriptionRef}
  //         title="Description"
  //         defaultValue={localSpace?.description}
  //       />
  //     </div>
  //     <div className="flex justify-end w-full">
  //       <Button variant="filled" onClick={handleSubmit} loading={loading}>
  //         Submit
  //       </Button>
  //     </div>
  //   </div>
  // );
}
