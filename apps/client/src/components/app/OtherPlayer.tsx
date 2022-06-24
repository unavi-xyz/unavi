import { useContext, useEffect, useRef, useState } from "react";
import { Group } from "three";

import { Avatar } from "@wired-xr/engine";

import { useAnimationWeights } from "../../helpers/app/hooks/useAnimationWeights";
import { useApplyLocation } from "../../helpers/app/hooks/useApplyLocation";
import { useInterpolateLocation } from "../../helpers/app/hooks/useInterpolateLocation";
import { PlayerLocation } from "../../helpers/app/types";
import { RecievedWebsocketMessage } from "../../helpers/host/types";
import { useAvatarUrlFromProfile } from "../../helpers/lens/hooks/useAvatarFromProfile";
import { useProfileByHandle } from "../../helpers/lens/hooks/useProfileByHandle";
import { SpaceContext } from "./SpaceProvider";

export const DEFAULT_AVATAR_URL = "/models/Default.vrm";
export const ANIMATIONS_URL = "/models/animations.fbx";

interface Props {
  id: string;
  track?: MediaStreamTrack;
}

export default function OtherPlayer({ id, track }: Props) {
  const groupRef = useRef<Group>(null);
  const locationRef = useRef<PlayerLocation>({
    position: [0, 0, 0],
    rotation: 0,
  });

  const [handle, setHandle] = useState<string>();

  const { socket } = useContext(SpaceContext);
  const profile = useProfileByHandle(handle);
  const avatarUrl = useAvatarUrlFromProfile(profile);
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
        src={avatarUrl ?? DEFAULT_AVATAR_URL}
        animationsSrc={ANIMATIONS_URL}
        animationWeights={animationWeights}
      />
    </group>
  );
}
