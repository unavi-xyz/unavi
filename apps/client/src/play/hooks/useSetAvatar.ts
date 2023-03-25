import { useEngine, useWebSocket } from "@wired-labs/react-client";

import { getTempUpload } from "../../../app/api/temp/helper";
import { usePlayStore } from "../../../app/play/[id]/store";
import { env } from "../../env/client.mjs";
import { LocalStorageKey } from "../constants";

export function tempURL(fileId: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/temp/${fileId}`;
}

export function useSetAvatar() {
  const engine = useEngine();
  const { send } = useWebSocket();

  async function setAvatar(avatar: string) {
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
    send({ subject: "set_avatar", data: avatarURL });

    // Save to local storage
    localStorage.setItem(LocalStorageKey.Avatar, avatarURL);
  }

  return setAvatar;
}
