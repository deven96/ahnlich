import type {ReactNode} from 'react';
import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

import IndexSearchImg from '@site/static/img/landingPage/imagesearch.png'
import SearchPyImg from '@site/static/img/landingPage/Searchphrase-python.png'
import StoreItemImg from '@site/static/img/landingPage/metadasearchrs.png'
import UtilizeAIImg from '@site/static/img/landingPage/utilize-ai-models.png'
import SimilarityScoreImg from '@site/static/img/landingPage/similarityscore.png'
import QueryByPropsImg from '@site/static/img/landingPage/query-by-properties.png'

type FeatureItem = {
  title: string;
  img: any[];
  description: ReactNode;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Index and search items',
    img: [IndexSearchImg, SearchPyImg],
    description: "Quickly index data and retrieve the most similar items using vector-based semantic search.",
  },
  {
    title: 'Store item properties',
    img: [StoreItemImg],
    description: "Attach custom metadata to each item so searches can use both vectors and structured properties.",
  },
  {
    title: 'Utilize custom AI models',
    img: [UtilizeAIImg],
    description: "Plug in your own embedding or AI models to control how items are encoded and stored.",
  },
  {
    title: 'Configure similarity score',
    img: [SimilarityScoreImg],
    description: "Customize the similarity metric (e.g., cosine, Euclidean) to match your retrieval behavior.",
  },
  {
    title: 'Query by properties',
    img:[ QueryByPropsImg],
    description: "Filter search results using metadata constraints for more targeted and precise retrieval.",
  },
  // {
  //   title: 'Local Document search',
  //   img: UtilizeAIImg,
  //   description: "Run fast, privacy-preserving semantic search on your own machine without sending data to the cloud.",
  // },
];

const colClasses = {
  1: "grid-cols-1",
  2: "grid-cols-2",
  3: "grid-cols-3",
  4: "grid-cols-4",
  5: "grid-cols-5",
  6: "grid-cols-6",
};


function Feature({title, img, description}: FeatureItem) {
  return (
    <div className='w-full'>
      <div className="flex flex-col items-center gap-5 text-center my-7">
        <Heading as="h3" className='font-semibold text-4xl'>{title}</Heading>
        <p className='text-lg'>{description}</p>
      </div>

      <div className={`flex flex-col lg:flex-row justify-center items-center gap-5`}>
        {img.map((i, idx) => (
          <img
            src={i}
            alt={`Example for ${title}-${idx}`}
            className={`object-cover rounded-lg ${ img.length > 1 ? "w-full lg:w-1/2" : "w-full lg:w-2/3"}`}
            // style={}
          />
        ))}
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="flex flex-col gap-10 items-center">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
