import { useClientStore } from "@unavi/react-client";

import { getTempUpload } from "@/app/api/temp/helper";
import { usePlayStore } from "@/app/play/store";

import { cdnURL, S3Path } from "../../utils/s3Paths";
import { LocalStorageKey } from "../constants";

/**
 * Wraps around the client's setAvatar function to support uploading avatars to S3.
 */
export function useSetAvatar() {
  async function setAvatar(value: string | null) {
    usePlayStore.setState({ didChangeAvatar: false });

    let avatar = value || "";

    // Upload avatar to S3 if it's a local uri
    if (avatar && !avatar.startsWith("http")) {
      // Get avatar file
      const [blob, { url, fileId }] = await Promise.all([
        fetch(avatar).then((res) => res.blob()),
        getTempUpload(),
      ]);

      // Upload to S3
      const res = await fetch(url, {
        body: blob,
        headers: { "Content-Type": blob.type, "x-amz-acl": "public-read" },
        method: "PUT",
      });
      if (!res.ok) throw new Error("Failed to upload avatar");

      avatar = cdnURL(S3Path.temp(fileId));
    }

    // Set avatar
    usePlayStore.setState({ avatar });
    useClientStore.setState({ avatar });
    localStorage.setItem(LocalStorageKey.Avatar, avatar);
  }

  return setAvatar;
}
