export default function Home() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-center p-24">
      <div className="text-center">
        <h1 className="text-5xl font-bold tracking-tighter mb-4">
          Ghost Pirates
        </h1>
        <p className="text-xl text-muted-foreground mb-8">
          Welcome to your Next.js 14 application
        </p>
        <div className="flex gap-4 justify-center">
          <a
            href="#"
            className="px-8 py-2 rounded-md bg-primary text-primary-foreground font-semibold hover:bg-primary/90 transition-colors"
          >
            Get Started
          </a>
          <a
            href="#"
            className="px-8 py-2 rounded-md border border-border bg-background hover:bg-accent/5 transition-colors"
          >
            Learn More
          </a>
        </div>
      </div>
    </main>
  )
}
