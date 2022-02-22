import { useEffect, useState } from "react";
import { DIDDataStore } from "@glazed/did-datastore";

import { ceramicRead, loader } from "../contexts/CeramicContext";
import { useFollowing } from "./useFollowing";
import { Post } from "../models/Post/types";

const model = require("../models/Feed/model.json");

export type FeedItem = {
  post: Post;
  author: string;
  streamId: string;
};

export default function useTimeline(userId: string) {
  const [allPosts, setAllPosts] = useState<FeedItem[]>([]);

  const following = useFollowing(userId);

  useEffect(() => {
    if (!following) return;

    const store = new DIDDataStore({ ceramic: ceramicRead, model });

    const feeds = [...following, userId];

    feeds.forEach(async (did) => {
      const feed = await store.get("feed", did);
      const feedArray = Object.values(feed) as string[];

      feedArray.forEach(async (streamId) => {
        const found = allPosts.find((item) => item.streamId === streamId);
        if (found) return;

        const doc = await loader.load(streamId);
        const post = doc.content as Post;
        const author = doc.controllers[0];

        const item: FeedItem = { post, author, streamId };

        setAllPosts((prev) => {
          const newUnsorted = [...prev, item];

          const newValue = newUnsorted.sort((a, b) => {
            return b.post.timestamp - a.post.timestamp;
          });

          return newValue;
        });
      });
    });
  }, [following]);

  return allPosts;
}
