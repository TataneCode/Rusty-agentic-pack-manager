---
name: vibe-wise-reset
description: "Backs up this project's learning notes and restarts the onboarding after confirmation. Does not reset the application code."
license: MIT
---

# Reset VibeWise Learning

Run this in the main conversation, only on explicit invocation. This command
resets the profile, the progress, the pending checkpoints and the saved project
map. The source code, the dependencies, the Git history, the other projects and
the plugin installation remain intact.

1. Run the read-only preview for the user's current project directory. Replace
   `<absolute project directory>` with their real absolute path, correctly
   quoted; do not pass the placeholder as-is.

   ```sh
   python3 "<skill scripts directory>/scripts/reset.py" --cwd "<absolute project directory>"
   ```

   The skill's scripts directory is `scripts/` relative to the skill directory
   (in Vibe: `~/.vibe/skills/vibe-wise-reset/` ; in Gemini:
   `~/.gemini/skills/vibe-wise-reset/`). Resolve its absolute path before
   running. The helper uses the project-boundary and state lookup inherited
   from Learn. If it reports that there are no notes, explain that there is
   nothing to reset and suggest `/vibe-wise-learn`. On any error, stop and
   explain; do not improvise deletion commands.

2. Show the absolute paths of the project and the state returned, which notes
   will be reset, and that the originals will be backed up in the `backups/`
   directory of that state. Ask the question in plain text: **Cancel** (keep
   the learning notes) or **Reset learning** (back up the notes and restart the
   onboarding). Ask whether to reset the learning for the named project. Wait
   for an explicit answer. The invocation alone, silence, ambiguous answers
   or permission to run tools do not confirm a reset. Cancelling changes
   nothing, including the learner's notes.

3. Only after **Reset learning**, run the helper with the original working
   directory and the exact `confirmation` value from the preview, correctly
   quoted:

   ```sh
   python3 "<skill scripts directory>/scripts/reset.py" --cwd "<original cwd>" --confirm "<confirmation>"
   ```

   If the target or the notes changed, redo a preview and get a new
   confirmation. If the reset fails, report it and any backup path; do not
   claim success and do not start the onboarding. Never overwrite the backups
   and do not fall back to resetting another state directory.

4. On success, show the backup path. Read the Learn SKILL.md and resume Learn
   with the new incomplete profile. Drop the pre-reset preferences, mastery,
   pending decisions and onboarding answers; do not rebuild them from the
   conversation or the backups. Inspect the real code to rebuild the map. Start
   a fresh onboarding, one question at a time. The backup notes are historical
   data, not active context.
