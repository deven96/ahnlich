import type {ReactNode} from 'react';
import Content from '@theme-original/DocSidebar/Desktop/Content';
import type ContentType from '@theme/DocSidebar/Desktop/Content';
import type {WrapperProps} from '@docusaurus/types';
import GithubRepoCard from '@site/src/components/GithubRepoCard';

type Props = WrapperProps<typeof ContentType>;

/** Show the GitHub repo card at the very top of the docs sidebar, above the menu. */
export default function ContentWrapper(props: Props): ReactNode {
  return (
    <>
      <GithubRepoCard />
      <Content {...props} />
    </>
  );
}
