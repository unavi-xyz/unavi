import { useRouter } from "next/router";
import { useSession } from "next-auth/react";
import { useEffect } from "react";

import { trpc } from "../../client/trpc";

export function useAnalytics() {
  const router = useRouter();
  const id = router.query.id as string;

  const { mutateAsync } = trpc.useMutation("auth.space-view");
  const { status } = useSession();

  useEffect(() => {
    if (!id || status !== "authenticated") return;

    // Send space view event
    mutateAsync({ spaceId: id });
  }, [mutateAsync, id, status]);
}
