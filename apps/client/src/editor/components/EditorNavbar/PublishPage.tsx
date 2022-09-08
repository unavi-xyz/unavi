import produce from "immer";
import { atom, useAtomValue } from "jotai";
import { nanoid } from "nanoid";
import { useRouter } from "next/router";
import { useContext, useRef, useState } from "react";

import Button from "../../../ui/base/Button";
import FileUpload from "../../../ui/base/FileUpload";
import TextArea from "../../../ui/base/TextArea";
import TextField from "../../../ui/base/TextField";
import { crop } from "../../../utils/crop";
import { useEditorStore } from "../../store";
import HostPage from "./HostPage";

export const didSetHostAtom = atom(false);

export default function PublishPage() {
  return null;
  // const project = useProject();
  // const nameRef = useRef<HTMLInputElement>(null);
  // const descriptionRef = useRef<HTMLTextAreaElement>(null);

  // const { uploadFileToIpfs } = useContext(IpfsContext);
  // const { handle } = useContext(LensContext);

  // const rootHandle = useEditorStore((state) => state.rootHandle);
  // const [imageFile, setImageFile] = useState<File>();
  // const [loading, setLoading] = useState(false);

  // const router = useRouter();
  // const profile = useProfileByHandle(handle);
  // const createPost = useCreatePost(profile?.id);

  // const didSetHost = useAtomValue(didSetHostAtom);
  // const host = profile?.attributes?.find((item) => item.key === "host");
  // const disableSubmit = !imageFile || !handle;

  // async function handleSubmit() {
  //   if (loading || disableSubmit || !project?.scene || !rootHandle) return;

  //   setLoading(true);

  //   try {
  //     //upload image to IPFS
  //     const cropped = await crop(URL.createObjectURL(imageFile), 5 / 3);
  //     const imageURI = await uploadFileToIpfs(cropped);
  //     if (!imageURI) throw new Error("Failed to upload image");

  //     //upload scene assets to IPFS
  //     const finalScene = await produce(project.scene, async (draft) => {
  //       await Promise.all(
  //         Object.entries(draft.assets).map(async ([key, asset]) => {
  //           const fileHandle = await getFileByPath(asset.uri, rootHandle);
  //           if (!fileHandle) return;
  //           const file = await fileHandle.getFile();
  //           if (!file) return;

  //           const uri = await uploadFileToIpfs(file);
  //           if (!uri) throw new Error("Failed to upload file");
  //           draft.assets[key].uri = uri;
  //         })
  //       );
  //     });

  //     //create metadata
  //     const metadata: Metadata = {
  //       version: MetadataVersions.one,
  //       metadata_id: nanoid(),
  //       name: nameRef.current?.value ?? "",
  //       description: descriptionRef.current?.value ?? "",
  //       content: JSON.stringify(finalScene),
  //       image: imageURI,
  //       imageMimeType: imageFile.type,
  //       attributes: [],
  //       animation_url: undefined,
  //       external_url: "https://thewired.space",
  //       media: [{ item: imageURI, type: imageFile.type }],
  //       appId: AppId.Space,
  //     };

  //     //create post
  //     await createPost(metadata);

  //     router.push(`/user/${handle}`);
  //   } catch (err) {
  //     console.error(err);
  //   }

  //   setLoading(false);
  // }

  // if (profile && !host && !didSetHost) return <HostPage />;

  // return (
  //   <div className="space-y-8">
  //     <div className="flex flex-col items-center space-y-1">
  //       <h1 className="text-3xl flex justify-center">Publish Space</h1>
  //       <p className="text-lg flex justify-center">Mint a new space NFT</p>
  //     </div>

  //     <div className="space-y-4">
  //       <TextField
  //         inputRef={nameRef}
  //         autoComplete="off"
  //         title="Name"
  //         defaultValue={project?.name}
  //       />
  //       <TextArea
  //         textAreaRef={descriptionRef}
  //         autoComplete="off"
  //         title="Description"
  //         defaultValue={project?.description}
  //       />

  //       <div className="space-y-4">
  //         <div className="text-lg font-bold">Image</div>

  //         {imageFile && (
  //           <div className="w-full aspect-card">
  //             <img
  //               src={URL.createObjectURL(imageFile)}
  //               alt="cover picture preview"
  //               className="object-cover rounded-xl h-full w-full border"
  //             />
  //           </div>
  //         )}

  //         <FileUpload
  //           color="SurfaceVariant"
  //           title="Cover Picture"
  //           accept="image/*"
  //           onChange={(e) => {
  //             const file = e.target.files?.[0];
  //             if (file) setImageFile(file);
  //           }}
  //         />
  //       </div>
  //     </div>

  //     <div className="flex justify-end">
  //       <Button onClick={handleSubmit} variant="filled" disabled={disableSubmit} loading={loading}>
  //         Submit
  //       </Button>
  //     </div>
  //   </div>
  // );
}
