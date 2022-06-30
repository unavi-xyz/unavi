import { Triplet } from "@react-three/cannon";
import { useContext, useEffect, useRef, useState } from "react";
import { Group } from "three";

import { Avatar } from "../avatar";
import { LocationMessageSchema, NetworkingContext } from "../networking";
import { PlayerLocation } from "../networking/types";
import { useAnimationWeights } from "./useAnimationWeights";
import { useApplyLocation } from "./useApplyLocation";
import { useInterpolateLocation } from "./useInterpolateLocation";

interface Props {
  id: string;
  track?: MediaStreamTrack;
  animationsUrl: string;
  defaultAvatarUrl: string;
}

export function OtherPlayer({
  id,
  track,
  animationsUrl,
  defaultAvatarUrl,
}: Props) {
  const groupRef = useRef<Group>(null);
  const locationRef = useRef<PlayerLocation>({
    position: [0, 0, 0],
    rotation: 0,
  });

  const [handle, setHandle] = useState<string>();

  const { dataConsumers } = useContext(NetworkingContext);

  // const profile = useProfileByHandle(handle);
  // const avatarUrl = useAvatarUrlFromProfile(profile);
  const interpolatedLocation = useInterpolateLocation(locationRef);
  const animationWeights = useAnimationWeights(groupRef, interpolatedLocation);

  useApplyLocation(groupRef, interpolatedLocation);

  useEffect(() => {
    if (!dataConsumers) return;

    const consumer = dataConsumers[id];
    if (!consumer) return;

    function onMessage(message: string) {
      const json = JSON.parse(message);

      try {
        const location = LocationMessageSchema.parse(json);

        //set location
        locationRef.current = {
          position: location.position as Triplet,
          rotation: location.rotation,
        };
      } catch {}
    }

    consumer.on("message", onMessage);
    return () => {
      consumer.on("message", onMessage);
    };
  }, [dataConsumers, id, handle]);

  // useEffect(() => {
  //   if (!track) return;

  //   function startAudio() {
  //     if (!track) return;

  //     const listener = new AudioListener();
  //     const positional = new PositionalAudio(listener);

  //     camera.add(listener);
  //     groupRef.current?.add(positional);

  //     const stream = new MediaStream();
  //     stream.addTrack(track);

  //     const el = document.createElement("audio");
  //     el.srcObject = stream;

  //     positional.setMediaStreamSource(el.srcObject);
  //   }

  //   document.addEventListener("click", startAudio, { once: true });
  // }, [camera, track]);

  return (
    <group ref={groupRef}>
      <Avatar
        src={defaultAvatarUrl}
        animationsSrc={animationsUrl}
        animationWeights={animationWeights}
      />
    </group>
  );
}
