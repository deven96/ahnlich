/* eslint-disable @next/next/no-img-element */
import { Github } from "lucide-react";

export default function Home() {
  return (
    <div className="grid items-center justify-items-center min-h-screen font-[family-name:var(--font-geist-sans)]">
      <main className="flex flex-col gap-8 items-center sm:items-start w-full">
        <section className="hero relative p-10 w-full h-[70vh] grid bg-[url('/assets/hero.webp')]">
          <div className="m-auto text-center text-white z-[15]">
            <h1 className="text-7xl font-semibold my-5">Ahnlich</h1>
            <h2 className="text-3xl">
              A project by developers bringing vector database <br/> and artificial intelligence powered semantic search abilities closer to you
            </h2>
          </div>
          <div className="absolute bg-black/60 inset-0 h-full w-full z-[10]" />
        </section>

        <section className="p-10">
          <h3 className="text-primary font-medium text-3xl font-[family-name:var(--font-black-ops-one)]">How to Use</h3>
          <p>
            <code>ahnlich-db</code>, ahnlich-ai and ahnlich-cli are packaged and released as binaries for multiple platforms alongside docker images
          </p>
          <div className="grid sm:grid-cols-2 gap-20 items-center flex-wrap justify-around my-10">
            <div className="flex flex-col items-center gap-10 cursor-pointer">
              <p className="text-xl font-medium">Install binary</p>
              {/* <img src="assets/cargo-install.png" alt="Pyhton installing snippet" /> */}
              <code>
                wget https://github.com/deven96/ahnlich/releases/download/bin%2Fdb%2F0.0.0/aarch64-darwin-ahnlich-db.tar.gz  
              </code>
            </div>
            <div className="flex flex-col items-center gap-10 cursor-pointer">
              <p className="text-xl font-medium">Docker image</p>
              <img src="assets/pip-install.png" alt="Cargo installing snippet" className="" />
            </div>
          </div>
        </section>

        <section className="p-10">
          <h3 className="text-primary font-medium text-3xl font-[family-name:var(--font-black-ops-one)]">Libraries</h3>
          <div className="grid grid-cols-2 gap-20 items-center flex-wrap justify-around my-10">
            <a
              href='https://github.com/deven96/ahnlich/tree/main/examples/python/book-search'
              className="flex flex-col items-center gap-10 cursor-pointer"
              target="_blank"
              rel="noopener noreferrer"
            >
              <img src="assets/python-logo.png" alt="Pyhton programming language logo" className="w-20" />
            </a>
            <a
              href='https://github.com/deven96/ahnlich/tree/main/examples/rust/image-search'
              className="flex flex-col items-center gap-10 cursor-pointer"
              target="_blank"
              rel="noopener noreferrer"
            >
              <img src="assets/rust-logo.png" alt="Rust programming language logo" className="w-20" />
            </a>
          </div>
        </section>
      </main>
      <footer className="row-start-3 flex gap-6 flex-wrap items-center justify-center py-5">
        <a
          className="flex items-center gap-2 hover:underline hover:underline-offset-4 text-primary"
          href="https://github.com/deven96/ahnlich"
          target="_blank"
          rel="noopener noreferrer"
        >
          <Github />
          Github
        </a>
      </footer>
    </div>
  );
}
