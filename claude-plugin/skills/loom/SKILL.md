---
name: loom
description: >
  Work with the user's Loom graph (a self-hosted tracker of projects, studies and ideas) through
  the `loom` MCP server's `loom_*` tools. Use it when the user mentions Loom, or asks to plan or
  build a learning path, roadmap or curriculum, capture an idea, promote an idea into a project,
  log progress on a course or book, review what they're working on or what's stale or blocked,
  or organise nodes, edges, checklists and topics. Also use it when setting up or troubleshooting
  the Loom MCP connection.
---

# Loom

Loom is a single-user graph for juggling many parallel projects, courses and half-formed ideas.
Every item is a **node**, and nodes are connected by **edges**. You work with it through the
`loom_*` MCP tools. If those tools aren't available, the connection isn't set up yet; see
[references/setup.md](references/setup.md).

## Domain model

**Node kinds.** A kind decides what a node can hold, and the server rejects anything else:

| Kind      | What it is                                  | Can hold                                                                                |
| --------- | ------------------------------------------- | --------------------------------------------------------------------------------------- |
| `idea`    | An uncommitted capture                      | Notes only                                                                              |
| `project` | Something to build or do                    | A **checklist**                                                                         |
| `study`   | A course, book, video series or topic       | A **progress counter**: `progress_current` / `progress_total` / `progress_unit` (e.g. 3 / 12 "chapters") |
| `path`    | A container, like a learning path or roadmap | **Children**: other nodes join it with a `part_of` edge. Paths can nest.               |

**Status** runs `idea` (the backlog) → `queued` → `active` → `paused` → `done`, or `archived`.
Any node with status `idea` counts as backlog, whatever its kind.

**Focus tier** is `primary`, `secondary` or `background`. It is separate from status, and it
decides what the dashboard surfaces. Keep `primary` for the few things the user is actively
pushing.

**Notes** are free-form Markdown. Use them for descriptions, links, resources and acceptance
criteria.

**Topics** are free-form tags such as "rust" or "math". They never block anything.

**Edges** always point *from* the node being described:

- `requires`: `from` depends on `to`. `from` stays **blocked** until `to` is `done`. Use this for
  order: "Lesson 2 requires Lesson 1". Cycles are rejected.
- `part_of`: `from` is a child of the path `to`. The target must be a `path`. A node sits in at
  most one path. Cycles are rejected.
- `related`: a soft link with no effect. Cycles are allowed.

**Derived fields.** These are computed and read-only, so never try to set them:

- `blocked`
- `container_progress`: done/total of a path's children
- `checklist_progress`
- `last_poked_at`

**Pokes and staleness.** A **poke** is an "I worked on this" log entry. An `active` or `queued`
node with no poke for 14 days is **stale**.

**Promoting** means changing a node's `kind` in place, keeping the same id and edges. The node
loses whatever the new kind can't hold:

- leaving `project` deletes its checklist
- leaving `study` clears its progress
- leaving `path` releases its children

Always tell the user what will be lost, and confirm before promoting.

## Tools

| Tool | Use it to |
| --- | --- |
| `loom_get_overview` | Get dashboard counts plus the primary, stale and recent-backlog nodes and paths. Good for "what am I working on?" |
| `loom_get_graph` | Get every node, edge and topic. **Start here before any bulk change.** |
| `loom_list_nodes` | Filter nodes by `kind` / `status` / `focus` / `view` (`backlog`, `archived`), or by a title `search`. |
| `loom_get_node` | Get one node with its edges both ways, its checklist and recent pokes. |
| `loom_create_node` | Create one node. It can also take `checklist` (projects) and `topic_ids`. |
| `loom_create_subgraph` | Create many nodes and the edges between them in **one all-or-nothing call**. |
| `loom_update_node` | Change a node's fields. Send only the fields that change; `null` clears a field. A new `kind` promotes the node. |
| `loom_delete_node` | Delete a node for good, along with its edges, checklist and pokes. |
| `loom_create_edge` / `loom_delete_edge` | Connect or disconnect nodes. |
| `loom_add_checklist_items` / `loom_update_checklist_item` / `loom_delete_checklist_item` | Manage a project's checklist. `done: true` ticks an item. |
| `loom_list_topics` / `loom_create_topic` / `loom_update_topic` / `loom_delete_topic` | Manage tags. |
| `loom_attach_topic` / `loom_detach_topic` | Tag or untag a node. |
| `loom_poke_node` | Log work on a node. |

When a tool fails (validation, not found, conflict, cycle), it returns an error result with a
plain message. Read the message, fix the input and retry. Don't guess ids: look them up.

## Working rules

1. **Read before you write.** Call `loom_get_graph` or `loom_list_nodes` first. Reuse existing
   nodes, paths and topics instead of creating duplicates. A topic name that already exists is
   a conflict.
2. **Ask before destructive or sweeping changes.** That covers deleting nodes, promoting away
   from a kind that holds data, and rewriting many nodes at once. Creating things the user asked
   for needs no extra confirmation.
3. **Don't invent progress.** Only set `status: done`, tick checklist items or raise
   `progress_current` when the user says the work happened. Log real work with a poke.
4. **New nodes start in the backlog.** The default status is `idea`, which is right for captured
   ideas. For a plan the user wants to start, set the first step `queued` or `active` and leave
   the rest `queued`.
5. **Pick the kind by what the node needs.** Something to read or watch with a count is a
   `study`. Something to build with steps is a `project`. A grouping is a `path`. Anything not
   yet committed is an `idea`.
6. **Keep notes useful.** Put a short description, key resources (links) and what "done" means
   in `notes`, as Markdown.
7. After creating things, **summarise what you made**: titles, kinds and how they connect. Mention
   that the user will find it on the Board, under the path's box on the canvas.

## Recipe: build a learning path

For "Create a learning path for X":

1. Call `loom_get_graph` to check for an existing path or nodes on X, and to find topics to reuse.
2. Design 4–10 steps. Use a `study` per course, book or doc set, with `progress_total` and
   `progress_unit` when the size is known (e.g. 12 "chapters"). Use a `project` for hands-on
   practice, with a `checklist` of milestones.
3. Make **one** `loom_create_subgraph` call:
   - `nodes`: the `path` (the goal, with notes on scope) plus every step, each with a `ref`
     (`"path"`, `"s1"`, `"s2"`, …), a clear title and Markdown `notes`.
   - `edges`:
     - each step `part_of` the path
     - `requires` edges for the order: `{"from": "s2", "to": "s1", "kind": "requires"}`. Add
       these only where a real prerequisite exists; parallel steps get none.
     - optionally `related` links to existing nodes, using their ids.
4. If topics fit, pass existing `topic_ids` on the nodes. You can also create one topic first
   with `loom_create_topic` and use its id.
5. Report back:
   - the path
   - the steps in order
   - which steps are blocked until earlier ones are done

Example `loom_create_subgraph` arguments:

```json
{
  "nodes": [
    {"ref": "path", "kind": "path", "title": "Learn async Rust", "status": "queued",
     "notes": "Goal: write and debug async services with Tokio."},
    {"ref": "book", "kind": "study", "title": "Asynchronous Programming in Rust (book)",
     "status": "queued", "progress_current": 0, "progress_total": 10, "progress_unit": "chapters",
     "notes": "https://rust-lang.github.io/async-book/"},
    {"ref": "tokio", "kind": "study", "title": "Tokio tutorial", "status": "queued",
     "progress_current": 0, "progress_total": 9, "progress_unit": "sections"},
    {"ref": "chat", "kind": "project", "title": "Build a chat server", "status": "queued",
     "checklist": ["Accept TCP connections", "Broadcast messages", "Graceful shutdown"]}
  ],
  "edges": [
    {"from": "book", "to": "path", "kind": "part_of"},
    {"from": "tokio", "to": "path", "kind": "part_of"},
    {"from": "chat", "to": "path", "kind": "part_of"},
    {"from": "tokio", "to": "book", "kind": "requires"},
    {"from": "chat", "to": "tokio", "kind": "requires"}
  ]
}
```

## Other common requests

- **"Capture an idea"**: `loom_create_node` with `kind: idea`. Put the gist in `notes`. Don't
  over-structure it.
- **"Turn idea X into a project"**: `loom_update_node` with `kind: project` and usually
  `status: queued`. Then add a checklist with `loom_add_checklist_items`. An idea holds nothing
  that could be lost, so this is safe.
- **"I read two more chapters of Y"**: `loom_get_node` to read `progress_current`, then
  `loom_update_node` with the new value, then `loom_poke_node`.
- **"What should I work on?"**: call `loom_get_overview`, then weigh these:
  - primary-focus nodes
  - active nodes that aren't blocked
  - stale nodes
  - paths with little progress

  Recommend next steps; don't change anything unless asked.
- **"Move X into path Y"**: a node can be in only one path. Check `loom_get_node` for an
  existing outgoing `part_of` edge, delete it, then create the new one.
