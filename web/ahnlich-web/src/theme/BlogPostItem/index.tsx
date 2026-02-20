import React from 'react';
import BlogPostItem from '@theme-original/BlogPostItem';
import type BlogPostItemType from '@theme/BlogPostItem';
import type { WrapperProps } from '@docusaurus/types';
import { useBlogPost } from '@docusaurus/plugin-content-blog/client';
import GiscusComments from '@site/src/components/GiscusComments';

type Props = WrapperProps<typeof BlogPostItemType>;

export default function BlogPostItemWrapper(props: Props): JSX.Element {
  const { metadata, isBlogPostPage } = useBlogPost();
  const { frontMatter } = metadata;

  return (
    <>
      <BlogPostItem {...props} />
      {/* Only show comments on the blog post page, not in list views */}
      {isBlogPostPage && <GiscusComments />}
    </>
  );
}
