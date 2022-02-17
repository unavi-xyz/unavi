import { useFollowing, useProfile } from "ceramic";
import { useRouter } from "next/router";

import HomeNavbar from "../../../../src/components/HomeNavbar";
import UserItem from "../../../../src/components/UserItem";
import HomeLayout from "../../../../src/layouts/HomeLayout";

export default function Following() {
  const router = useRouter();
  const id = router.query.id as string;

  const { profile } = useProfile(id);
  const following = useFollowing(id);

  return (
    <div>
      <HomeNavbar text={profile?.name} back />

      {following &&
        Object.values(following).map((id) => {
          return <UserItem key={id} id={id} />;
        })}
    </div>
  );
}

Following.Layout = HomeLayout;
