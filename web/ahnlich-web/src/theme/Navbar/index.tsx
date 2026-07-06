import type {ReactNode} from 'react';
import {useLocation} from '@docusaurus/router';
import Navbar from '@theme-original/Navbar';
import type NavbarType from '@theme/Navbar';
import type {WrapperProps} from '@docusaurus/types';

import HomeNavbar from '@site/src/components/HomeNavbar';

type Props = WrapperProps<typeof NavbarType>;

/**
 * Render a bespoke marketing navbar on the landing page and the default
 * Docusaurus navbar everywhere else (docs, blog, etc).
 */
export default function NavbarWrapper(props: Props): ReactNode {
  const {pathname} = useLocation();

  if (pathname === '/') {
    return <HomeNavbar />;
  }

  return <Navbar {...props} />;
}
