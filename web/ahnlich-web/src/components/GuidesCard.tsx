import React from "react";
import Link from "@docusaurus/Link";
import { useColorMode } from "@docusaurus/theme-common";

const GuidesCard = ({ title, description, link, logoLight, logoDark }) => {
  const { colorMode } = useColorMode();

  return (
    <Link
      className="font-medium !no-underline"
      to={link}
    >
      <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700
                      rounded-2xl p-6 shadow-md hover:shadow-xl transition transform hover:-translate-y-1 hover:border-primary flex flex-col 
                      items-start justify-between h-[22rem]"
      >
      <img src={colorMode === "dark" ? logoDark : logoLight} className="w-20" />
      <div className="">
        <h3 className="text-lg font-semibold mb-3">{title}</h3>
        <p className="mb-4 text-xs font-normal">{description}</p>
      </div>
    </div>
    </Link>
  );
}

export default GuidesCard;
