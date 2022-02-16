import { useContext, useState } from "react";
import {
  Avatar,
  Divider,
  IconButton,
  Menu,
  MenuItem,
  Skeleton,
  Stack,
  Typography,
} from "@mui/material";
import MoreHorizIcon from "@mui/icons-material/MoreHoriz";
import Link from "next/link";
import {
  ceramic,
  CeramicContext,
  removeStatus,
  useProfile,
  useStatus,
} from "ceramic";

interface Props {
  streamId: string;
}

export default function FeedItem({ streamId }: Props) {
  const { authenticated, userId } = useContext(CeramicContext);

  const { status, controller } = useStatus(streamId);
  const { profile, imageUrl } = useProfile(controller);

  const [deleted, setDeleted] = useState(false);
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
  const open = Boolean(anchorEl);

  async function handleDelete() {
    setAnchorEl(null);
    await removeStatus(streamId, userId, ceramic);
    setDeleted(true);
  }

  if (deleted) return null;

  return (
    <div>
      <Stack direction="row" spacing={2} sx={{ padding: 2 }}>
        <Link href={`/home/user/${controller}`} passHref>
          <Avatar
            className="clickable"
            variant="rounded"
            src={imageUrl}
            sx={{ width: "3rem", height: "3rem" }}
          />
        </Link>

        <Stack spacing={0} sx={{ width: "100%" }}>
          {profile ? (
            <Link href={`/home/user/${controller}`} passHref>
              <Typography className="link">{profile?.name}</Typography>
            </Link>
          ) : (
            <Skeleton width="100px" />
          )}

          {status ? <Typography>{status.text}</Typography> : <Skeleton />}
        </Stack>

        {authenticated && userId === controller && (
          <div>
            <IconButton
              size="small"
              onClick={(e) => setAnchorEl(e.currentTarget)}
            >
              <MoreHorizIcon />
            </IconButton>

            <Menu
              anchorEl={anchorEl}
              open={open}
              onClose={() => setAnchorEl(null)}
            >
              <MenuItem onClick={handleDelete}>Delete</MenuItem>
            </Menu>
          </div>
        )}
      </Stack>

      <Divider />
    </div>
  );
}
