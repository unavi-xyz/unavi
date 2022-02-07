import { Dispatch, SetStateAction } from "react";
import {
  Card,
  CardActionArea,
  CardContent,
  Stack,
  Typography,
} from "@mui/material";
import NavigateNextIcon from "@mui/icons-material/NavigateNext";

interface Props {
  name: string;
  setPack: Dispatch<SetStateAction<string>>;
}

export default function PackCard({ name, setPack }: Props) {
  function handleClick() {
    setPack(name);
  }

  return (
    <Card variant="outlined">
      <CardActionArea onClick={handleClick}>
        <CardContent>
          <Stack
            direction="row"
            alignItems="center"
            justifyContent="space-between"
          >
            <Typography>{name}</Typography>
            <NavigateNextIcon />
          </Stack>
        </CardContent>
      </CardActionArea>
    </Card>
  );
}
