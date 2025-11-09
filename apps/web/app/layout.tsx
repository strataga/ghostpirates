import type { Metadata, Viewport } from "next"
import { Inter } from "next/font/google"
import "./globals.css"

const inter = Inter({ subsets: ["latin"] })

export const metadata: Metadata = {
  title: "Ghost Pirates",
  description: "Ghost Pirates Web Application",
  keywords: ["ghost", "pirates"],
  authors: [{ name: "Ghost Pirates Team" }],
  creator: "Ghost Pirates",
  openGraph: {
    type: "website",
    locale: "en_US",
    url: "https://ghostpirates.com",
    title: "Ghost Pirates",
    description: "Ghost Pirates Web Application",
    siteName: "Ghost Pirates",
  },
  twitter: {
    card: "summary_large_image",
    title: "Ghost Pirates",
    description: "Ghost Pirates Web Application",
  },
}

export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
  maximumScale: 5,
  userScalable: true,
  colorScheme: "light dark",
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body className={inter.className}>
        {children}
      </body>
    </html>
  )
}
