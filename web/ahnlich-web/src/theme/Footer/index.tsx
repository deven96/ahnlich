import type {ReactNode} from 'react';
import {useLocation} from '@docusaurus/router';
import Footer from '@theme-original/Footer';
import type FooterType from '@theme/Footer';
import type {WrapperProps} from '@docusaurus/types';

import HomeFooter from '@site/src/components/HomeFooter';

type Props = WrapperProps<typeof FooterType>;

/**
 * Render a bespoke marketing footer on the landing page, no footer at all on
 * docs pages, and the default Docusaurus footer everywhere else (blog, etc).
 */
export default function FooterWrapper(props: Props): ReactNode {
  const {pathname} = useLocation();

  if (pathname === '/') {
    return <HomeFooter />;
  }

  // Docs pages: drop the footer entirely for a cleaner, app-like reading view.
  if (pathname.startsWith('/docs')) {
    return null;
  }

  return <Footer {...props} />;
}
