import Link from "@docusaurus/Link"
import { ReactNode } from "react"

interface ActionLinksProps {
  href: string;
  children: ReactNode;
  icon?: ReactNode;
  /** 'primary' = solid accent pill, 'ghost' = hairline outline pill */
  variant?: "primary" | "ghost";
}

const base =
  "inline-flex items-center gap-2 rounded-full px-6 py-2.5 text-base font-semibold no-underline transition-all duration-150 hover:no-underline";

const variants = {
  primary:
    "border border-solid border-transparent bg-primary text-white hover:-translate-y-px hover:bg-primary/90 hover:text-white",
  ghost:
    "border-[1.5px] border-solid border-[#cdddE4] bg-transparent text-primary hover:border-primary hover:bg-[#e4f4f8] hover:text-primary dark:border-white/20 dark:text-white dark:hover:border-secondary/60 dark:hover:bg-white/5 dark:hover:text-white",
} as const;

export const ActionLinks = ({
  href,
  icon,
  children,
  variant = "primary",
}: ActionLinksProps) => {
  return (
    <Link href={href} className={`${base} ${variants[variant]}`}>
      {icon}
      {children}
    </Link>
  )
}
