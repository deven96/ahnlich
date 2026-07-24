import type {ReactNode} from 'react';
import {useLocation} from '@docusaurus/router';
import Navbar from '@theme-original/Navbar';
import type NavbarType from '@theme/Navbar';
import type {WrapperProps} from '@docusaurus/types';

import HomeNavbar from '@site/src/components/HomeNavbar';
import DocsSubnav from '@site/src/components/DocsSubnav';

type Props = WrapperProps<typeof NavbarType>;

/**
 * Render a bespoke marketing navbar on the landing page, and the default
 * Docusaurus navbar elsewhere. On docs pages, add a Prisma-style secondary
 * tab bar (Getting Started · Vector DB · CLI · Guides) under the main navbar.
 */
export default function NavbarWrapper(props: Props): ReactNode {
  const {pathname} = useLocation();

  if (pathname === '/') {
    return <HomeNavbar />;
  }

  return (
    <>
      <Navbar {...props} />
      {pathname.startsWith('/docs') && <DocsSubnav />}
    </>
  );
}
