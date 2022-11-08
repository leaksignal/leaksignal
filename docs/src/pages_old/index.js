import React from 'react';
import Layout from '@theme/Layout';

export default function Home() {
  React.useEffect(() => {
    window.location.href = "/docs/";
  }, []);
  return (
    <Layout
      title={`LeakSignal`}
      description="">
      
    </Layout>
  );
}
