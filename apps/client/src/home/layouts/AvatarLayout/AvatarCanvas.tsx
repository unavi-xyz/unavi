import { Triplet } from "@wired-labs/engine";

interface Props {
  url: string;
  background?: boolean;
  cameraPosition?: Triplet;
}

export default function AvatarCanvas({
  url,
  background = true,
  cameraPosition = [0, 1.6, 1.6],
}: Props) {
  return (
    <div>
      <canvas></canvas>
    </div>
  );
}
