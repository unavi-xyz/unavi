import { useThree } from "@react-three/fiber";
import { useEffect, useRef } from "react";
import { AudioListener, Group, PositionalAudio } from "three";

import useDataChannels from "../../helpers/app/hooks/useDataChannels";
import useInterpolation from "../../helpers/app/hooks/useInterpolation";
import { PlayerChannels } from "../../helpers/app/types";

interface Props {
  id: string;
  channels: PlayerChannels;
  track: MediaStreamTrack | undefined;
}

export default function OtherPlayer({ id, channels, track }: Props) {
  const groupRef = useRef<Group>(null);

  const { locationRef, identity } = useDataChannels(id, channels);

  // const { profile } = useProfile(identity?.did);
  // const { avatar } = useAvatar(profile?.avatar ?? DEFAULT_AVATAR);
  // const { url } = useIpfsFile(avatar?.vrm);
  const animationWeights = useInterpolation(groupRef, locationRef);

  const { camera } = useThree();

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
      {/* {url && (
        <Avatar
          src={url}
          animationsSrc="/models/animations.fbx"
          animationWeights={animationWeights}
        />
      )} */}

      <mesh>
        <boxBufferGeometry attach="geometry" args={[1, 1.6, 1]} />
      </mesh>
    </group>
  );
}
