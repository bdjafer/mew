# WRITE PR

1. RUN TESTS FIRST
   - Execute the full test suite
   - Capture the exact counts: X/Y passing, Z failing
   - This is your baseline

2. CATEGORIZE FAILURES
   Group errors by type:
   - Parse errors → Parser
   - Compile errors → Compiler/schema  
   - Assertion failures → Semantic/logic
   - Step execution errors → Runtime
   
   Attack categories in order: parse → compile → runtime → semantic

3. PICK ONE SPECIFIC TEST
   - Choose the simplest failing test in the highest-priority category
   - Run it with --nocapture to get full error output
   - Extract the EXACT error message

4. TRACE TO ROOT CAUSE
   - Read the spec in `specs/specification/*.md` that defines expected behavior
   - Read the code that implements it
   - Identify the gap between spec and implementation
   - Don't guess - find the actual line(s) causing the issue

5. IMPLEMENT
   - Make the minimal change needed
   - Follow existing code patterns
   - Update all places that match on enums/types (exhaustive matches)

6. BUILD AND VERIFY
   - Compile first (fix all compile errors)
   - Run the specific test that was failing
   - Confirm it passes before moving on

7. COMMIT LOGICAL CHUNKS
   - When a category of bugs is fixed, commit
   - Message: what changed, why, test counts
   - Push to preserve progress
