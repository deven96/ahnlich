import React, {type ReactNode} from 'react';
import clsx from 'clsx';
import {ThemeClassNames, useColorMode} from '@docusaurus/theme-common';
import {useSidebarBreadcrumbs} from '@docusaurus/plugin-content-docs/client';
import {useHomePageRoute} from '@docusaurus/theme-common/internal';
import Link from '@docusaurus/Link';
import {translate} from '@docusaurus/Translate';
import HomeBreadcrumbItem from '@theme/DocBreadcrumbs/Items/Home';
import DocBreadcrumbsStructuredData from '@theme/DocBreadcrumbs/StructuredData';

import styles from './styles.module.css';

// Helper function to get the correct icon based on theme
function getThemedIcon(customProps: Record<string, unknown> | undefined, colorMode: string) {
  if (!customProps) return null;
  
  const iconType = (customProps.iconType as string) || 'emoji';
  
  // Check if themed icons are provided
  if (customProps.iconLight && customProps.iconDark) {
    return {
      icon: colorMode === 'dark' ? (customProps.iconDark as string) : (customProps.iconLight as string),
      iconType,
    };
  }
  
  // Fallback to single icon
  return {
    icon: customProps.icon as string,
    iconType,
  };
}

// TODO move to design system folder
function BreadcrumbsItemLink({
  children,
  href,
  isLast,
}: {
  children: ReactNode;
  href: string | undefined;
  isLast: boolean;
}): ReactNode {
  const className = 'breadcrumbs__link';
  if (isLast) {
    return <span className={className}>{children}</span>;
  }
  return href ? (
    <Link className={className} href={href}>
      <span>{children}</span>
    </Link>
  ) : (
    <span className={className}>{children}</span>
  );
}

// TODO move to design system folder
function BreadcrumbsItem({
  children,
  active,
}: {
  children: ReactNode;
  active?: boolean;
}): ReactNode {
  return (
    <li
      className={clsx('breadcrumbs__item', {
        'breadcrumbs__item--active': active,
      })}>
      {children}
    </li>
  );
}

export default function DocBreadcrumbs(): ReactNode {
  const breadcrumbs = useSidebarBreadcrumbs();
  const homePageRoute = useHomePageRoute();
  const {colorMode} = useColorMode();

  if (!breadcrumbs) {
    return null;
  }

  return (
    <>
      <DocBreadcrumbsStructuredData breadcrumbs={breadcrumbs} />
      <nav
        className={clsx(
          ThemeClassNames.docs.docBreadcrumbs,
          styles.breadcrumbsContainer,
        )}
        aria-label={translate({
          id: 'theme.docs.breadcrumbs.navAriaLabel',
          message: 'Breadcrumbs',
          description: 'The ARIA label for the breadcrumbs',
        })}>
        <ul className="breadcrumbs">
          {homePageRoute && <HomeBreadcrumbItem />}
          {breadcrumbs.map((item, idx) => {
            const isLast = idx === breadcrumbs.length - 1;
            const href =
              item.type === 'category' && item.linkUnlisted
                ? undefined
                : item.href;
            
            // Get icon data
            const iconData = getThemedIcon(item.customProps, colorMode);
            const icon = iconData?.icon;
            const iconType = iconData?.iconType || 'emoji';

            return (
              <BreadcrumbsItem key={idx} active={isLast}>
                <BreadcrumbsItemLink href={href} isLast={isLast}>
                  {icon && iconType === 'emoji' && (
                    <span className={styles.breadcrumbIcon} aria-hidden="true">
                      {icon}
                    </span>
                  )}
                  {icon && iconType === 'img' && (
                    <img 
                      src={icon} 
                      alt="" 
                      className={styles.breadcrumbIcon}
                      aria-hidden="true"
                    />
                  )}
                  {icon && iconType === 'svg' && (
                    <span 
                      className={styles.breadcrumbIcon}
                      dangerouslySetInnerHTML={{__html: icon}}
                      aria-hidden="true"
                    />
                  )}
                  {item.label}
                </BreadcrumbsItemLink>
              </BreadcrumbsItem>
            );
          })}
        </ul>
      </nav>
    </>
  );
}