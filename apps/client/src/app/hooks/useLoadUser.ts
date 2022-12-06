import { useGetPublicationQuery } from "lens";
import { useEffect, useState } from "react";

import { useLens } from "../../client/lens/hooks/useLens";
import { useProfileByHandle } from "../../client/lens/hooks/useProfileByHandle";
import { parseUri } from "../../utils/parseUri";
import { LocalStorageKey } from "../constants";
import { useAppStore } from "../store";

export function useLoadUser() {
  const [avatarId, setAvatarId] = useState<string | null>(null);

  const engine = useAppStore((state) => state.engine);
  const { handle } = useLens();

  const profile = useProfileByHandle(handle);

  const [{ data: avatarPublication }] = useGetPublicationQuery({
    variables: { request: { publicationId: avatarId } },
    pause: !avatarId,
  });

  useEffect(() => {
    // Publish handle
    engine?.setHandle(handle ?? null);
  }, [engine, handle]);

  useEffect(() => {
    if (!engine) return;

    // Name
    const displayName = localStorage.getItem(LocalStorageKey.Name);
    useAppStore.setState({ displayName });
    engine.setName(displayName);

    // Avatar
    const customAvatar = localStorage.getItem(LocalStorageKey.Avatar);

    const equippedAvatar = profile?.attributes
      ? profile.attributes.find((attr) => attr.key === "avatar")
      : undefined;

    if (equippedAvatar?.value) {
      setAvatarId(equippedAvatar.value);

      useAppStore.setState({ customAvatar: null });
      engine.setAvatar(null);
    } else {
      useAppStore.setState({ customAvatar });
      engine.setAvatar(customAvatar);
    }
  }, [engine, profile]);

  useEffect(() => {
    async function fetchAvatar() {
      if (!engine || !avatarPublication) return;

      const avatarURI =
        avatarPublication?.publication?.metadata.media[1]?.original.url;
      if (!avatarURI) return;

      const url = parseUri(avatarURI);

      const response = await fetch(url);
      const blob = await response.blob();
      const customAvatar = URL.createObjectURL(blob);

      useAppStore.setState({ customAvatar });
      engine.setAvatar(avatarURI);
    }

    fetchAvatar();
  }, [engine, avatarPublication]);
}
