# Issue tracker: GitHub

Issues and PRDs for this repo live as GitHub issues. Use the `gh` CLI for all operations.

## Conventions

- **Create an issue**: `gh issue create --title "..." --body "..."`. Use a heredoc for multi-line bodies.
- **Read an issue**: `gh issue view <number> --comments`, filtering comments by `jq` and also fetching labels.
- **List issues**: `gh issue list --state open --json number,title,body,labels,comments --jq '[.[] | {number, title, body, labels: [.labels[].name], comments: [.comments[].body]}]'` with appropriate `--label` and `--state` filters.
- **Comment on an issue**: `gh issue comment <number> --body "..."`
- **Apply / remove labels**: `gh issue edit <number> --add-label "..."` / `--remove-label "..."`
- **Close**: `gh issue close <number> --comment "..."`

Infer the repo from `git remote -v` — `gh` does this automatically when run inside a clone.

## When a skill says "publish to the issue tracker"

Create a GitHub issue.

## When a skill says "fetch the relevant ticket"

Run `gh issue view <number> --comments`.

## Wayfinding operations

The wayfinder skill charts multi-session efforts as a labelled **map issue** with **child ticket issues**. Conventions on this repo:

- **Where the map lives**: a GitHub issue labelled `wayfinder:map`. Its body carries the Destination, Notes, Decisions-so-far, Not yet specified, and Out of scope sections.
- **Where child tickets live**: separate GitHub issues, each labelled with one of `wayfinder:research`, `wayfinder:grilling`, `wayfinder:prototype`, or `wayfinder:task`. Native GitHub sub-issues are **not** used — each ticket body ends with `**Parent**: #<map-id>`.
- **How blocking is expressed**: body convention `**Blocked by**: #<id>, #<id>` at the end of a ticket. Native issue dependencies (`trackedIssues`) are **not** wired.
- **How the frontier is queried**: list open, unassigned tickets whose labels include any of the four ticket types, then post-filter by parsing each body's `**Blocked by**:` line — a ticket is on the frontier when every issue it names is closed.

  ```bash
  gh issue list \
    --state open \
    --search 'label:wayfinder:research,wayfinder:grilling,wayfinder:prototype,wayfinder:task no:assignee' \
    --json number,title,body,labels
  ```

- **How a ticket is claimed**: `gh issue edit <id> --add-assignee "@me"` before any work. The assignee is the claim; a ticket with no assignee is unclaimed.
- **How a ticket is resolved**: post a resolution comment with the answer and any linked assets, close via `gh issue close <id>` (or via a PR whose merge closes the ticket through `Closes #<id>`), and append a one-line pointer to the map's Decisions-so-far via `gh issue edit <map-id> --body-file <file>`.

Category (`bug`/`enhancement`) and component (`rust`/`python`/`infra`) labels still apply to wayfinder tickets.
