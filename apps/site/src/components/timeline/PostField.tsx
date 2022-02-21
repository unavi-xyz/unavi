import { Dispatch, SetStateAction, useContext, useState } from "react";
import { LoadingButton } from "@mui/lab";
import { Avatar, CircularProgress, InputBase, Stack } from "@mui/material";
import Link from "next/link";
import {
  addFeedItem,
  ceramic,
  CeramicContext,
  loader,
  newPost,
  Post,
  useProfile,
} from "ceramic";

import { useIdenticon } from "../../hooks/useIdenticon";

const CHARACTER_LIMIT = 280;

interface Props {
  setNewPosts: Dispatch<SetStateAction<string[]>>;
}

export default function PostField({ setNewPosts }: Props) {
  const { userId } = useContext(CeramicContext);

  const { imageUrl } = useProfile(userId);
  const identicon = useIdenticon(userId);

  const [text, setText] = useState("");
  const [loading, setLoading] = useState(false);

  async function handlePost() {
    setLoading(true);

    const timestamp = Date.now();
    const post: Post = { text, timestamp };
    const streamId = await newPost(post, loader);

    await addFeedItem(streamId, ceramic);

    setText("");
    setLoading(false);

    setNewPosts((prev) => {
      const newValue = [streamId, ...prev];
      return newValue;
    });
  }

  return (
    <Stack spacing={1} sx={{ padding: 2 }}>
      <Stack direction="row" spacing={2}>
        <Link href={`/home/user/${userId}`} passHref>
          <Avatar
            className="clickable"
            variant="rounded"
            src={imageUrl ?? identicon}
            sx={{ width: "3rem", height: "3rem" }}
          />
        </Link>

        <InputBase
          fullWidth
          multiline
          placeholder="What's happening?"
          value={text}
          onChange={(e) => setText(e.target.value)}
          inputProps={{ maxLength: CHARACTER_LIMIT }}
        />
      </Stack>

      <Stack
        direction="row"
        alignItems="center"
        justifyContent="flex-end"
        spacing={3}
      >
        <CircularProgress
          variant="determinate"
          size="1.4em"
          color={text.length === CHARACTER_LIMIT ? "secondary" : "primary"}
          value={(text.length / CHARACTER_LIMIT) * 100}
        />

        <LoadingButton
          loading={loading}
          variant="contained"
          disabled={text.length === 0}
          onClick={handlePost}
          sx={{ width: "100px" }}
        >
          Post
        </LoadingButton>
      </Stack>
    </Stack>
  );
}
