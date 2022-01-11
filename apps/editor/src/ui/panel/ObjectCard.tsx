import {
  Card,
  CardActionArea,
  CardContent,
  Grid,
  Stack,
  Typography,
} from "@mui/material";
import { OBJ_NAMES } from "3d";

import { useStore } from "../../state";

interface Props {
  name: OBJ_NAMES;
}

export default function ObjectCard({ name }: Props) {
  const newObject = useStore((state) => state.newObject);

  function handleClick() {
    newObject(name);
  }

  return (
    <Grid item xs sx={{ minWidth: "120px" }}>
      <Card variant="outlined">
        <CardActionArea onClick={handleClick}>
          <CardContent>
            <Stack alignItems="center">
              <Typography>{name}</Typography>
            </Stack>
          </CardContent>
        </CardActionArea>
      </Card>
    </Grid>
  );
}
