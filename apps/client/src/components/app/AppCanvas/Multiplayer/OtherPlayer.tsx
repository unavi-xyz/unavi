import { useContext, useEffect, useRef, useState } from "react";
import { useFrame } from "@react-three/fiber";
import { Group, MathUtils, Vector3 } from "three";
import { Avatar } from "3d";

import { PUBLISH_INTERVAL } from "./constants";
import { MultiplayerContext } from "../../MultiplayerProvider";
import { useAvatar, useIpfsFile, useProfile } from "ceramic";

const defaultAvatar =
  "kjzl6cwe1jw1495s2wbkxyf0d7a4a5k82980jms3m1utm0yvmaev8s1dhmv20qv";

interface Props {
  id: string;
}

export default function OtherPlayer({ id }: Props) {
  const { getLocations } = useContext(MultiplayerContext);

  const { profile } = useProfile(id);
  const { avatar } = useAvatar(profile?.avatar ?? defaultAvatar);
  const vrmFile = useIpfsFile(avatar?.vrm);

  const [vrmUrl, setVrmUrl] = useState<string>();

  useEffect(() => {
    if (!vrmFile) {
      setVrmUrl(undefined);
      return;
    }

    const url = URL.createObjectURL(vrmFile);
    setVrmUrl(url);
  }, [vrmFile]);

  const walkWeight = useRef(0);
  const jumpWeight = useRef(0);
  const deltaTotal = useRef(0);
  const ref = useRef<Group>();
  const tempVector3 = useRef(new Vector3());
  const prev = useRef({ position: new Vector3(), rotation: 0 });
  const real = useRef({ position: new Vector3(), rotation: 0 });

  useEffect(() => {
    const interval = setInterval(() => {
      const locations = getLocations();
      const location = locations[id];

      prev.current.position.copy(real.current.position);
      prev.current.rotation = real.current.rotation;

      real.current.position.fromArray(location.position);
      real.current.rotation = location.rotation;

      deltaTotal.current = 0;
    }, PUBLISH_INTERVAL);

    return () => {
      clearInterval(interval);
    };
  }, [getLocations, id]);

  useFrame((_, delta) => {
    deltaTotal.current += delta;
    const alpha = Math.min(deltaTotal.current * (1000 / PUBLISH_INTERVAL), 1);

    //animations
    const velocity = tempVector3.current
      .subVectors(real.current.position, ref.current.position)
      .divideScalar(delta);

    walkWeight.current = Math.abs(velocity.x) + Math.abs(velocity.z);
    jumpWeight.current = Math.abs(velocity.y);

    //position interp
    tempVector3.current.lerpVectors(
      prev.current.position,
      real.current.position,
      alpha
    );
    ref.current.position.copy(tempVector3.current);

    //rotation interp
    function normalize(angle: number) {
      while (angle < 0) {
        angle += 2 * Math.PI;
      }
      while (angle >= 2 * Math.PI) {
        angle -= 2 * Math.PI;
      }
      return angle;
    }

    const start = prev.current.rotation;
    let end = real.current.rotation;

    const forward = start - end;
    const backward = end - start;

    if (normalize(forward) < normalize(backward)) {
      if (end > start) end -= 2 * Math.PI;
    } else {
      if (end < start) end += 2 * Math.PI;
    }

    const rot = MathUtils.lerp(start, end, alpha);
    ref.current.rotation.y = rot;
  });

  return (
    <group ref={ref}>
      {vrmUrl && (
        <Avatar
          src={vrmUrl}
          animationsSrc="/models/animations.fbx"
          walkWeight={walkWeight}
          jumpWeight={jumpWeight}
        />
      )}
    </group>
  );
}
