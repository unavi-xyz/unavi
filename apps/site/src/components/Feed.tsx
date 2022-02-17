import {
  MutableRefObject,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";
import { CeramicContext, useFeed, useFollowing, useStatus } from "ceramic";
import FeedItem from "./FeedItem";

export default function Feed() {
  const { userId } = useContext(CeramicContext);

  const following = useFollowing(userId);

  const feedRef = useRef(new Set<string>());
  const statusRef = useRef({});

  const [unsorted, setUnsorted] = useState([]);
  const [feed, setFeed] = useState([]);

  useEffect(() => {
    const interval = setInterval(() => {
      setUnsorted(Array.from(feedRef.current));

      const sorted = Object.keys(statusRef.current).sort((a, b) => {
        return statusRef.current[b] - statusRef.current[a];
      });

      setFeed(sorted);
    }, 500);

    return () => {
      clearInterval(interval);
    };
  }, []);

  return (
    <div>
      <FeedLoader id={userId} feedRef={feedRef} />

      {following?.map((user) => {
        return <FeedLoader key={user} id={user} feedRef={feedRef} />;
      })}

      {unsorted.map((id) => {
        return <StatusLoader key={id} id={id} statusRef={statusRef} />;
      })}

      {feed.map((streamId) => {
        return <FeedItem key={streamId} streamId={streamId} />;
      })}
    </div>
  );
}

function StatusLoader({
  id,
  statusRef,
}: {
  id: string;
  statusRef: MutableRefObject<{}>;
}) {
  const { status } = useStatus(id);

  useEffect(() => {
    if (!status?.time) return;
    statusRef.current[id] = status.time;
  }, [id, status, statusRef]);

  return null;
}

function FeedLoader({
  id,
  feedRef,
}: {
  id: string;
  feedRef: MutableRefObject<Set<string>>;
}) {
  const feed = useFeed(id);

  useEffect(() => {
    if (!feed) return;
    Object.values(feed).forEach((item) => {
      feedRef.current.add(item);
    });
  }, [feed, feedRef]);

  return null;
}
