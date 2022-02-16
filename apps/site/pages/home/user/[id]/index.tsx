import { useFeed } from "ceramic";
import { useRouter } from "next/router";

import FeedItem from "../../../../src/components/FeedItem";
import UserLayout from "../../../../src/layouts/UserLayout";

export default function User() {
  const router = useRouter();
  const id = router.query.id as string;

  const feed = useFeed(id);

  return (
    <div>
      {feed &&
        Object.values(feed).map((streamId) => {
          return <FeedItem key={streamId} streamId={streamId} />;
        })}
    </div>
  );
}

User.Layout = UserLayout;
