import { getServerSession as getSession } from "next-auth";
import { cache } from "react";

import { CustomSession } from "../../client/auth/useSession";
import { getAuthOptions } from "../../pages/api/auth/[...nextauth]";

export const getServerSession = cache(async () => {
  return (await getSession(getAuthOptions())) as CustomSession | null;
});
