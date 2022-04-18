import { useIpfsFile, useProfile, useSpace } from "ceramic";
import { useAtomValue, useSetAtom } from "jotai";
import Link from "next/link";
import { useRouter } from "next/router";
import { useState } from "react";

import { pageAtom, spaceAtom, userAtom } from "../helpers/atoms";

export default function SpacePage() {
  const router = useRouter();
  const currentSpace = router.query.space as string;

  const spaceId = useAtomValue(spaceAtom);

  const setPage = useSetAtom(pageAtom);
  const setUser = useSetAtom(userAtom);

  const [inSpace, setInSpace] = useState(false);

  const { space, controller } = useSpace(spaceId);
  const { url } = useIpfsFile(space?.image);
  const { profile } = useProfile(controller);

  function handleAuthorClick() {
    setPage("User");
    setUser(controller);
  }

  return (
    <div className="space-y-8">
      <div>
        <div className="flex justify-center text-2xl">{space?.name}</div>
        <div className="flex justify-center text-lg space-x-1 text-neutral-500">
          <div>By</div>
          <div
            onClick={handleAuthorClick}
            className="hover:underline cursor-pointer"
          >
            {profile?.name ?? controller?.substring(0, 10)}
          </div>
        </div>
      </div>

      {url && (
        <img
          src={url}
          alt="space image"
          className="object-cover w-full h-80 rounded-2xl"
        />
      )}

      <div>{space?.description}</div>

      {currentSpace === spaceId ? null : (
        <Link href={`/app?space=${spaceId}`} passHref>
          <div
            className="h-14 text-md rounded-full bg-black text-white justify-center
          hover:cursor-pointer transition-all flex items-center"
          >
            Enter Space
          </div>
        </Link>
      )}
    </div>
  );
}
