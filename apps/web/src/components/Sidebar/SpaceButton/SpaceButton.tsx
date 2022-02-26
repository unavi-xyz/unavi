import { useEffect, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/router";
import { useSpace } from "ceramic";

import SidebarButton from "../SidebarButton/SidebarButton";

interface Props {
  streamId: string;
}

export default function SpaceButton({ streamId }: Props) {
  const router = useRouter();

  const space = useSpace(streamId);

  const [selected, setSelected] = useState(false);

  useEffect(() => {
    if (router.query.id === streamId) setSelected(true);
    else setSelected(false);
  }, [router.query.id, streamId]);

  return (
    <Link href={`/space/${streamId}`} passHref>
      <span>
        <SidebarButton
          icon={space?.name?.charAt(0)}
          image={space?.image}
          selected={selected}
          dark
        />
      </span>
    </Link>
  );
}
