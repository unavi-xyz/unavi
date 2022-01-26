import {
  Card,
  CardActionArea,
  CardContent,
  Grid,
  Stack,
  Typography,
} from "@mui/material";
import { OBJECTS } from "3d";

import { useScene } from "../../state/useScene";

interface Props {
  name: OBJECTS;
}

export default function ObjectCard({ name }: Props) {
  const newObject = useScene((state) => state.newObject);

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
