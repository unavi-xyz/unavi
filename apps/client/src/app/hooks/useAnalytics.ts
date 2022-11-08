import { useRouter } from "next/router";
import { useEffect } from "react";

import { trpc } from "../../client/trpc";

export function useAnalytics() {
  const router = useRouter();
  const id = router.query.id as string;

  const { mutateAsync: addView } = trpc.public.addView.useMutation();

  useEffect(() => {
    if (!id) return;

    // Send space view event after 30 seconds
    const timeout = setTimeout(() => {
      addView({ lensId: id });
    }, 30 * 1000);

    // Send another after 3 minutes
    const timeout2 = setTimeout(() => {
      addView({ lensId: id });
    }, 3 * 60 * 1000);

    return () => {
      clearTimeout(timeout);
      clearTimeout(timeout2);
    };
  }, [addView, id]);
}
