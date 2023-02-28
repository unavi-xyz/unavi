import { getServerSession } from "../../src/server/helpers/getServerSession";
import ProfileButton from "./ProfileButton";
import SignInButton from "./SignInButton";

export default async function ClientButtons() {
  const session = await getServerSession();

  // @ts-expect-error Server Component
  return session ? <ProfileButton session={session} /> : <SignInButton />;
}
