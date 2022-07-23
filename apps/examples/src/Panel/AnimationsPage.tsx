import { AnimationAction } from "three";

import AnimationControl from "./AnimationControl";

interface Props {
  animations: AnimationAction[];
}

export default function AnimationsPage({ animations }: Props) {
  return (
    <div className="space-y-2">
      {animations.map((animation, index) => (
        <AnimationControl key={index} action={animation} />
      ))}
    </div>
  );
}
