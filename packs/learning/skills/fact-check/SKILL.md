---
name: fact-check
description: "Verifies the facts of existing or in-progress content: spots high-risk claims (figures, dates, absolute statements, contradictions), checks them against sources, and produces a short report with evidence. Use to make a course, an article or documentation reliable before publication."
license: MIT
---

# Fact Check — Factual Verification

Keep serious factual errors and hallucinations out of a content, without turning writing into an exhaustive audit. Focus on the few claims that really carry the reader's trust.

## Two modes (choose based on the request, do not ask the user to choose)

- **Creation**: the content does not exist yet or is being built → verify **after** its production, before delivery.
- **Review**: the content already exists and inspection is requested → report first. Only modify if corrections have been requested or approved after the report.

Out of scope: grammar, style, layout.

## In creation

Produce the content normally. Once finished, re-read it in full (visible text + narration). Do a quick health check on exact facts and contradictions between sections. Correct obvious errors before delivery — creating the content authorizes making it factually correct. At delivery, briefly mention only material corrections or remaining uncertainties. Do not cut creation short with a separate audit report, unless a claim depends on a user decision.

## In review of existing content

1. **Read the content in full** (visible text and narration), within the requested scope.
2. **Read once for context** and silently retain a **short list of high-signal risks**:
   - numbers, dates, counts, exact names and attributed quotes;
   - laws, standards, formulas, technical definitions and classifications;
   - "first", "only", "always", "mandatory" and other absolute claims;
   - causal or professional conclusions presented as established facts;
   - contradictions between sections;
   - very specific claims without visible support.
3. **Do not verify everything.** Skip correct content, wording preferences, harmless simplifications and low-value trivia.

## Verify only the short list

- Consult the user's provided materials first: the user's sources can support a claim, but the content being verified cannot prove itself.
- Use web research for short-list claims that depend on current, exact, contested or specialized knowledge. A first normal pass should not exceed **6 to 8 searches**. Retrieve the source to read it: a result snippet or the mere existence of a nearby source is not proof.
- Do not pause the session to ask for sources or permissions. Use the available tools. If web research is unavailable, continue with stable knowledge, make fewer factual commitments, and mark uncertain claims. **Never guess a URL**: only retrieve a URL provided by the user or returned by a search.
- For a compound statement, isolate the doubtful part and verify exactly that part. Prefer primary or official sources. One authoritative source suffices for an obvious error; add corroboration only for contested or high-impact claims. For versioned knowledge (law, policy, standards, medicine), verify the relevant date, version and jurisdiction.
- "No reliable evidence found" does not mean "false". If verification remains inconclusive, say so rather than inventing a verdict or a correction.

## Preserve approved inputs

Do not make a correction that would materially conflict with the validated brief, the materials provided by the user, or facts already transmitted as generation inputs. Treat them as approved inputs, not as ordinary generated text.

If the evidence indicates that an approved input itself contains a factual error: do not modify the concerned content and do not silently overload the input. Flag the conflict to the user, indicate the concerned section or claim and the contrary evidence concisely, and propose to keep the approved input, allow the factual correction, or review the conflict without modifying.

## The review report (short and readable)

In review mode, return about **3 to 8 useful findings** on the first pass (or fewer if there are fewer). Group them under these headings, in this order, and omit an empty group:

- **A. Confirmed factual error**
- **B. Misleading formulation**
- **C. Needs verification** — only when the claim matters

Number the findings continuously throughout the report. Each finding has a short bold line with its number, its location and its problem, for example: `**1. Section 5, "quiz on meetings": confusion about thresholds**`.

Under each heading, exactly three bullets:

- **Original statement:** quote only the concerned sentence or fragment;
- **Observed problem:** explain the error and the correct fact in plain language; indicate a concise source and its date when useful;
- **Proposed correction:** give only the edit action or a compact replacement.

One bullet per idea, short. Do not repeat the same fact or the same quote from one bullet to another. Do not show scores, confidence percentages, endless methodology, correct claims or minor style problems. Do not pad the report to reach a quota. If no material problem is found, say which scope was covered and that no obvious error was found; do not claim the content is perfectly accurate.

## Let the user choose after a review

When there are actionable findings and the corrections have not already been authorized, the last action of the turn offers the user a choice between:

- fix all reported problems;
- fix only the confirmed errors;
- keep the report without modification.

Modify nothing before the answer. After approval, re-read each selected section and change only the approved claims.

## Verification before finishing

- [ ] Only high-signal claims were verified, not the whole content.
- [ ] Each finding has a location, evidence and a proposed correction.
- [ ] Approved inputs (brief, materials, transmitted facts) were not contradicted without flagging.
- [ ] The inconclusive is stated as such, never presented as a certainty.
