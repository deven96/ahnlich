import Link from '@docusaurus/Link';
import { useColorMode } from '@docusaurus/theme-common';
import React from 'react'

interface Props {
  title: string;
  icon?: string;
  link: string;
  logoLight?: any;
  logoDark?: any;
  description?: string;
}

const CustomDocCard = ({ title, icon, link, logoLight, logoDark, description }: Props) => {
  const { colorMode } = useColorMode();

  return (
    <Link
      className="font-medium !no-underline"
      to={link}
    >
      <div className='flex flex-col gap-3 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700
                rounded-2xl p-6 shadow-md hover:shadow-xl transition transform hover:-translate-y-1 
                hover:border-primary'>
        <div className='flex items-center gap-4'>
          {icon && <p className='text-3xl'>{icon}</p>}
          {(logoDark || logoLight) && <img src={colorMode === "dark" ? logoDark : logoLight} className="w-10" />}
          <h4 className='text-xl'>{title}</h4>
        </div>
        {description && <p className='text-sm'>{description}</p>}
      </div>
    </Link>
  )
}

export default CustomDocCard;