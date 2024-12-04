/* eslint-disable @next/next/no-img-element */
import { BookOpenText, BrainCircuit, Github, Waypoints } from "lucide-react";
import CodeSnippet from "./components/CodeSnippet";
import Card from "./components/Card";
import { Title } from "./components/Title";

export default function Home() {
  const wget = `wget https://github.com/deven96/ahnlich/releases/download/bin%2Fdb%2F0.0.0/aarch64-darwin-ahnlich-db.tar.gz`;

  const dockerCompose = `
    services:
      ahnlich_db: 
        image: ghcr.io/deven96/ahnlich-db:latest
        command: >
          "ahnlich-db run --host 0.0.0.0"
        ports:
          - "1369:1369"

      ahnlich_ai:
        image: ghcr.io/deven96/ahnlich-ai:latest
        command: >
          "ahnlich-ai run --db-host ahnlich_db --host 0.0.0.0 \
          --supported-models all-minilm-l6-v2,resnet-50"
        ports:
          - "1370:1370"
  `

  return (
    <div className="font-[family-name:var(--font-lato)] text-lg leading-loose">
      <main className="flex flex-col gap-8 items-center sm:items-start w-full">
        <section className="hero relative p-10 w-full h-[70vh] grid bg-[url(https://res.cloudinary.com/drfw1bzcw/image/upload/v1733262252/Ahnlich/hero_f4xrul.webp)]">
          <div className="m-auto text-center text-white z-[15]">
            <h1 className="text-7xl font-semibold my-5">Ahnlich</h1>
            <h2 className="text-xl md:text-3xl">
              A project by developers bringing vector database <br className="hidden md:block" /> and artificial intelligence powered semantic search abilities closer to you
            </h2>
          </div>
          <div className="absolute bg-black/60 inset-0 h-full w-full z-[10]" />
        </section>

        <div className="w-full xl:w-[90%] 2xl:w-[80%] m-auto p-5 md:p-10">
          <section className="flex flex-col lg:items-center w-full max-w-screen">
            <Title>How To Use</Title>
            <p>Ahnlich comprises of multiple tools for usage and development such as</p>
            <ul className="my-3">
              <li>
                <span className="font-semibold">ahnlich-db:</span> In-memory vector key value store for storing embeddings/vectors with corresponding metadata(key-value maps).
              </li>
              <li>
                <span className="font-semibold">ahnlich-ai:</span> AI proxy to communicate with <code className="native-code">ahnlich-db</code>, receiving raw input, transforming into embeddings, and storing within the DB
              </li>
            </ul>
            <p>
              <code className="native-code">ahnlich-db</code>, <code className="native-code">ahnlich-ai</code> and <code className="native-code">ahnlich-cli</code>
              are packaged and released as <a href="https://github.com/deven96/ahnlich/releases" className="mr-1">binaries</a>
              for multiple platforms alongside <a href="https://github.com/deven96?tab=packages&repo_name=ahnlich">docker images</a>. <br />
              The DB can be used without the AI proxy for more fine grained control of the generated vector embeddings as all clients support both.
            </p>
          </section>

          <section className="mt-3 w-full">
            <Title>Installation</Title>
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-20 w-full items-start justify-around my-10 text-wrap">
              <Card title="Using Docker">
                <p className="text-xl font-medium">Ahnlich AI</p>
                <CodeSnippet code={"docker pull ghcr.io/deven96/ahnlich-ai:latest"} language="bash" />
                <p className="text-xl font-medium">Ahnlich DB</p>
                <CodeSnippet code={"docker pull ghcr.io/deven96/ahnlich-db:latest"} language="bash" />
              </Card>

              <Card title="Example Docker Compose">
                <CodeSnippet code={dockerCompose} language="yaml" />
              </Card>

              <Card title="Download Binaries" style="col-span-full">
                <p className="text-xl font-medium">wget</p>
                <CodeSnippet code={wget} language="bash" />
                <p className="text-xl font-medium">Extract the File</p>
                <CodeSnippet code={"tar -xvzf aarch64-darwin-ahnlich-db.tar.gz"} language="bash" />
                <p className="text-xl font-medium">Run the binary</p>
                <CodeSnippet code={"./ahnlich-db "} language="bash" />
              </Card>
            </div>
          </section>

          <section className="flex flex-col items-center w-full">
            <Title>Libraries</Title>
            <div className="flex flex-col gap-5 my-3">
              <div className="flex items-center">
                <img src="assets/rust-logo.png" alt="Rust programming language logo" className="w-8 mr-3" />
                <p className="text-xl"><span className="font-semibold">ahnlich-client-rs: </span>Rust client for ahnlich-db and ahnlich-ai with support for connection pooling</p>
              </div>

              <CodeSnippet code="cargo add ahnlich_client_rs" language="bash" />

              <div className="grid lg:grid-cols-2 gap-10 my-3 w-full">
                <img className="w-full" src="https://res.cloudinary.com/drfw1bzcw/image/upload/v1733236237/Ahnlich/query-image_nfrmfc.gif" alt="" />
                <img className="w-full" src="https://res.cloudinary.com/drfw1bzcw/image/upload/v1733236235/Ahnlich/index-image_pyc3t2.gif" alt="" />
              </div>

              <div className="flex items-center gap-5 text-base">
                <a
                  className="flex items-center gap-2 text-white bg-primary w-fit px-2 py-1 rounded"
                  href='https://github.com/deven96/ahnlich/tree/main/examples/rust/image-search'
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  See Example
                </a>
                <a
                  className="flex items-center gap-2 text-white bg-primary w-fit px-2 py-1 rounded"
                  href="https://github.com/deven96/ahnlich/tree/main/ahnlich/client"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  See Github
                </a>
                <a
                  className="flex items-center gap-2 text-white bg-primary w-fit px-2 py-1 rounded"
                  href="https://crates.io/crates/ahnlich_client_rs"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  Docs
                </a>
              </div>
            </div>

            <hr className="my-5 border-grey-3 w-full" />

            <div className="flex flex-col gap-5 my-3">
              <div className="flex items-center">
                <img src="assets/python-logo.png" alt="Pyhton programming language logo" className="w-8 mr-3" />
                <p className="text-xl"><span className="font-semibold">ahnlich-client-py: </span>Python client for ahnlich-db and ahnlich-ai with support for connection pooling.</p>
              </div>
              <p className="font-semibold text-xl">Using Poetry</p>
              <CodeSnippet code="poetry add ahnlich-client-py" language="bash" />
              <p className="font-semibold text-xl">Using Pip</p>
              <CodeSnippet code="pip3 install ahnlich-client-py" language="bash" />

              <div className="grid lg:grid-cols-2 gap-10 my-auto lg:my-3 w-full">
                <img className="w-full" src="https://res.cloudinary.com/drfw1bzcw/image/upload/v1733236276/Ahnlich/insertbook_pplxxk.gif" alt="" />
                <img className="w-full" src="https://res.cloudinary.com/drfw1bzcw/image/upload/v1733236276/Ahnlich/searchphrase_k8svqo.gif" alt="" />
              </div>

              <div className="flex items-center gap-5 text-base">
                <a
                  className="flex items-center gap-2 text-white bg-primary w-fit px-2 py-1 rounded"
                  href='https://github.com/deven96/ahnlich/tree/main/examples/python/book-search'
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  See Example
                </a>
                <a
                  className="flex items-center gap-2 text-white bg-primary w-fit px-2 py-1 rounded"
                  href="https://github.com/deven96/ahnlich/tree/main/sdk/ahnlich-client-py"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  See Github
                </a>
                <a
                  className="flex items-center gap-2 text-white bg-primary w-fit px-2 py-1 rounded"
                  href="https://pypi.org/project/ahnlich-client-py/ "
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  Docs
                </a>
              </div>
            </div>
          </section>
        </div>
      </main>
      <footer className="row-start-3 flex gap-6 flex-wrap items-center justify-center p-5">
        <a
          className="flex items-center gap-2 hover:underline hover:underline-offset-4 text-primary"
          href="https://github.com/deven96/ahnlich"
          target="_blank"
          rel="noopener noreferrer"
        >
          <Github />
          Github
        </a>
        <a
          className="flex items-center gap-2 hover:underline hover:underline-offset-4 text-primary"
          href="https://github.com/deven96/ahnlich"
          target="_blank"
          rel="noopener noreferrer"
        >
          <BookOpenText />
          Docs
        </a>
        <a
          className="flex items-center gap-2 hover:underline hover:underline-offset-4 text-primary"
          href="https://github.com/deven96/ahnlich"
          target="_blank"
          rel="noopener noreferrer"
        >
          <Waypoints />
          Ahnlich
        </a>
        <a
          className="flex items-center gap-2 hover:underline hover:underline-offset-4 text-primary"
          href="https://github.com/deven96/ahnlich"
          target="_blank"
          rel="noopener noreferrer"
        >
          <BrainCircuit />
          AI proxy
        </a>
      </footer>
    </div>
  );
}
