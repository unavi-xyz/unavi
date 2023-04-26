import AuthProvider from "@/src/client/AuthProvider";
import { getUserSession } from "@/src/server/auth/getUserSession";
import { fetchProfileFromAddress } from "@/src/server/helpers/fetchProfileFromAddress";

import ProfileButton from "./ProfileButton";
import SignInButton from "./SignInButton";

export default async function AccountButton() {
  const session = await getUserSession();
  const profile = session ? await fetchProfileFromAddress(session.user.address) : null;

  return (
    <AuthProvider>
      {session ? <ProfileButton user={session.user} profile={profile} /> : <SignInButton />}
    </AuthProvider>
  );
}
