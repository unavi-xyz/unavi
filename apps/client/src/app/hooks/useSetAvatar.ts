import { trpc } from "../../client/trpc";
import { env } from "../../env/client.mjs";
import { LocalStorageKey } from "../constants";
import { useAppStore } from "../store";

function getAvatarURL(fileId: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/temp/${fileId}`;
}

export function useSetAvatar() {
  const { mutateAsync: createTempUpload } = trpc.useMutation(
    "public.get-temp-upload"
  );

  async function setAvatar(avatar: string) {
    const { engine } = useAppStore.getState();
    if (!engine) return;

    useAppStore.setState({ didChangeAvatar: false });

    // Get avatar file
    const body = await fetch(avatar).then((res) => res.blob());
    const { url, fileId } = await createTempUpload();

    // Upload to S3
    const res = await fetch(url, {
      method: "PUT",
      body,
      headers: {
        "Content-Type": body.type,
        "x-amz-acl": "public-read",
      },
    });

    if (!res.ok) throw new Error("Failed to upload avatar");

    // Publish avatar
    const avatarURL = getAvatarURL(fileId);
    engine.setAvatar(avatarURL);

    // Save to local storage
    localStorage.setItem(LocalStorageKey.Avatar, avatarURL);
  }

  return setAvatar;
}
