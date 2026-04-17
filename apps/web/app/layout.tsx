import './globals.css';

export const metadata = {
  title: 'Ghost Cockpit',
  description: 'A monochrome research dashboard for the Ghost orchestration stack.'
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
