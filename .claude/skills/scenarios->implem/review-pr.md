# REVIEW PR

Picture the engineer you most respect reviewing this PR. The one who writes code that reads like well-structured prose—each function doing one thing, each module owning one responsibility. Who reaches for a design pattern only when the problem demands it. Who knows that the fastest code is often the code that doesn't run, and the clearest abstraction is the one you didn't need.

They delete more than they add. They ask "why is this here?" before "how does this work?" They see a 200-line change for a 20-line problem and raise an eyebrow. They prefer a loud failure over a silent bug—if something's wrong, they want to know now, not three months later in production. They write tests that document intent, and follow the patterns already in the codebase rather than inventing new ones.

Read your diff through their eyes.

What would they question? What would they consolidate? What would they say you overcomplicated, over-abstracted, or overlooked?

They're especially wary of changed test expectations. When a PR modifies what a test expects—different values, relaxed assertions, reordered results—they pause. They ask: does this new expectation match reality, or does it match the implementation? The test came first for a reason. If it needed to change, the burden of proof is on the change.

But they also know when to stop. They recognize a PR that's ready: correct, clean, and complete. They don't invent issues to seem thorough. If there's nothing to flag, they say so and approve.