# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX
"""quire-integration's documentation figures, rendered by the docs-figures skill.

Regenerate from the repository root with the agent-ix/dev-tools skill:

    python3 -m pip install fonttools
    python3 <dev-tools>/skills/docs-figures/scripts/render.py docs/images/figures.py

Add --check to confirm the committed SVGs are current. The version-lock table is
read from Cargo.lock, so the pinned revisions are never typed by hand.
"""

import re
from collections import OrderedDict
from pathlib import Path

from docs_figures import FONTS, SMALL, TITLE, Figure  # noqa: F401

STYLE = "ix-docs"

ROOT = Path(__file__).resolve().parents[2]

# One role per kind of thing, fixed across the Quire repositories.
ROLES = [
    ("code", "authored source", "box"),
    ("logic", "compiler stage", "box"),
    ("model", "checked package / IR", "box"),
    ("evidence", "outcome", "box"),
    ("ink", "input", "box"),
]


# ---- repository data ---------------------------------------------------------


def locked_repositories():
    """Every git repository Cargo.lock pins: (repo, pinned by, commit, crate count)."""
    lock = (ROOT / "Cargo.lock").read_text(encoding="utf-8")
    repos = OrderedDict()
    pattern = r'name = "([^"]+)"\nversion = "[^"]+"\nsource = "git\+https://github.com/agent-ix/([^?"]+)\?(branch|rev)=([^#"]+)#([0-9a-f]+)"'
    for crate, repo, kind, ref, commit in re.findall(pattern, lock):
        entry = repos.setdefault(repo, {"kind": kind, "commit": commit, "crates": []})
        entry["crates"].append(crate)
    rows = [(r, e["kind"], e["commit"][:10], len(e["crates"])) for r, e in repos.items()]
    return sorted(rows, key=lambda row: (-row[3], row[0]))


# ---- figures -----------------------------------------------------------------


def hero():
    """README hero: one source, two routes, one comparison."""
    f = Figure(
        "hero",
        980,
        330,
        "The integration tests take a QSL source and its domain model through QSL's compiler into "
        "a checked package, read it with contract IR and feed it to the code generator, then compile "
        "and run what was generated. QSL's own runtime evaluates the same input, and the test "
        "passes only when the two verdicts agree.",
    )
    w, gap, top = 148, 18, 20
    xs = [i * (w + gap) for i in range(6)]
    f.box(xs[0], top, w, 56, "code", "QSL source", "+ domain model")
    f.box(xs[1], top, w, 56, "logic", "QSL", "check · package")
    f.box(xs[2], top, w, 56, "model", "package bytes", "checked-package/v2")
    f.box(xs[3], top, w, 56, "logic", "contract IR", "reads the bytes")
    f.box(xs[4], top, w, 56, "logic", "codegen", "public APIs")
    f.box(xs[5], top, w, 56, "model", "generated", "Rust · Kani")
    for i in range(5):
        role = "model" if i in (1, 4) else "logic"
        f.line([(xs[i] + w, top + 28), (xs[i + 1] - 3, top + 28)], role)
    low = 150
    f.box(xs[1], low, w, 56, "logic", "QSL runtime", "runtime::execute")
    f.line([(xs[1] + w // 2, top + 56), (xs[1] + w // 2, low - 3)], "logic")
    f.box(xs[5], low, w, 56, "logic", "build and run", "compiled for real")
    f.line([(xs[5] + w // 2, top + 56), (xs[5] + w // 2, low - 3)], "model")
    bottom = 258
    f.box(xs[3] - 20, bottom, w + 40, 56, "evidence", "verdicts agree", "or the test fails")
    f.line([(xs[1] + w // 2, low + 56), (xs[1] + w // 2, bottom + 28), (xs[3] - 23, bottom + 28)], "evidence")
    f.line([(xs[5] + w // 2, low + 56), (xs[5] + w // 2, bottom + 28), (xs[3] + w + 23, bottom + 28)], "evidence")
    f.note(xs[1] + w // 2 + 8, low + 80, "native verdict", "evidence")
    f.note(xs[5] + w // 2 - 8, low + 80, "generated verdict", "evidence", anchor="end")
    return f


def lock():
    """The version lock: every git repository pinned in Cargo.lock."""
    rows = locked_repositories()
    row_h = 34
    head = 34
    f = Figure(
        "lock",
        980,
        head + row_h * len(rows) + 8,
        "The version lock is Cargo.lock. It pins "
        + ", ".join(r for r, _, _, _ in rows)
        + " each to one exact commit, so every crate of a repository resolves to a single copy.",
    )
    cols = [0, 330, 520, 760]
    for x, s in zip(cols, ["REPOSITORY", "TRACKS", "PINNED COMMIT", "CRATES LOCKED"]):
        f.text(x + 12, 16, s, "muted", "mono", SMALL)
    for i, (repo, kind, commit, n) in enumerate(rows):
        y = head + i * row_h
        role = "ink"
        f.rect(0, y, 980, row_h - 6, role, fill=role != "ink")
        f.text(12, y + 21, repo, role, "title", TITLE)
        f.text(cols[1] + 12, y + 21, "branch main" if kind == "branch" else "fixed revision", "muted", "mono", SMALL + 1)
        f.text(cols[2] + 12, y + 21, commit, "model", "mono", SMALL + 1)
        f.text(cols[3] + 12, y + 21, str(n), "ink", "mono", SMALL + 1)
    return f


REPOS = [
    (1, 0, "quire-driver", "the quire command", True),
    (2, 0, "quire-integration", "tests the whole chain", True),
    (3, 0, "quire-verification", "plans verification", True),
    (0, 1, "quire-specification", "the standard", True),
    (1, 1, "quire-spec-language", "compiler · evaluator", False),
    (2, 1, "quire-contract-ir", "contract model", False),
    (3, 1, "quire-contract-codegen", "code · tests · proofs", False),
    (4, 1, "quire-contract-runtime", "runs generated checks", False),
    (0, 2, "quire-observation", "qualified observations", False),
    (1, 2, "quire-protocol", "protocol conformance", True),
    (2, 2, "quire-analyze", "SMT consistency", False),
    (0, 3, "quire-mltl", "MLTL over observations", False),
]
FEEDS = [
    ("quire-specification", "quire-spec-language"),
    ("quire-spec-language", "quire-contract-ir"),
    ("quire-contract-ir", "quire-contract-codegen"),
    ("quire-contract-codegen", "quire-contract-runtime"),
    ("quire-spec-language", "quire-protocol"),
    ("quire-contract-ir", "quire-analyze"),
    ("quire-observation", "quire-protocol"),
    ("quire-observation", "quire-mltl"),
]
SHARED = ["quire-eval", "quire-exact", "quire-semantic-value", "quire-walk", "quire-canonical",
          "quire-verification-contracts"]


def ecosystem(here="quire-integration"):
    """The Quire repositories: the main chain, its branches, and what runs and shares it."""
    f = Figure(
        "ecosystem",
        980,
        476,
        "The Quire repositories. The main chain runs from the standard, quire-specification, to "
        "the compiler, quire-spec-language, then quire-contract-ir, quire-contract-codegen and "
        "quire-contract-runtime. quire-spec-language also feeds quire-protocol, and "
        "quire-contract-ir feeds quire-analyze. quire-observation feeds quire-protocol and "
        "quire-mltl. Above the chain, quire-driver runs it, quire-integration tests it and "
        "quire-verification plans verification. Below it, shared libraries used throughout: "
        "quire-eval, quire-exact, quire-semantic-value, quire-walk, quire-canonical and "
        "quire-verification-contracts.",
    )
    w, h, step = 176, 50, 201
    rows = {0: 20, 1: 120, 2: 210, 3: 290}
    where = {}
    for col, row, name, sub, _ in REPOS:
        x, y = col * step, rows[row]
        where[name] = (x, y)
        role = "logic" if name == here else "ink"
        f.box(x, y, w, h, role, name, sub)
    for a, b in FEEDS:
        (ax, ay), (bx, by) = where[a], where[b]
        if ay == by:
            f.line([(ax + w, ay + h / 2), (bx - 3, by + h / 2)], "model")
        else:
            f.line([(ax + w / 2, ay + h), (bx + w / 2, by - 3)], "model")
    f.text(0, 12, "ORCHESTRATION", "muted", "mono", SMALL)
    f.text(0, 112, "THE CHAIN", "muted", "mono", SMALL)
    f.text(0, 382, "SHARED LIBRARIES · USED THROUGHOUT", "muted", "mono", SMALL)
    gap = 10
    widths = [FONTS["title"].width(name, TITLE) + 24 for name in SHARED]
    spare = (980 - sum(widths) - gap * (len(SHARED) - 1)) / len(SHARED)
    x = 0
    for name, w0 in zip(SHARED, widths):
        w0 = round(w0 + spare)
        f.box(x, 390, w0, h, "logic" if name == here else "ink", name)
        x += w0 + gap
    f.legend(0, 472, [("logic", "this repository", "box"), ("model", "feeds", "fill")])
    return f


FIGURES = [hero, lock, ecosystem]
