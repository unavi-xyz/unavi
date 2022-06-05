import { useThree } from "@react-three/fiber";
import { useEffect, useRef } from "react";
import { AudioListener, Group, PositionalAudio } from "three";

import { Avatar } from "@wired-xr/avatar";

import useDataChannels from "../../helpers/app/hooks/useDataChannels";
import useInterpolation from "../../helpers/app/hooks/useInterpolation";
import { PlayerChannels } from "../../helpers/app/types";

const DEFAULT_AVATAR_URL = "/models/Default.vrm";
const ANIMATIONS_URL = "/models/animations.fbx";

interface Props {
  id: string;
  channels?: Partial<PlayerChannels>;
  track?: MediaStreamTrack;
}

export default function OtherPlayer({ id, channels, track }: Props) {
  const groupRef = useRef<Group>(null);

  const { camera } = useThree();
  const { locationRef } = useDataChannels(id, channels);
  const animationWeights = useInterpolation(groupRef, locationRef);

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
        src={DEFAULT_AVATAR_URL}
        animationsSrc={ANIMATIONS_URL}
        animationWeights={animationWeights}
      />
    </group>
  );
}
