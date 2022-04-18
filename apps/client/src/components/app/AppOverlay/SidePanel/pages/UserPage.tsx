import { useAtomValue, useSetAtom } from "jotai";
import { useIpfsFile, useProfile, useUserSpaces } from "ceramic";

import { pageAtom, spaceAtom, userAtom } from "../helpers/atoms";
import ProfileAvatar from "../../../../home/profile/ProfileAvatar";
import SpaceCard from "../../../../home/space/SpaceCard";

export default function UserPage() {
  const did = useAtomValue(userAtom);

  const setPage = useSetAtom(pageAtom);
  const setSpace = useSetAtom(spaceAtom);

  const { profile } = useProfile(did);
  const { url } = useIpfsFile(profile?.image?.original.src);
  const spaces = useUserSpaces(did);

  return (
    <div className="space-y-12">
      <div className="flex items-center space-x-8">
        <div className="w-24 h-24 flex-shrink-0">
          {url && (
            <img
              src={url}
              alt="profile picture"
              className="object-cover rounded-full w-full h-full"
            />
          )}
        </div>

        <div>
          <div className="text-2xl">{profile?.name}</div>
          <div className="text-neutral-400 break-all text-sm">{did}</div>
        </div>
      </div>

      <div className="bg-neutral-100 rounded-3xl h-96">
        <ProfileAvatar avatarId={profile?.avatar} />
      </div>

      <div className="space-y-2">
        <div className="text-xl flex justify-center">About</div>
        <div>{profile?.description}</div>
      </div>

      <div className="space-y-2">
        <div className="text-xl flex justify-center">Spaces</div>
        <div className="relative overflow-auto h-96">
          <div className="absolute h-full w-full top-0 left-0 px-4 pb-4 space-x-8 flex">
            {spaces?.map((streamId) => {
              return (
                <div
                  onClick={() => {
                    setPage("Space");
                    setSpace(streamId);
                  }}
                  key={streamId}
                >
                  <div className="flex-shrink-0 float-left w-64 h-full">
                    <SpaceCard streamId={streamId} />
                  </div>
                </div>
              );
            })}

            <div className="pr-2" />
          </div>
        </div>
      </div>
    </div>
  );
}
