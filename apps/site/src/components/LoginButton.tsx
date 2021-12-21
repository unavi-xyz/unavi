import { useContext } from "react";
import { Button } from "@mui/material";
import Link from "next/link";

import { MatrixContext } from "../matrix/MatrixProvider";

export default function LoginButton() {
  const { loggedIn, logout } = useContext(MatrixContext);

  return (
    <div>
      {loggedIn ? (
        <Button variant="outlined" onClick={logout}>
          Logout
        </Button>
      ) : (
        <Link href="/home/login" passHref>
          <Button variant="contained">Login</Button>
        </Link>
      )}
    </div>
  );
}
