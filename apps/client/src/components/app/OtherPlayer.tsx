import { useThree } from "@react-three/fiber";
import { useEffect, useRef } from "react";
import { AudioListener, Group, PositionalAudio } from "three";

import { Avatar } from "@wired-xr/avatar";

import { useAnimationWeights } from "../../helpers/app/hooks/useAnimationWeights";
import { useApplyLocation } from "../../helpers/app/hooks/useApplyLocation";
import useDataChannels from "../../helpers/app/hooks/useDataChannels";
import { useInterpolateLocation } from "../../helpers/app/hooks/useInterpolateLocation";
import { PlayerChannels } from "../../helpers/app/types";
import { useIpfsUrl } from "../../helpers/ipfs/useIpfsUrl";
import { useProfileByHandle } from "../../helpers/lens/hooks/useProfileByHandle";
import { usePublication } from "../../helpers/lens/hooks/usePublication";

export const DEFAULT_AVATAR_URL = "/models/Default.vrm";
export const ANIMATIONS_URL = "/models/animations.fbx";

interface Props {
  id: string;
  channels?: Partial<PlayerChannels>;
  track?: MediaStreamTrack;
}

export default function OtherPlayer({ id, channels, track }: Props) {
  const groupRef = useRef<Group>(null);

  const { camera } = useThree();
  const { locationRef, identity } = useDataChannels(id, channels);
  const profile = useProfileByHandle(identity?.handle);
  const interpolatedLocation = useInterpolateLocation(locationRef);
  const animationWeights = useAnimationWeights(groupRef, interpolatedLocation);
  useApplyLocation(groupRef, interpolatedLocation);

  const avatarId = profile?.attributes?.find(
    (item) => item.key === "avatar"
  )?.value;
  const avatar = usePublication(avatarId);
  const avatarUri = avatar?.metadata.content;
  const avatarUrl = useIpfsUrl(avatarUri);

  useEffect(() => {
    if (!track) return;

    function startAudio() {
      if (!track) return;

      const listener = new AudioListener();
      const positional = new PositionalAudio(listener);

      camera.add(listener);
      groupRef.current?.add(positional);

      const stream = new MediaStream();
      stream.addTrack(track);

      const el = document.createElement("audio");
      el.srcObject = stream;

      positional.setMediaStreamSource(el.srcObject);
    }

    document.addEventListener("click", startAudio, { once: true });
  }, [camera, track]);

  return (
    <group ref={groupRef}>
      <Avatar
        src={avatarUrl ?? DEFAULT_AVATAR_URL}
        animationsSrc={ANIMATIONS_URL}
        animationWeights={animationWeights}
      />
    </group>
  );
}
