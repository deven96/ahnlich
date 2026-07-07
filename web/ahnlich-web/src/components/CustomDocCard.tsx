import Link from '@docusaurus/Link';
import {useColorMode} from '@docusaurus/theme-common';
import React from 'react';
import CardIcon from '@site/src/components/CardIcons';

interface Props {
  title: string;
  icon?: string;
  link: string;
  logoLight?: any;
  logoDark?: any;
  description?: string;
}

/**
 * Overview tile — shares the Quickstart chooser card styling (accent icon chip,
 * title, optional description, and a "Learn more" arrow) so every menu/overview
 * page looks consistent.
 */
const CustomDocCard = ({title, icon, link, logoLight, logoDark, description}: Props) => {
  const {colorMode} = useColorMode();
  const logo = colorMode === 'dark' ? logoDark : logoLight;

  return (
    <Link className="ahn-qs-card" to={link}>
      <span className="ahn-qs-icon" aria-hidden>
        {logoDark || logoLight ? (
          <img src={logo} alt="" style={{width: 24, height: 24, objectFit: 'contain'}} />
        ) : (
          <CardIcon name={icon} />
        )}
      </span>
      <span className="ahn-qs-title">{title}</span>
      {description && <span className="ahn-qs-desc">{description}</span>}
      <span className="ahn-qs-start">
        Learn more
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2.2}
          strokeLinecap="round" strokeLinejoin="round" aria-hidden>
          <path d="m9 6 6 6-6 6" />
        </svg>
      </span>
    </Link>
  );
};

export default CustomDocCard;
