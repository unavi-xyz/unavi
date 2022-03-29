import { useAuth, useUserAvatars } from "ceramic";
import Link from "next/link";
import { useState } from "react";
import { AiOutlinePlus } from "react-icons/ai";

import { IconButton } from "../components/base";
import AvatarCard from "../components/home/avatar/AvatarCard";
import NewAvatarDialog from "../components/home/avatar/NewAvatarDialog";
import SidebarLayout from "../layouts/SidebarLayout/SidebarLayout";

const defaultAvatars = [
  "kjzl6cwe1jw147ynxh5eor6f7yqvr1fzahsow74034df5i7iq6jvlc7xh3rv01v",
  "kjzl6cwe1jw145etkya62bigdwx2jllly50h5qrysg9cxw2lzqu66cbgx6m162m",
  "kjzl6cwe1jw1495s2wbkxyf0d7a4a5k82980jms3m1utm0yvmaev8s1dhmv20qv",
];

export default function Avatars() {
  const [open, setOpen] = useState(false);

  const { viewerId, authenticated } = useAuth();
  const avatars = useUserAvatars(viewerId);

  function handleNew() {
    setOpen(true);
  }

  return (
    <>
      <NewAvatarDialog open={open} setOpen={setOpen} />

      <div className="space-y-4">
        {authenticated && (
          <>
            <div className="card flex items-center justify-between ">
              <div className="text-2xl">Your Avatars</div>

              <div className="flex items-center space-x-4 h-8">
                <IconButton onClick={handleNew}>
                  <AiOutlinePlus />
                </IconButton>
              </div>
            </div>

            <div className="card w-full top-0 left-0 px-4 pb-4 space-x-8 flex">
              {avatars?.map((id) => {
                return (
                  <Link key={id} href={`/avatar/${id}`} passHref>
                    <div className="w-72 h-96">
                      <AvatarCard id={id} />
                    </div>
                  </Link>
                );
              }) ?? (
                <p className="text-neutral-500 text-lg">
                  It looks like you don{"'"}t have any avatars.{" "}
                  <span
                    onClick={handleNew}
                    className="text-amber-500 underline hover:decoration-2 hover:cursor-pointer"
                  >
                    Click here
                  </span>{" "}
                  to upload one.
                </p>
              )}
            </div>
          </>
        )}

        <div className="card flex items-center justify-between ">
          <div className="text-2xl">Default Avatars</div>
        </div>

        <div className="card w-full top-0 left-0 px-4 pb-4 space-x-8 flex">
          {defaultAvatars.map((id) => {
            return (
              <Link key={id} href={`/avatar/${id}`} passHref>
                <div className="w-72 h-96">
                  <AvatarCard id={id} />
                </div>
              </Link>
            );
          })}
        </div>
      </div>
    </>
  );
}

Avatars.Layout = SidebarLayout;
