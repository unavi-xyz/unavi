import { Triplet } from "@react-three/cannon";
import { useThree } from "@react-three/fiber";
import { useContext, useEffect, useRef, useState } from "react";
import { AudioListener, Group, PositionalAudio } from "three";

import { useFetchData } from "@wired-xr/ipfs";
import { useAvatarUrlFromProfile, useProfileByHandle } from "@wired-xr/lens";

import { Avatar } from "../avatar";
import { LocationMessageSchema, NetworkingContext } from "../networking";
import { PlayerLocation } from "../networking/types";
import { useAnimationWeights } from "./useAnimationWeights";
import { useApplyLocation } from "./useApplyLocation";
import { useInterpolateLocation } from "./useInterpolateLocation";

interface Props {
  id: string;
  animationsUrl: string;
  defaultAvatarUrl: string;
}

export function OtherPlayer({ id, animationsUrl, defaultAvatarUrl }: Props) {
  const groupRef = useRef<Group>(null);
  const locationRef = useRef<PlayerLocation>({
    position: [0, 0, 0],
    rotation: 0,
  });

  const [handle, setHandle] = useState<string>();

  const { otherPlayers } = useContext(NetworkingContext);

  const profile = useProfileByHandle(handle);
  const avatarUrl = useAvatarUrlFromProfile(profile);
  const fetchedAvatar = useFetchData(avatarUrl);
  const interpolatedLocation = useInterpolateLocation(locationRef);
  const animationWeights = useAnimationWeights(groupRef, interpolatedLocation);

  const { camera } = useThree();

  useApplyLocation(groupRef, interpolatedLocation);

  useEffect(() => {
    if (!otherPlayers) return;

    const consumer = otherPlayers[id].dataConsumer;
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
      consumer.off("message", onMessage);
    };
  }, [otherPlayers, id]);

  useEffect(() => {
    if (!otherPlayers) return;

    const consumer = otherPlayers[id].audioConsumer;
    if (!consumer) return;

    const stream = new MediaStream();
    stream.addTrack(consumer.track);

    function startAudio() {
      const listener = new AudioListener();
      const positional = new PositionalAudio(listener);

      camera.add(listener);
      groupRef.current?.add(positional);

      const audio = new Audio();
      audio.srcObject = stream;

      positional.setMediaStreamSource(audio.srcObject);
    }

    document.addEventListener("click", startAudio, { once: true });
  }, [camera, otherPlayers]);

  return (
    <group ref={groupRef}>
      <Avatar
        src={fetchedAvatar ?? defaultAvatarUrl}
        animationsSrc={animationsUrl}
        animationWeights={animationWeights}
      />
    </group>
  );
}
