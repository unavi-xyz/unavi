import { Dispatch, SetStateAction } from "react";
import { Checkbox, Grid, Typography } from "@mui/material";
import { ASSETS, ASSET_NAMES, PARAM_NAMES } from "3d";

interface Props {
  title: string;
  type: PARAM_NAMES;
  params: typeof ASSETS[ASSET_NAMES]["params"];
  setParams: Dispatch<SetStateAction<typeof ASSETS[ASSET_NAMES]["params"]>>;
}

export default function CheckField({ title, type, params, setParams }: Props) {
  function handleChange(e) {
    setParams((prev) => {
      const newParams = { ...prev };
      newParams[type] = Boolean(e.target.checked);
      return newParams;
    });
  }

  return (
    <Grid container direction="row" alignItems="center" columnSpacing={2}>
      <Grid item xs={3}>
        <Typography>{title}</Typography>
      </Grid>
      <Grid item xs>
        <Checkbox checked={params[type]} onChange={handleChange} />
      </Grid>
    </Grid>
  );
}
