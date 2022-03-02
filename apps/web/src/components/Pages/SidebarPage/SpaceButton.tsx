import { useSpace } from "ceramic";
import SidebarButton from "./SidebarButton";

interface Props {
  streamId: string;
  selected: boolean;
}

export default function SpaceButton({ streamId, selected }: Props) {
  const { space } = useSpace(streamId);

  return (
    <SidebarButton
      tooltip={space?.name}
      icon={space?.name?.charAt(0)}
      image={space?.image}
      selected={selected}
      dark
    />
  );
}
