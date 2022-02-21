import { useContext } from "react";
import { CeramicContext } from "ceramic";

import FeedItem from "./FeedItem";
import useTimeline from "ceramic/hooks/useTimeline";

export default function Timeline() {
  const { userId } = useContext(CeramicContext);

  const timeline = useTimeline(userId);

  return (
    <div>
      {timeline.map(({ streamId }) => {
        return <FeedItem key={streamId} streamId={streamId} />;
      })}
    </div>
  );
}
