import { AppId, Post, useGetPublicationsQuery } from "lens";
import Link from "next/link";
import { useSession } from "next-auth/react";
import { useState } from "react";
import { MdAdd } from "react-icons/md";

import { useLens } from "../../client/lens/hooks/useLens";
import { useProfileByHandle } from "../../client/lens/hooks/useProfileByHandle";
import CreateAvatarPage from "../../home/CreateAvatarPage";
import { getNavbarLayout } from "../../home/layouts/NavbarLayout/NavbarLayout";
import AvatarCard from "../../home/lens/AvatarCard";
import Button from "../../ui/Button";
import Dialog from "../../ui/Dialog";

export default function Avatars() {
  const [openCreateDialog, setopenCreateDialog] = useState(false);

  const { handle } = useLens();
  const profile = useProfileByHandle(handle);
  const { status: authState } = useSession();
  const authenticated = authState === "authenticated";

  const [{ data }] = useGetPublicationsQuery({
    variables: { request: { profileId: profile?.id, sources: [AppId.Avatar] } },
    pause: !profile?.id,
  });

  function handleCreateAvatar() {
    if (!authenticated) return;
    setopenCreateDialog(true);
  }

  return (
    <>
      <Dialog open={openCreateDialog} onClose={() => setopenCreateDialog(false)}>
        <CreateAvatarPage />
      </Dialog>

      <div className="mx-4 flex justify-center py-8">
        <div className="max-w-content space-y-8">
          <div className="flex justify-center text-3xl font-black">Avatars</div>

          <div className="flex items-center justify-between">
            <div className="text-2xl font-bold">💃 Your Avatars</div>

            <div>
              <Button
                variant="outlined"
                rounded="small"
                disabled={!authenticated}
                onClick={handleCreateAvatar}
              >
                <MdAdd className="text-lg" />
              </Button>
            </div>
          </div>

          <div>
            {authenticated ? (
              data ? (
                <div className="grid grid-cols-5 gap-3">
                  {data.publications.items.map((publication) => (
                    <Link key={publication.id} href={`/avatar/${publication.id}`}>
                      <AvatarCard sizes="195px" avatar={publication as Post} />
                    </Link>
                  ))}
                </div>
              ) : (
                <div>
                  No avatars found.{" "}
                  <button
                    onClick={handleCreateAvatar}
                    className="cursor-pointer font-bold text-sky-400 decoration-2 hover:underline"
                  >
                    Click here
                  </button>{" "}
                  to create one.
                </div>
              )
            ) : (
              <div className="text-neutral-500">You need to be signed in to upload an avatar.</div>
            )}
          </div>
        </div>
      </div>
    </>
  );
}

Avatars.getLayout = getNavbarLayout;
