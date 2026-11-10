"""Offline regression tests for the literal Bash routing step in claude.yml.

These tests stub GitHub's API. They do not validate GitHub event delivery or the
Actions expression engine, and they do not exercise Claude or any credentials.
"""

import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = (ROOT / ".github/workflows/claude.yml").read_text()
ROUTE_JOB = WORKFLOW.split("\n  route:\n", 1)[1].split("\n  claude:\n", 1)[0]
RUN_BLOCK = re.search(r"(?m)^        run: \|\n((?:^          .*\n|^\n)+)", ROUTE_JOB)
if RUN_BLOCK is None:
    raise AssertionError("Expected one literal routing run block in claude.yml")
ROUTE_SCRIPT = "\n".join(line[10:] for line in RUN_BLOCK.group(1).splitlines())


class ClaudeWorkflowRoutingTests(unittest.TestCase):
    def route(self, *, actor="FunKite", event="issue_comment", pr="122",
              head="FunKite/solunatus", calls=1,
              at="2026-11-10T02:00:00Z", fail="", partial=False):
        with tempfile.TemporaryDirectory(prefix="claude-route-test-") as temp:
            root = Path(temp)
            gh = root / "gh"
            gh.write_text("""#!/bin/sh
fail_if_requested() {
  if [ "$TEST_FAIL" = "$1" ]; then
    [ "$TEST_PARTIAL" != 1 ] || echo 1
    exit 99
  fi
}
case "$*" in
  *"/pulls/122 --jq"*)
    fail_if_requested head
    printf '%s\\n' "$TEST_HEAD"
    ;;
  *"/issues/122/comments"*)
    fail_if_requested issue_comments
    i=0
    while [ "$i" -lt "$TEST_CALLS" ]; do
      echo 1
      i=$((i + 1))
    done
    ;;
  *"/pulls/122/comments"*) fail_if_requested review_comments ;;
  *"/pulls/122/reviews"*) fail_if_requested reviews ;;
  *) echo "Unexpected API route: $*" >&2; exit 99 ;;
esac
""")
            gh.chmod(0o700)
            output = root / "output"
            # Deliberately do not inherit any credentials from the test process.
            env = {
                "PATH": str(root) + os.pathsep + "/usr/bin:/bin",
                "REPO": "FunKite/solunatus",
                "ACTOR": actor,
                "EVENT_NAME": event,
                "CODEX": "chatgpt-codex-connector[bot]",
                "CODEX_CALL_LIMIT": "3",
                "EVENT_AT": at,
                "PR_NUMBER": pr,
                "TEST_HEAD": head,
                "TEST_CALLS": str(calls),
                "TEST_FAIL": fail,
                "TEST_PARTIAL": "1" if partial else "0",
                "GITHUB_OUTPUT": str(output),
            }
            result = subprocess.run(
                ["/bin/bash", "-c", ROUTE_SCRIPT], env=env,
                text=True, capture_output=True, timeout=5,
            )
            mode = output.read_text() if output.exists() else ""
            return result, mode

    def assert_mode(self, expected, **kwargs):
        result, output = self.route(**kwargs)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(output, "mode=" + expected + "\n", result.stdout)

    def test_owner_same_repository_and_issue_remain_full(self):
        self.assert_mode("full")
        self.assert_mode("full", pr="", event="issues")

    def test_all_fork_and_deleted_fork_comment_events_skip(self):
        for head in ("someone/solunatus", ""):
            for event in ("issue_comment", "pull_request_review",
                          "pull_request_review_comment"):
                with self.subTest(head=head, event=event):
                    self.assert_mode("skip", head=head, event=event)

    def test_codex_same_repository_limit_remains_enforced(self):
        actor = "chatgpt-codex-connector[bot]"
        for calls, expected in ((1, "full"), (3, "full"), (4, "skip")):
            with self.subTest(calls=calls):
                self.assert_mode(expected, actor=actor, calls=calls)

    def test_codex_requires_same_repository_pr_and_event_time(self):
        actor = "chatgpt-codex-connector[bot]"
        for kwargs in ({"head": "someone/solunatus"}, {"head": ""},
                       {"pr": ""}, {"at": ""}):
            with self.subTest(**kwargs):
                self.assert_mode("skip", actor=actor, **kwargs)

    def test_head_lookup_failure_never_enables_claude(self):
        for actor in ("FunKite", "chatgpt-codex-connector[bot]"):
            with self.subTest(actor=actor):
                result, output = self.route(actor=actor, fail="head")
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(output, "")

    def test_codex_count_api_failures_never_enable_claude(self):
        for endpoint in ("issue_comments", "review_comments", "reviews"):
            for partial in (False, True):
                with self.subTest(endpoint=endpoint, partial=partial):
                    result, output = self.route(
                        actor="chatgpt-codex-connector[bot]", calls=3,
                        fail=endpoint, partial=partial,
                    )
                    self.assertNotEqual(result.returncode, 0)
                    self.assertEqual(output, "")

    def test_fork_job_and_shared_bot_comment_trust_are_absent(self):
        self.assertNotIn("\n  fork-review:", WORKFLOW)
        self.assertNotIn("mode=review-only", ROUTE_SCRIPT)
        self.assertIn("if: needs.route.outputs.mode == 'full'", WORKFLOW)
        actor_lists = re.findall(r'^\s+include_comments_by_actor: "([^"]+)"$',
                                 WORKFLOW, re.MULTILINE)
        self.assertEqual(len(actor_lists), 1)
        self.assertEqual(actor_lists[0].split(","),
                         ["FunKite", "chatgpt-codex-connector[bot]", "claude[bot]"])


if __name__ == "__main__":
    unittest.main()
