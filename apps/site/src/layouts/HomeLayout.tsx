import React from "react";
import HomeNavbar from "../components/HomeNavbar";

export default function HomeLayout({ children }) {
  return (
    <div>
      <HomeNavbar />
      {children}
    </div>
  );
}
