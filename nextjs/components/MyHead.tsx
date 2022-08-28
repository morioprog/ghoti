import type { NextPage } from 'next';
import Head from 'next/head';

interface MyHeadProps {
  title?: string;
}

const MyHead: NextPage<MyHeadProps> = ({ title }) => {
  if (title === undefined) {
    title = 'ghoti - Puyo AI';
  } else {
    title = `${title} - ghoti (Puyo AI)`;
  }

  return (
    <Head>
      <link
        rel="stylesheet"
        href="https://fonts.googleapis.com/icon?family=Material+Icons"
      ></link>
      <link rel="icon" href="/favicon.ico" />
      <title>{title}</title>
    </Head>
  );
};

export default MyHead;
