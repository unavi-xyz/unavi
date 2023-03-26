import { useClient } from "@wired-labs/react-client";

import { getTempUpload } from "../../../app/api/temp/helper";
import { usePlayStore } from "../../../app/play/[id]/store";
import { cdnURL } from "../../utils/s3Paths";
import { S3Path } from "../../utils/s3Paths";
import { LocalStorageKey } from "../constants";

/**
 * Wraps around the client's setAvatar function to update local storage,
 * and support uploading avatars to S3
 */
export function useSetAvatar() {
  const { setAvatar: clientSetAvatar } = useClient();

  async function setAvatar(avatar: string | null) {
    usePlayStore.setState({ didChangeAvatar: false });

    if (avatar) {
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

        avatarURL = cdnURL(S3Path.temp(fileId));
      }

      usePlayStore.setState({ avatar: avatarURL });

      // Update client
      clientSetAvatar(avatarURL);

      // Save to local storage
      localStorage.setItem(LocalStorageKey.Avatar, avatarURL);
    } else {
      // Remove avatar
      usePlayStore.setState({ avatar: null });

      // Update client
      clientSetAvatar(null);

      // Remove from local storage
      localStorage.removeItem(LocalStorageKey.Avatar);
    }
  }

  return setAvatar;
}
