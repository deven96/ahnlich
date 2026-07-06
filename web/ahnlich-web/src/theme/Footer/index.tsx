import type {ReactNode} from 'react';
import {useLocation} from '@docusaurus/router';
import Footer from '@theme-original/Footer';
import type FooterType from '@theme/Footer';
import type {WrapperProps} from '@docusaurus/types';

import HomeFooter from '@site/src/components/HomeFooter';

type Props = WrapperProps<typeof FooterType>;

/**
 * Render a bespoke marketing footer on the landing page and the default
 * Docusaurus footer everywhere else (docs, blog, etc).
 */
export default function FooterWrapper(props: Props): ReactNode {
  const {pathname} = useLocation();

  if (pathname === '/') {
    return <HomeFooter />;
  }

  return <Footer {...props} />;
}
