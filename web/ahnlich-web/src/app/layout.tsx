/* eslint-disable @next/next/no-img-element */
import type { Metadata } from "next";
import { Black_Ops_One, Lato } from 'next/font/google'
import "./globals.css";

 
const blackOpsOne = Black_Ops_One({
  weight: '400',
  subsets: ['latin'],
  variable: '--font-black-ops-one'
})

const lato = Lato({
  weight: '400',
  subsets: ['latin'],
  variable: '--font-lato'
})

export const metadata: Metadata = {
  title: "Ahnlich Web",
  description: "A project by developers bringing vector database and artificial intelligence powered semantic search abilities closer to you ",
  openGraph: {
    images: ["https://res.cloudinary.com/drfw1bzcw/image/upload/v1733266969/Ahnlich/logo_zfs3wk.webp"],
  },
  twitter: {
    images: ["https://res.cloudinary.com/drfw1bzcw/image/upload/v1733266969/Ahnlich/logo_zfs3wk.webp"],
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <head>
        <link rel="icon" type="image" href="assets/logo.jpg" />
      </head>
      <body
        className={`${lato.variable} ${blackOpsOne.variable}`}
      >
        <header className="shadow-lg flex items-center justify-between py-4 px-8">
          <div className={`flex items-center gap-4 text-primary ${blackOpsOne.className}`}>
            <img className="w-10" src="assets/logo.jpg" alt="A logo of ahnlich showing connecting node" />
            AHNLICH
          </div>
          {/* <nav className="navbar w-full">
            <ul className="nav-items">
              <li className="nav-item"><a href="#" className="nav-link">HOME</a></li>
              <li className="nav-item"><a href="#" className="nav-link">OFFER</a></li>
              <li className="nav-item"><a href="#" className="nav-link">SHOP</a></li>
              <li className="nav-item"><a href="#" className="nav-link">CONTACT</a></li>
            </ul>
          </nav>
          <div className="menu-toggle">
            <i className="bx bx-menu"></i>
            <i className="bx bx-x"></i>
          </div> */}
        </header>
        {children}
      </body>
    </html>
  );
}
