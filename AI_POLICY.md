# AI policy

**Stance: all in.** AI-assisted contributions are welcome here, with no special
scrutiny and no separate queue. It would be a strange project to hold the line
anywhere else — it is called *vibegraph*, and most of it was written by a human
steering a model, one session at a time, against a MadGraph oracle.

What follows is what we ask, not a gauntlet. All of it is the same thing said
four ways: **a model can write the patch, but only you can vouch for it.**

## Disclose it

Name the harness and the model, in the commit message, as a trailer:

```
Assisted-by: claude-code:claude-opus-5
```

`<harness>:<model>`, last line, one per model if more than one was involved.
That is all — no announcement in the PR title, no badge, no apology.

We ask for the model's real identifier rather than a vague "AI-assisted"
because it is genuinely useful downstream: when a class of bug turns out to
correlate with a model or a harness, the trailer is what makes that visible.

## Do not let a model sign for you

Two trailers are off limits for models, no matter what your harness does by
default:

- **`Co-Authored-By:`** — a co-author is a person who shares responsibility for
  the code. A model holds none.
- **`Signed-off-by:`** — a sign-off certifies provenance and licensing. That is
  a legal claim, and only a human can make it.

Most harnesses will try to add themselves as a co-author unprompted. Turn it
off. (The instruction lives in `AGENTS.md`, which every harness reading this
repo should pick up; if yours does not, fix it in your own config.)

If you are working through an agent harness, `AGENTS.md` is also where the
project's conventions live — point your tool at it and most of this comes for
free.

## Understand every line you send

You are the author of a patch a model typed for you. If a reviewer asks why a
sign is where it is and the honest answer is "the model put it there," the patch
is not ready. This is the whole policy, really; everything else is bookkeeping.

## Physics is not reviewable by inspection

The one thing this project asks that a general Python library would not:

Matrix elements, colour algebra, phase-space maps and scale choices are exactly
the code where a wrong patch looks *completely* right. A model will produce an
amplitude routine that compiles, runs, has plausible magnitudes, and is off by a
phase, a colour-basis permutation, or a factor of 2 that cancels in the one case
you checked. Reviewers cannot catch that by reading, and neither can you.

So: **a physics change is claimed validated only against a gate that would have
failed before it.** `cargo test` is the floor. Anything touching amplitudes,
colour, couplings, enumeration, sampling or output goes through the MadGraph
comparison — `.agents/skills/extended-validation/SKILL.md` maps the change to
the gate, and `AGENTS.md`'s Physics Validation section explains why each oracle
is blind to what it is blind to. Tell us in the PR which gates you ran and what
they printed.

If you cannot run the heavy gates — they need reference data a fresh clone does
not have — say so plainly and let CI do it. That is fine. Claiming a gate you
did not run is not.

## Talk to us as people

Write the PR description yourself, in your own words. A model's summary of its
own diff is longer than the diff, oddly confident, and tells a reviewer nothing
they could not read faster from the patch. Same for review replies: if a human
took the time to read your code, answer them personally.

If a stretch of prose really is model-written and worth including anyway, just
say so inline. Nobody minds; it is the pretending that wastes people's time.

## What we will not accept

Not because it is AI, but because it is unreviewable:

- Mass-filed unsolicited PRs, or a batch of changes nobody asked for.
- Refactors whose justification is that a model suggested them.
- A change to a validated numerical result with no gate output attached.

A patch that takes longer to verify than to have written is a cost, not a
contribution — and in a codebase whose correctness is pinned to machine
precision against another generator, that verification cost is real.
