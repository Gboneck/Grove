import { NextRequest, NextResponse } from "next/server";
import { getSoulMd } from "@/lib/soul";
import { getContext } from "@/lib/context";

const SYSTEM_PROMPT = `You are the reasoning engine for Grove OS, a personal operating system.
You are given a Soul.md (the user's identity document), a context.json
(their current ventures and state), and the current timestamp.

Your job: decide what to show this person RIGHT NOW. You are not a chatbot.
You are the brain of their operating system. Be opinionated. Be direct.
Prioritize ruthlessly.

Rules:
- Return ONLY valid JSON. No markdown, no explanation, no preamble.
- Return an object with a single key "blocks" containing an array.
- Each block has a "type" (one of: text, metric, actions, status, insight, input, divider)
- Think about: What time is it? What day? What's urgent? What's the user's current state?
- Morning = briefing energy. Evening = reflection. Weekend = different rhythm.
- Never show more than 8-10 blocks. Density kills usefulness.
- Write in a direct, warm, no-bullshit voice.

Block schemas:

{ "type": "text", "heading": "string", "body": "string" }
{ "type": "metric", "label": "string", "value": "string", "trend": "up|down|flat|null" }
{ "type": "actions", "title": "string", "items": [{ "action": "string", "detail": "string" }] }
{ "type": "status", "items": [{ "name": "string", "status": "green|yellow|red", "detail": "string" }] }
{ "type": "insight", "icon": "alert|opportunity|warning|idea", "message": "string" }
{ "type": "input", "prompt": "string", "placeholder": "string" }
{ "type": "divider" }`;

export async function POST(request: NextRequest) {
  try {
    const { userInput } = await request.json();

    const soulMd = getSoulMd();
    const contextData = getContext();
    const now = new Date();

    const userMessage = `Current time: ${now.toISOString()}
Day: ${now.toLocaleDateString("en-US", { weekday: "long" })}
Date: ${now.toLocaleDateString("en-US", { month: "long", day: "numeric", year: "numeric" })}

--- SOUL.MD ---
${soulMd}

--- CONTEXT ---
${JSON.stringify(contextData, null, 2)}

${userInput ? `--- USER INPUT ---\n${userInput}` : ""}

Decide what to show. Return JSON only.`;

    const response = await fetch("https://api.anthropic.com/v1/messages", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "x-api-key": process.env.ANTHROPIC_API_KEY || "",
        "anthropic-version": "2023-06-01",
      },
      body: JSON.stringify({
        model: "claude-sonnet-4-20250514",
        max_tokens: 2000,
        system: SYSTEM_PROMPT,
        messages: [
          {
            role: "user",
            content: userMessage,
          },
        ],
      }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.error("Claude API error:", response.status, errorText);
      return NextResponse.json(
        { error: "Reasoning engine unavailable", detail: errorText },
        { status: 502 }
      );
    }

    const data = await response.json();
    const content = data.content?.[0]?.text;

    if (!content) {
      return NextResponse.json(
        { error: "Empty response from reasoning engine" },
        { status: 502 }
      );
    }

    const parsed = JSON.parse(content);
    return NextResponse.json(parsed);
  } catch (error) {
    console.error("Reasoning error:", error);
    return NextResponse.json(
      { error: "Failed to reason", detail: String(error) },
      { status: 500 }
    );
  }
}
