import AuthProvider from "@/src/client/AuthProvider";
import { getUserSession } from "@/src/server/auth/getUserSession";
import { prisma } from "@/src/server/prisma";

import ProfileButton from "./ProfileButton";
import SignInButton from "./SignInButton";

export default async function AccountButton() {
  const session = await getUserSession();

  const profile = session
    ? await prisma.profile.findUnique({ where: { userId: session.user.userId } })
    : null;

  return (
    <AuthProvider>
      {session ? <ProfileButton user={session.user} image={profile?.image} /> : <SignInButton />}
    </AuthProvider>
  );
}
