import { createTheme } from "@mui/material/styles";

const primary = "#ffffff";
const secondary = "#F85F73";

const theme = createTheme({
  palette: {
    mode: "dark",
    primary: {
      main: primary,
    },
    secondary: {
      main: secondary,
    },
  },
});

export default theme;
