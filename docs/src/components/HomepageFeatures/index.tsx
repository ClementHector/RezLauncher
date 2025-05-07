import type {ReactNode} from 'react';
import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  description: ReactNode;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Streamline Rez Workflows',
    description: (
      <>
        RezLauncher simplifies managing and launching Rez environments,
        making your development process more efficient.
      </>
    ),
  },
  {
    title: 'MongoDB Integration',
    description: (
      <>
        Leverage MongoDB to store and manage your Rez package configurations,
        stages, and toolsets with ease.
      </>
    ),
  },
  {
    title: 'User-Friendly Interface',
    description: (
      <>
        Built with a modern UI, RezLauncher provides an intuitive experience
        for both configuring and launching Rez environments.
      </>
    ),
  },
];

function Feature({title, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props) => (
            <Feature key={props.title} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
