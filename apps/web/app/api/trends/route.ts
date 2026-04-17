import { NextResponse } from "next/server";

export const dynamic = "force-dynamic";
export const revalidate = 0;
export const fetchCache = "force-no-store";

const REDDIT_URL =
  process.env.REDDIT_TRENDS_URL ?? "https://api.reddit.com/r/popular?limit=10&t=day&raw_json=1";

const FALLBACK_ITEMS = [
  {
    title: "Reddit trends temporarily unavailable",
    subreddit: "popular",
    score: 0,
    url: "https://www.reddit.com/r/popular/"
  },
  {
    title: "Fallback signal stream active",
    subreddit: "ghost",
    score: 0,
    url: "https://www.reddit.com/"
  },
  {
    title: "Live fetch blocked upstream",
    subreddit: "ops",
    score: 0,
    url: "https://www.reddit.com/"
  }
];

function mapItems(payload: any) {
  return Array.isArray(payload?.data?.children)
    ? payload.data.children.map((entry: any) => ({
        title: entry?.data?.title ?? "",
        subreddit: entry?.data?.subreddit ?? "",
        score: entry?.data?.score ?? 0,
        url: entry?.data?.url ?? ""
      }))
    : [];
}

export async function GET() {
  try {
    const response = await fetch(REDDIT_URL, {
      headers: {
        Accept: "application/json",
        "Accept-Language": "en-US,en;q=0.9",
        "User-Agent": "Mozilla/5.0 Ghost OSINT Harness/1.0 (+https://github.com/kartikkabadi/Ghost)"
      },
      cache: "no-store",
      redirect: "follow"
    });

    if (!response.ok) {
      return NextResponse.json(
        {
          ok: true,
          source: "reddit_fallback",
          live: false,
          upstreamStatus: response.status,
          items: FALLBACK_ITEMS
        },
        { status: 200 }
      );
    }

    const payload = await response.json();
    const items = mapItems(payload);

    return NextResponse.json(
      {
        ok: true,
        source: "reddit",
        live: true,
        items
      },
      { status: 200 }
    );
  } catch {
    return NextResponse.json(
      {
        ok: true,
        source: "reddit_fallback",
        live: false,
        items: FALLBACK_ITEMS
      },
      { status: 200 }
    );
  }
}
