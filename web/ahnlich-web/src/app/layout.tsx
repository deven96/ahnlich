/* eslint-disable @next/next/no-img-element */
import type { Metadata } from "next";
import localFont from "next/font/local";
import { Black_Ops_One } from 'next/font/google'
import "./globals.css";

const geistSans = localFont({
  src: "../fonts/GeistVF.woff",
  variable: "--font-geist-sans",
  weight: "100 900",
});
const geistMono = localFont({
  src: "../fonts/GeistMonoVF.woff",
  variable: "--font-geist-mono",
  weight: "100 900",
});
 
const blackOpsOne = Black_Ops_One({
  weight: '400',
  subsets: ['latin'],
  variable: '--font-black-ops-one'
})

export const metadata: Metadata = {
  title: "Ahnlich Web",
  description: "A project by developers bringing vector database and artificial intelligence powered semantic search abilities closer to you ",
  openGraph: {
    images: ["assets/logo.jpg"],
  },
  twitter: {
    images: ["assets/logo.jpg"],
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
        className={`${geistSans.variable} ${geistMono.variable} ${blackOpsOne.variable} antialiased`}
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
