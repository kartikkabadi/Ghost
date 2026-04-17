import { NextResponse } from "next/server";

export const dynamic = "force-dynamic";
export const revalidate = 0;
export const fetchCache = "force-no-store";

const REDDIT_URL = "https://www.reddit.com/r/popular.json?limit=10&t=day";

export async function GET() {
  try {
    const response = await fetch(REDDIT_URL, {
      headers: {
        "User-Agent": "Ghost OSINT Harness/1.0"
      },
      cache: "no-store"
    });

    if (!response.ok) {
      return NextResponse.json(
        {
          ok: false,
          source: "reddit",
          status: response.status,
          items: []
        },
        { status: 200 }
      );
    }

    const payload = await response.json();
    const items = Array.isArray(payload?.data?.children)
      ? payload.data.children.map((entry: any) => ({
          title: entry?.data?.title ?? "",
          subreddit: entry?.data?.subreddit ?? "",
          score: entry?.data?.score ?? 0,
          url: entry?.data?.url ?? ""
        }))
      : [];

    return NextResponse.json(
      {
        ok: true,
        source: "reddit",
        items
      },
      { status: 200 }
    );
  } catch {
    return NextResponse.json(
      {
        ok: false,
        source: "reddit",
        items: []
      },
      { status: 200 }
    );
  }
}
