import { useRouter } from "next/router";
import { useEffect } from "react";

import { trpc } from "../../client/trpc";

export function useAnalytics() {
  const router = useRouter();
  const id = router.query.id as string;

  const { mutateAsync } = trpc.public.spaceView.useMutation();

  useEffect(() => {
    if (!id) return;

    // Send space view event if the user is still here after 30 seconds
    const timeout = setTimeout(() => {
      mutateAsync({ id });
    }, 30 * 1000);

    return () => clearTimeout(timeout);
  }, [mutateAsync, id]);
}
