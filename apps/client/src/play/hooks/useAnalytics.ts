import { useRouter } from "next/router";
import { useEffect } from "react";

import { trpc } from "../../client/trpc";
import { hexDisplayToNumber } from "../../utils/numberToHexDisplay";

export function useAnalytics() {
  const router = useRouter();
  const id = router.query.id as string | undefined;

  const { mutateAsync: addView } = trpc.public.addView.useMutation();

  useEffect(() => {
    if (!id) return;

    const spaceId = hexDisplayToNumber(id);

    // Send space view event after 30 seconds
    const timeout = setTimeout(() => {
      addView({ spaceId });
    }, 30 * 1000);

    // Send another after 3 minutes
    const timeout2 = setTimeout(() => {
      addView({ spaceId });
    }, 3 * 60 * 1000);

    return () => {
      clearTimeout(timeout);
      clearTimeout(timeout2);
    };
  }, [addView, id]);
}
