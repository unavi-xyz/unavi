import { useProfile } from "ceramic";
import { useRouter } from "next/router";

import { ProfilePicture } from "../../components/base";
import SidebarLayout from "../../layouts/SidebarLayout/SidebarLayout";

export default function User() {
  const router = useRouter();
  const did = router.query.did as string;

  const { profile, imageUrl } = useProfile(did);

  return (
    <div className="p-16">
      <div className="flex items-center space-x-8">
        <div className="w-32 h-32">
          <ProfilePicture src={imageUrl} />
        </div>

        <div>
          <div className="text-3xl">{profile?.name}</div>
          <div className="text-lg text-neutral-400">{did}</div>
        </div>
      </div>
    </div>
  );
}

User.Layout = SidebarLayout;
