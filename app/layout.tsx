import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Grove OS",
  description: "A living operating system that knows you.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className="min-h-screen bg-[#0a0a0a] text-[#e5e5e5] antialiased">
        {children}
      </body>
    </html>
  );
}
