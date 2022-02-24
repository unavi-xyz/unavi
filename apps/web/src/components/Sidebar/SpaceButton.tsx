import { getImageUrl, useSpace } from "ceramic";
import Link from "next/link";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";

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

  const round = selected ? "rounded-xl" : "rounded-3xl";

  return (
    <div>
      <Link href={`/space/${streamId}`} passHref>
        {space?.image ? (
          <img
            src={getImageUrl(space?.image)}
            alt=""
            className={`object-cover relative flex items-center justify-center h-12 w-12 my-2 
                        mx-auto shadow-lg hover:cursor-pointer hover:rounded-xl transition-all
                        ease-linear text-xl ${round}`}
          />
        ) : (
          <div
            className={`object-cover relative flex items-center justify-center h-12 w-12 my-2 
                        mx-auto shadow-lg hover:cursor-pointer hover:rounded-xl transition-all
                        ease-linear text-xl bg-slate-800 ${round}`}
          >
            {space?.name?.charAt(0)}
          </div>
        )}
      </Link>
    </div>
  );
}
