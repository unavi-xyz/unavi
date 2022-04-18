import { useEffect, useRef } from "react";
import { AudioListener, Group, PositionalAudio } from "three";
import { useThree } from "@react-three/fiber";
import { useAvatar, useIpfsFile, useProfile } from "ceramic";
import { Avatar } from "3d";

import { PlayerChannels } from "../../helpers/types";
import { DEFAULT_AVATAR } from "./helpers/constants";

import useInterpolation from "./hooks/useInterpolation";
import useDataChannels from "./hooks/useDataChannels";

interface Props {
  id: string;
  channels: PlayerChannels;
  track: MediaStreamTrack;
}

export default function OtherPlayer({ id, channels, track }: Props) {
  const groupRef = useRef<Group>();

  const { transformRef, identity } = useDataChannels(id, channels);

  const { profile } = useProfile(identity?.did);
  const { avatar } = useAvatar(profile?.avatar ?? DEFAULT_AVATAR);
  const { url } = useIpfsFile(avatar?.vrm);
  const animationWeights = useInterpolation(groupRef, transformRef);

  const { camera } = useThree();

  useEffect(() => {
    if (!track) return;

    function startAudio() {
      const listener = new AudioListener();
      const positional = new PositionalAudio(listener);

      camera.add(listener);
      groupRef.current.add(positional);

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
      {url && (
        <Avatar
          src={url}
          animationsSrc="/models/animations.fbx"
          animationWeights={animationWeights}
        />
      )}
    </group>
  );
}
