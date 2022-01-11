import Head from "next/head";
import { LoginPage } from "ui";

export default function Login() {
  return (
    <div>
      <Head>
        <title>The Wired - Editor</title>
      </Head>
      <LoginPage />;
    </div>
  );
}
