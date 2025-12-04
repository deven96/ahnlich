import type {ReactNode} from 'react';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import HomepageFeatures from '@site/src/components/HomepageFeatures';

import ImageSearch from '@site/static/img/landingPage/rustimagesearch.png'

import { ActionLinks } from '../components/buttons';
import { DownloadIcon, GithubIcon, RocketIcon } from '../components/icons';

function HomepageHeader() {
  const {siteConfig} = useDocusaurusContext();
  return (
    <header className="pt-8 md:pt-16 text-center sticky top-0 bg-[url(@site/static/img/landingPage/hero.jpg)]">
      <div className="container relative flex flex-col items-center text-white z-10">
        <img src={ImageSearch} alt='An architectural diagram of Ahnlich' className='w-full lg:w-1/2 rounded-2xl' />
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
      <main className='z-10 bg-white dark:bg-[#242526]'>
        <section className='p-12 bg-slate-200 dark:bg-slate-500'>
          <p className='text-3xl text-center'>Smarter search.<br /> A vector engine that gets out of your way.</p>
          <div className='flex flex-col md:flex-row items-center justify-center my-10 gap-12'>
            <ActionLinks href="/docs/getting-started" icon={<RocketIcon />}>Get Started</ActionLinks>
            <ActionLinks href="https://github.com/deven96/ahnlich" icon={<GithubIcon />}>View Github</ActionLinks>
          </div>
        </section>

        <HomepageFeatures />

        <section className='p-12 bg-slate-200 dark:bg-slate-500'>
          <p className='text-3xl text-center'>Get releases for Mac and Linux</p>
          <div className='flex items-center justify-center my-10 gap-12'>
            <ActionLinks href="https://github.com/deven96/ahnlich/releases" icon={<DownloadIcon />}>Download now</ActionLinks>
          </div>
        </section>
      </main>
    </Layout>
  );
}
