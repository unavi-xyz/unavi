import { useContext, useEffect, useRef, useState } from "react";
import { Group } from "three";

import { Avatar } from "../avatar";
import { NetworkingContext } from "../networking";
import { PlayerLocation, RecievedWebsocketMessage } from "../networking/types";
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

  const { socket } = useContext(NetworkingContext);

  // const profile = useProfileByHandle(handle);
  // const avatarUrl = useAvatarUrlFromProfile(profile);
  const interpolatedLocation = useInterpolateLocation(locationRef);
  const animationWeights = useAnimationWeights(groupRef, interpolatedLocation);

  useApplyLocation(groupRef, interpolatedLocation);

  useEffect(() => {
    if (!socket) return;

    function onMessage(event: MessageEvent) {
      const { type, data } = JSON.parse(event.data) as RecievedWebsocketMessage;

      if (type === "location") {
        //check if this player
        if (data.userid === id) {
          //set handle
          if (handle !== data.handle) {
            setHandle(data.handle);
          }

          //set location
          locationRef.current = data.location;
        }
      }
    }

    socket.addEventListener("message", onMessage);
    return () => {
      socket.removeEventListener("message", onMessage);
    };
  }, [socket, id, handle]);

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
