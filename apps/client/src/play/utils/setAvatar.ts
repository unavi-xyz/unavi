import { useClientStore } from "@unavi/engine";

import { getTempUpload } from "@/app/api/temp/helper";
import { usePlayStore } from "@/app/play/playStore";
import { cdnURL, S3Path } from "@/src/utils/s3Paths";

import { LocalStorageKey } from "../constants";

export async function setAvatar(value: string | null) {
  if (useClientStore.getState().avatar === value) return;

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
  usePlayStore.setState({ uiAvatar: avatar });
  useClientStore.getState().setAvatar(avatar);
  localStorage.setItem(LocalStorageKey.Avatar, avatar);
}
