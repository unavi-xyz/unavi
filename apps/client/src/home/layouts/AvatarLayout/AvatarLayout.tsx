import Image from "next/image";
import Link from "next/link";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";

import { LocalStorageKey } from "../../../app/constants";
import { getAvatarPerformanceRank } from "../../../app/helpers/getAvatarPerformanceRank";
import { useLens } from "../../../client/lens/hooks/useLens";
import { PublicationProps } from "../../../client/lens/utils/getPublicationProps";
import { trimHandle } from "../../../client/lens/utils/trimHandle";
import { GLTFStats } from "../../../server/helpers/getGltfStats";
import Button from "../../../ui/Button";
import NavigationTab from "../../../ui/NavigationTab";
import { isFromCDN } from "../../../utils/isFromCDN";
import MetaTags from "../../MetaTags";

interface Props extends PublicationProps {
  stats: GLTFStats;
  children: React.ReactNode;
}

export default function AvatarLayout({
  stats,
  children,
  metadata,
  image,
  publication,
}: Props) {
  const router = useRouter();
  const id = router.query.id as string;

  const { handle } = useLens();
  const author = trimHandle(publication?.profile.handle);
  const isAuthor = handle && handle === author;

  const [isEquipped, setIsEquipped] = useState(false);

  useEffect(() => {
    setIsEquipped(localStorage.getItem(LocalStorageKey.AvatarId) === id);
  }, [id]);

  function handleEquipAvatar() {
    localStorage.setItem(LocalStorageKey.AvatarId, id);
    setIsEquipped(true);
  }

  function handleUnequipAvatar() {
    localStorage.removeItem(LocalStorageKey.AvatarId);
    setIsEquipped(false);
  }

  const performanceRank = getAvatarPerformanceRank(stats);

  return (
    <>
      <MetaTags
        title={metadata.title ?? id}
        description={metadata.description ?? undefined}
        image={metadata.image ?? undefined}
        card="summary_large_image"
      />

      <div className="mx-4 h-full">
        <div className="max-w-content mx-auto h-full w-full space-y-8 py-8">
          <div className="flex flex-col items-center space-y-8 md:flex-row md:items-stretch md:space-y-0 md:space-x-8">
            <div className="aspect-vertical h-full w-1/2 rounded-3xl bg-sky-100">
              <div className="relative h-full w-full object-cover">
                {image &&
                  (isFromCDN(image) ? (
                    <Image
                      src={image}
                      priority
                      fill
                      sizes="425px"
                      alt=""
                      className="rounded-3xl object-cover"
                    />
                  ) : (
                    <img
                      src={image}
                      alt=""
                      className="h-full w-full rounded-3xl object-cover"
                      crossOrigin="anonymous"
                    />
                  ))}
              </div>
            </div>

            <div className="flex min-w-fit flex-col justify-between space-y-8 md:w-2/3">
              <div className="space-y-4">
                <div className="flex justify-center text-3xl font-black">
                  {publication?.metadata.name}
                </div>

                <div className="flex justify-center space-x-1 font-bold md:justify-start">
                  <div className="text-neutral-500">By</div>
                  <Link href={`/user/${author}`}>
                    <div className="cursor-pointer hover:underline">
                      @{author}
                    </div>
                  </Link>
                </div>

                <div className="flex justify-center space-x-1 font-bold md:justify-start">
                  <div className="text-neutral-500">Performance</div>
                  <a
                    href="https://docs.thewired.space/avatars#-perfomance-ranks"
                    target="_blank"
                    rel="noreferrer"
                    className="cursor-pointer hover:underline"
                  >
                    {performanceRank}
                  </a>
                </div>
              </div>

              {isEquipped ? (
                <div className="group">
                  <Button
                    variant="outlined"
                    fullWidth
                    onClick={handleUnequipAvatar}
                  >
                    <div className="py-2 group-hover:hidden">
                      Avatar Equipped
                    </div>
                    <div className="hidden py-2 group-hover:block">Unequip</div>
                  </Button>
                </div>
              ) : (
                <Button variant="filled" fullWidth onClick={handleEquipAvatar}>
                  <div className="py-2">Equip Avatar</div>
                </Button>
              )}
            </div>
          </div>

          <div className="space-y-4 pb-4">
            <div className="flex space-x-4">
              <NavigationTab href={`/avatar/${id}`} text="About" />

              {isAuthor && (
                <NavigationTab
                  href={`/avatar/${id}/settings`}
                  text="Settings"
                />
              )}
            </div>

            <div>{children}</div>
          </div>
        </div>
      </div>
    </>
  );
}
