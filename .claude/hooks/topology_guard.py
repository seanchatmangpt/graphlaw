#!/usr/bin/env python3
"""PreToolUse guard: repository topology is transport, never ontology (operator 2026-09-23/24; extended v26.9.25, design P2 §5).

Mechanically refuses the actions that rebuilt shadow topology:
  - Bash: `git worktree add` (any form)
  - Bash: `git clone ... <dest>` whose destination is under a protected root:
          ~/wt/, any `/.claude/worktrees/`, /private/tmp/ and /tmp/, ~/*-worktrees/, ~/*-wt/, ~/*-wt<N>/, ~/wt-*/
  - Bash: copying a repository: `cp -R|-r|-a`, `rsync -a|-r`, `ditto` whose source is a directory containing .git
  - Bash: writing git state in a non-canonical working copy: `git -C X <verb>` / `cd X && git <verb>` with verb in
          commit|add|switch|checkout|merge|cherry-pick|push|stash|reset|rebase|am|apply, where X is under a protected root or
          is a linked worktree / shadow copy (a .git *file* that is not a submodule's modules/ pointer)
  - EnterWorktree tool; Agent tool with isolation "worktree"; Workflow scripts requesting agent({isolation: 'worktree'})
  - Write/Edit/MultiEdit/NotebookEdit whose nearest ancestor holding .git lies under a protected root or is a linked worktree,
    and any write under ~/wt/
Still allowed: reading; `git worktree remove|prune|list`; `mv <shadow> ~/.Trash/...`; `git -C <canonical> fetch <shadow> ...`
(reads the copy only); writes to the session scratchpad (/private/tmp/claude-501/**/scratchpad/) when no .git sits between the
file and the scratchpad root.
"""
import json
import os
import re
import shlex
import sys

HOME = os.path.expanduser("~")
WT = os.path.join(HOME, "wt") + "/"
CMD_PREFIX = r"(?:^|[;&|(\n]|\$\()\s*(?:sudo\s+|xargs\s+|env\s+(?:\S+=\S*\s+)*)?"
GIT_OPTS = r"(?:\s+(?:-C\s+(?P<C>\"[^\"]*\"|'[^']*'|\S+)|-c\s+\S+|--[\w-]+(?:=\S+)?))*"
CMD_GIT = CMD_PREFIX + r"git" + GIT_OPTS.replace("(?P<C>", "(?:") + r"\s+"
WORKTREE_ADD = re.compile(CMD_GIT + r"worktree\s+add\b", re.M)
CLONE = re.compile(CMD_GIT + r"clone\b(?P<args>[^\n;&|)]*)", re.M)
GIT_STASH = re.compile(CMD_GIT + r"stash(?P<sub>\s+\S+)*\b", re.M)
GIT_WRITE = re.compile(CMD_PREFIX + r"git" + GIT_OPTS + r"\s+(?P<verb>commit|add|switch|checkout|merge|cherry-pick|push|stash|reset|rebase|am|apply)\b", re.M)
CD_THEN = re.compile(r"(?:^|[;&|(\n])\s*cd\s+(?P<dir>\"[^\"]*\"|'[^']*'|[^\s;&|]+)\s*(?:&&|;)\s*(?P<rest>[^\n]*)", re.M)
COPY = re.compile(CMD_PREFIX + r"(?P<tool>cp|rsync|ditto)\b(?P<args>[^\n;&|)]*)", re.M)
WF_ISOLATION = re.compile(r"isolation\s*:\s*['\"]worktree['\"]")
PROTECTED = [re.compile(p) for p in (
    r"^" + re.escape(WT),
    r"/\.claude/worktrees(/|$)",
    r"^/private/tmp/", r"^/tmp/",
    r"^" + re.escape(HOME) + r"/[^/]+-worktrees(/|$)",
    r"^" + re.escape(HOME) + r"/[^/]+-wt[0-9]*(/|$)",
    r"^" + re.escape(HOME) + r"/wt-[^/]+(/|$)",
)]
SCRATCH = re.compile(r"^/private/tmp/claude-\d+/[^/]+/[^/]+/scratchpad(/|$)")
REASON = ("REFUSED(topology): one canonical checkout per repository, normal git branches, no git worktrees or shadow "
          "copies (operator 2026-09-23; ~/.claude/rules/local-dfcm-manufacturing-engine.md law 3). Use a normal branch "
          "in the repository's canonical checkout; for another repo's code at a SHA use `git show`/`git archive` into scratch.")


STASH_REASON = ("REFUSED(topology/stash): `git stash` is banned in shared checkouts — it sweeps other lanes' in-flight "
                "edits (same-checkout fan-out rule 1, operator 2026-10-08; ~/.claude/rules/same-checkout-fanout.md). "
                "Snapshot a single file via `git show HEAD:path > /tmp/...` and swap it back instead.")


def deny(why, reason=None):
    print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "deny",
                                             "permissionDecisionReason": f"{reason or REASON} [{why}]"}}))
    sys.exit(0)


def norm(p, cwd=None):
    p = p.strip().strip("'\"")
    p = os.path.expanduser(p)
    if not os.path.isabs(p):
        p = os.path.join(cwd or os.getcwd(), p)
    p = os.path.normpath(p)
    if p.startswith("/tmp/"):
        p = "/private" + p
    return p


def protected(p, hard_only=False):
    pats = PROTECTED[:2] + PROTECTED[4:] if hard_only else PROTECTED
    return any(r.search(p) for r in pats)


def linked_worktree_root(p):
    """Nearest ancestor (incl. p) holding .git; returns (root, is_shadow)."""
    d = p
    while d and d != "/":
        g = os.path.join(d, ".git")
        if os.path.isdir(g):
            return d, False
        if os.path.isfile(g):
            try:
                with open(g) as f:
                    t = f.read()
            except OSError:
                t = ""
            return d, "/modules/" not in t
        d = os.path.dirname(d)
    return None, False


def shadow_target(p):
    if SCRATCH.search(p):
        root, _ = linked_worktree_root(p)
        if not root or SCRATCH.search(root) is None or not root.startswith(SCRATCH.search(p).group(0).rstrip("/")):
            return False
        return True  # a repository inside the scratchpad is still a copy
    if protected(p):
        return True
    root, shadow = linked_worktree_root(p)
    if root and (shadow or protected(root)):
        return True
    return False


def clone_dest(args):
    try:
        toks = shlex.split(args)
    except ValueError:
        toks = args.split()
    pos, skip = [], False
    for i, t in enumerate(toks):
        if skip:
            skip = False
            continue
        if t in ("-b", "--branch", "-o", "--origin", "--depth", "-c", "--config", "--reference", "--separate-git-dir", "-u",
                 "--upload-pack", "--template", "-j", "--jobs", "--filter", "--shallow-since", "--shallow-exclude"):
            skip = True
            continue
        if t.startswith("-"):
            continue
        pos.append(t)
    if len(pos) >= 2:
        return pos[1]
    if len(pos) == 1:
        base = os.path.basename(pos[0].rstrip("/"))
        return base[:-4] if base.endswith(".git") else base
    return None


def copy_sources(tool, args):
    try:
        toks = shlex.split(args)
    except ValueError:
        toks = args.split()
    flags = [t for t in toks if t.startswith("-")]
    paths = [t for t in toks if not t.startswith("-")]
    if tool == "cp" and not any(re.match(r"^-[a-zA-Z]*[rRa]", f) for f in flags):
        return []
    if tool == "rsync" and not any(re.match(r"^-[a-zA-Z]*[ar]", f) or f in ("--recursive", "--archive") for f in flags):
        return []
    return paths[:-1]


def check_bash(cmd, cwd):
    if WORKTREE_ADD.search(cmd):
        deny("git worktree add")
    if GIT_STASH.search(cmd):
        deny(f"git stash in a shared checkout: {GIT_STASH.search(cmd).group(0)!r}", STASH_REASON)
    for m in CLONE.finditer(cmd):
        d = clone_dest(m.group("args"))
        if d and protected(norm(d, cwd)):
            deny(f"git clone into protected root: {d}")
    for m in COPY.finditer(cmd):
        for s in copy_sources(m.group("tool"), m.group("args")):
            sp = norm(s.rstrip("/") or "/", cwd)
            if os.path.isdir(sp) and os.path.exists(os.path.join(sp, ".git")):
                deny(f"{m.group('tool')} of a repository: {s}")
    targets = []
    for m in GIT_WRITE.finditer(cmd):
        if m.group("C"):
            targets.append((m.group("C"), m.group("verb")))
    for m in CD_THEN.finditer(cmd):
        rest = m.group("rest")
        g = GIT_WRITE.search(rest)
        if g and not g.group("C"):
            targets.append((m.group("dir"), g.group("verb")))
    for x, verb in targets:
        p = norm(x, cwd)
        if shadow_target(p):
            deny(f"git {verb} in a non-canonical working copy: {x}")


def main():
    try:
        ev = json.load(sys.stdin)
    except json.JSONDecodeError:
        return
    tool, inp = ev.get("tool_name", ""), ev.get("tool_input") or {}
    cwd = ev.get("cwd") or os.getcwd()
    if tool == "Bash":
        check_bash(inp.get("command", ""), cwd)
    elif tool == "EnterWorktree":
        deny("EnterWorktree")
    elif tool == "Agent" and inp.get("isolation") == "worktree":
        deny("Agent isolation=worktree")
    elif tool == "Workflow" and WF_ISOLATION.search(inp.get("script", "") or ""):
        deny("Workflow agent isolation=worktree")
    elif tool in ("Write", "Edit", "MultiEdit", "NotebookEdit"):
        p = inp.get("file_path") or inp.get("notebook_path") or ""
        ap = norm(p, cwd) if p else ""
        if ap.startswith(WT):
            deny(f"write under ~/wt: {p}")
        if ap and SCRATCH.search(ap):
            root, _ = linked_worktree_root(ap)
            if root and root.startswith(SCRATCH.search(ap).group(0).rstrip("/")):
                deny(f"write into a repository inside the scratchpad: {p}")
            return
        if ap and protected(ap, hard_only=True):
            deny(f"write under a protected worktree root: {p}")
        if ap:
            root, shadow = linked_worktree_root(ap)
            if root and (shadow or protected(root)):
                deny(f"write into a non-canonical working copy: {p}")


if __name__ == "__main__":
    main()
