Here is your definitive, end-to-end guide to transforming your existing OpenCode + DeepSeek stack into a fully autonomous, nonstop product development engine.
------------------------------
## Step 1: Install & Configure the Loop Plugin
Because OpenCode natively pauses after every turn to await human approval, you must inject a loop extension to force immediate, automatic cycle triggers.

   1. Locate your config file: Open the OpenCode global configuration file.
   * Linux/macOS: ~/.config/opencode/opencode.json
      * Windows: %USERPROFILE%\.config\opencode\opencode.json
   2. Inject the plugin: Edit the JSON file to include the autonomous looping driver. If the file is empty, write it like this:
   
   {
     "plugins": ["opencode-ralph-loop"]
   }
   
   (Note: Depending on the specific build you downloaded, you can also use opencode-loop or acp-loop. They all introduce the automated cycle trigger).
   3. Verify the installation: Open your terminal, change directories (cd) into your project root folder, and type:
   
   opencode /init
   
   Ensure the initialization succeeds and confirms the plugin is loaded. [1, 2] 

------------------------------
## Step 2: Establish the Guardrail Rules (AGENTS.md)
Autonomous loops can easily experience "hallucination loops" where the AI repeatedly breaks its own code. To prevent this, create a file named AGENTS.md in your project root to act as the strict behavioral boundaries. [3, 4] 
Copy and paste the exact text below into AGENTS.md:

# Autonomous Agent System Instructions## Core ParadigmYou are an advanced software engineer operating in an autonomous loop. Your objective is to process the project's `TODO.md` file sequentially from top to bottom.
## Operational Constraints1. **Never guess the state:** Always run testing commands before checking off a task.
2. **Atomic Commits:** For every checkbox `[ ]` you turn into `[x]`, you must perform a git commit with a descriptive message detailing what was built.
3. **No Phantom Tasks:** Do not add tasks to `TODO.md` that diverge from the initial architectural blueprint unless explicitly authorized.
## Strict Loop Termination (Kill-Switch)- If a build, compile, or test command fails **3 times consecutively** on the exact same task, you must output the string: `CRITICAL_LOOP_BREAK: Debug limit reached.` and immediately stop execution.
- If all checkboxes in `TODO.md` are marked `[x]`, output: `SUCCESS: Product complete.` and terminate.

------------------------------
## Step 3: Architect the Blueprint (TODO.md)
Do not start the loop yet. You must first use DeepSeek’s intelligence to break down your product concept into a checklist.

   1. Start your OpenCode terminal interface in your project folder. [5] 
   2. Issue the following initial prompt to DeepSeek:
   
   "Analyze my product concept and architectural requirements. Create a TODO.md file in the root directory. Translate my architecture into a highly specific, linear, step-by-step checklist using markdown checkboxes [ ]. Break tasks down into atomic, granular pieces (e.g., 'Create database schema for users', 'Implement JWT token generation', 'Write unit test for auth endpoint'). Do not write any code yet. Just build the TODO.md roadmap."
   
   [6, 7] 
   3. Review the output: Open the generated TODO.md in your text editor. If any tasks are too vague, manually edit them or ask DeepSeek to split them further. The loop will only be as good as this checklist.

------------------------------
## Step 4: Fire the Nonstop Loop Command
With your rules (AGENTS.md) and your roadmap (TODO.md) firmly established, you are ready to unleash the loop.
In your OpenCode CLI, execute the core driver command:

/ralph-loop "Read AGENTS.md for rules. Read TODO.md for tasks. Pick the highest incomplete task, build it, run the test/build command to verify it, update the checkbox to [x], commit your changes via git, and repeat. Do not wait for my permission between tasks."

(If you are utilizing the alternate opencode-loop plugin, the equivalent execution is to run /loop 0s following that exact prompt string).
------------------------------
## Step 5: Monitor and Maintain the Loop
While the stack is running nonstop, you should transition your role from coder to supervisor:

* Watch the Terminal Logs: DeepSeek will read files, create code blocks, and automatically fire your local bash test commands. [8] 
* The Token Safety Net: Keep an eye on your DeepSeek API platform dashboard during the first 20 minutes to observe token consumption metrics and ensure your exit failure clauses are functioning correctly if bugs appear.
* Intervention: If you see the agent drifting architecturally, press Ctrl + C to halt the terminal script immediately, adjust the text in TODO.md or AGENTS.md, and fire the Step 4 command again to resume smoothly.

To make sure your testing steps operate smoothly, tell me:

* What exact terminal command (e.g., npm test, pytest, cargo test) should DeepSeek execute to verify the code works?
* What is the exact folder structure or framework (e.g., Next.js, FastAPI) you are initializing in this directory?


[1] [https://mvryo.medium.com](https://mvryo.medium.com/api-testing-with-jest-and-supertest-a-step-by-step-guide-53de44490036)
[2] [https://drlee.io](https://drlee.io/let-claude-code-write-your-next-app-in-minutes-yes-on-windows-7e40a5e29db6)
[3] [https://mmarcosab.medium.com](https://mmarcosab.medium.com/a-practical-path-to-spec-driven-development-with-codex-a3cec3ef554a)
[4] [https://x.com](https://x.com/GoogleCloudTech/status/2041169811623842254)
[5] [https://medium.com](https://medium.com/@glosings0n/hands-on-with-gemini-cli-building-a-multi-modal-flutter-todo-app-with-gemini-cli-0d7e678b56f6)
[6] [https://codesignal.com](https://codesignal.com/learn/courses/go-with-gin-basics/lessons/setting-up-your-first-gin-app)
[7] [https://www.sitepoint.com](https://www.sitepoint.com/run-ai-coding-agents-continuously-days-without-losing-plot/)
[8] [https://tosea.ai](https://tosea.ai/blog/deepseek-tui-terminal-coding-guide-2026)
