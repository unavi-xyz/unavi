import { getTempUpload } from "../../../app/api/temp/helper";
import { env } from "../../env/client.mjs";
import { usePlayStore } from "../../play/store";
import { LocalStorageKey } from "../constants";
import { sendToHost } from "./useHost";

export function tempURL(fileId: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/temp/${fileId}`;
}

export function useSetAvatar() {
  async function setAvatar(avatar: string) {
    const { engine } = usePlayStore.getState();
    if (!engine) return;

    usePlayStore.setState({ didChangeAvatar: false });

    let avatarURL = avatar;

    const isUrl = avatarURL.startsWith("http");
    if (!isUrl) {
      // Get avatar file
      const body = await fetch(avatar).then((res) => res.blob());
      const { url, fileId } = await getTempUpload();

      // Upload to S3
      const res = await fetch(url, {
        method: "PUT",
        body,
        headers: { "Content-Type": body.type, "x-amz-acl": "public-read" },
      });
      if (!res.ok) throw new Error("Failed to upload avatar");

      avatarURL = tempURL(fileId);
    }

    usePlayStore.setState({ avatar: avatarURL });

    // Update engine
    engine.render.send({ subject: "set_user_avatar", data: avatarURL });

    // Publish avatar
    sendToHost({ subject: "set_avatar", data: avatarURL });

    // Save to local storage
    localStorage.setItem(LocalStorageKey.Avatar, avatarURL);
  }

  return setAvatar;
}
