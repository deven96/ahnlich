import type {ReactNode} from 'react';
import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  Svg: React.ComponentType<React.ComponentProps<'svg'>>;
  description: ReactNode;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Ahnlich DB',
    Svg: require('@site/static/img/landing-2.svg').default,
    description: (
      <>
        In-memory vector key value store for storing embeddings/vectors with corresponding metadata(key-value maps). 
        It&apos;s a powerful system which enables developers/engineers to store and search similar vectors semantically.
      </>
    ),
  },
  {
    title: 'Ahnlich AI',
    Svg: require('@site/static/img/landing-1.svg').default,
    description: (
      <>
        AI proxy to communicate with DB. Eeceiving raw input, transforming into embeddings, and storing within 
        the DB. It extends the capabilities by allowing developers/engineers to issue queries to the same store using 
        raw input such as images/text. 
      </>
    ),
  },
];

function Feature({title, Svg, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="flex justify-center">
        <Svg className={styles.featureSvg} role="img" />
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h3" className='font-bold text-xl'>{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="flex flex-wrap justify-evenly">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
