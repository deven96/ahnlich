import Link from "@docusaurus/Link"
import { FunctionComponent, ReactNode } from "react"

interface ActionLinksProps {
  href: string;
  children: ReactNode;
  icon?: ReactNode;
}

export const ActionLinks = ({ href, icon, children }: ActionLinksProps) => {
  return (
    <Link href={href} className="bg-primary hover:bg-primary/75 px-8 py-4 text-white hover:text-white text-lg rounded">
      <div className='flex items-center gap-1'>
        {icon}
        {children}
      </div>
    </Link>
  )
}