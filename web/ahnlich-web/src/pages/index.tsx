import type {ReactNode} from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import HomepageFeatures from '@site/src/components/HomepageFeatures';
import Heading from '@theme/Heading';

import styles from './index.module.css';

function HomepageHeader() {
  const {siteConfig} = useDocusaurusContext();
  return (
    <header className={clsx('relative hero hero--primary bg-[url(https://res.cloudinary.com/drfw1bzcw/image/upload/v1733262252/Ahnlich/hero_f4xrul.webp)]', styles.heroBanner)}>
      <div className="absolute inset-0 bg-black opacity-80"></div>
      <div className="container text-white z-10">
        <Heading as="h1" className="hero__title text-7xl">
          {siteConfig.title}
        </Heading>
        <p className="hero__subtitle text-2xl w-full md:w-1/2 m-auto my-3">{siteConfig.tagline}</p>
      </div>
    </header>
  );
}

export default function Home(): ReactNode {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout
      title={`${siteConfig.title}`}
      description={`${siteConfig.tagline}`}>
      <HomepageHeader />
      <main>
        <HomepageFeatures />
      </main>
    </Layout>
  );
}
