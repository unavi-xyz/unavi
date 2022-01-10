import { useEffect, useState } from "react";

import { createAvatar } from "@dicebear/avatars";
import * as style from "@dicebear/avatars-identicon-sprites";

export function useIdenticon(seed: string | null) {
  const [profilePicture, setProfilePicture] = useState("");

  useEffect(() => {
    if (!seed) return;

    const image = createAvatar(style, {
      seed,
      dataUri: true,
      backgroundColor: "#ffffff",
    });

    setProfilePicture(image);
  }, [seed]);

  return profilePicture;
}
