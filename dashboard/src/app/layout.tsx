import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "Workforce OS | Operational Dashboard",
  description: "Enterprise operational intelligence and deep work analytics.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={inter.className}>
        <div className="dashboard-layout">
          <aside className="sidebar">
            <h1 className="brand">WORKFORCE OS</h1>
            <nav className="nav-links">
              <a href="#" className="nav-link active">Deep Work Analytics</a>
              <a href="#" className="nav-link">Context Switch Scatter</a>
              <a href="#" className="nav-link">Organizational Units</a>
            </nav>
            <div className="user-profile">
              <span className="user-role">MANAGER VIEW</span>
            </div>
          </aside>
          <main className="main-content">
            {children}
          </main>
        </div>
      </body>
    </html>
  );
}
