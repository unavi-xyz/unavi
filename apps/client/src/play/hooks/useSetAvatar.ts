import { useClient } from "@wired-labs/react-client";

import { getTempUpload } from "../../../app/api/temp/helper";
import { usePlayStore } from "../../../app/play/[id]/store";
import { cdnURL } from "../../utils/s3Paths";
import { S3Path } from "../../utils/s3Paths";

/**
 * Wraps around the client's setAvatar function to support uploading avatars to S3.
 */
export function useSetAvatar() {
  const { setAvatar: clientSetAvatar } = useClient();

  async function setAvatar(avatar: string | null) {
    usePlayStore.setState({ didChangeAvatar: false });

    if (avatar) {
      let avatarURL = avatar;

      // Upload avatar to S3 if it's a local uri
      if (!avatarURL.startsWith("http")) {
        // Get avatar file
        const [blob, { url, fileId }] = await Promise.all([
          fetch(avatar).then((res) => res.blob()),
          getTempUpload(),
        ]);

        // Upload to S3
        const res = await fetch(url, {
          method: "PUT",
          body: blob,
          headers: { "Content-Type": blob.type, "x-amz-acl": "public-read" },
        });
        if (!res.ok) throw new Error("Failed to upload avatar");

        avatarURL = cdnURL(S3Path.temp(fileId));
      }

      // Update client
      clientSetAvatar(avatarURL);
    } else {
      // Update client
      clientSetAvatar(null);
    }
  }

  return setAvatar;
}
