# EdTech & Academia Outreach Templates

## Target: Computer Science Professors (Data Structures & Algorithms)

**Subject:** Stop failing students over syntax errors (UEAS v4.0.0 is live)

Hi Prof. [Name],

I noticed you're teaching [Course Name] this semester. I built an open-source tool called UEAS (Universal Executable Algorithm Standard) that might fundamentally change how you grade algorithm assignments.

Currently, students write pseudocode on paper (which can't be executed) or write in Java/Python (which introduces syntax/boilerplate errors unrelated to their algorithmic logic). Also, grading big-O complexity usually relies on flaky millisecond-timeout scripts.

UEAS fixes this:
1. Students write in a mathematically pure pseudocode (CLRS / LaTeX style).
2. The Rust kernel executes it directly, tracking exact abstract operations.
3. If an assignment requires an `O(N log N)` solution and they write an `O(N^2)` algorithm, the UEAS kernel traps instantly with a `ComplexityViolation`.

It’s 100% free and open-source. You can try the browser-based WASM playground here: [Link to Vercel/Cloudflare]. There is also a Jupyter Notebook integration for seamless classroom use.

Would love your feedback on the specification!

Best,
[Your Name]
