#!/usr/bin/env python3
"""Temporary, never-merge probe for the review job's GitHub permissions."""

import base64
import json
import os
import re
import subprocess
import tempfile
from pathlib import Path

PERMISSION_DENIALS = {
    "Resource not accessible by integration",
    "Resource not accessible by personal access token",
}
APPROVAL_DISABLED = {
    "GitHub Actions is not permitted to create or approve pull requests",
    "GitHub Actions is not permitted to approve pull requests",
}


def api_response(output):
    """Read gh api --include output without treating stderr as an HTTP response."""
    parts = re.split(r"\r?\n\r?\n", output, maxsplit=1)
    first_line = parts[0].splitlines()[0] if parts[0] else ""
    match = re.fullmatch(r"HTTP/\S+ (\d{3})(?: .*)?", first_line)
    if not match or len(parts) != 2:
        return None, {}
    try:
        body = json.loads(parts[1])
    except json.JSONDecodeError:
        return int(match[1]), {}
    return int(match[1]), body if isinstance(body, dict) else {}


def is_api_denial(returncode, output, *, approval=False):
    status, body = api_response(output)
    message = body.get("message", "")
    if returncode == 0:
        return False
    if status == 403 and message in PERMISSION_DENIALS:
        return True
    if not approval or status != 422:
        return False
    if message.rstrip(".") in APPROVAL_DISABLED:
        return True
    errors = body.get("errors", [])
    # GitHub also wraps the policy denial in a Validation Failed response.
    return message == "Validation Failed" and bool(errors) and all(
        isinstance(error, dict)
        and error.get("resource") == "PullRequestReview"
        and error.get("message", "").rstrip(".") in APPROVAL_DISABLED
        for error in errors
    )


def is_push_denial(returncode, output):
    if returncode == 0:
        return False
    return (
        "The requested URL returned error: 403" in output
        and (
            "Write access to repository not granted" in output
            or re.search(r"Permission to [^\n]+ denied to github-actions\[bot\]", output)
        )
    )


def expect_api_denied(name, args, *, approval=False):
    result = subprocess.run(["gh", "api", "--include", *args], capture_output=True, text=True)
    status, body = api_response(result.stdout)
    if not is_api_denial(result.returncode, result.stdout, approval=approval):
        # Never print complete command output: a transport failure could contain credentials.
        raise RuntimeError(
            f"FAIL: {name}: expected an explicit permission denial; "
            f"exit={result.returncode}, HTTP={status}, "
            f"API message={body.get('message', '')!r}, errors={body.get('errors', [])!r}. "
            "Stop and inspect before retrying; a successful operation may need cleanup."
        )
    print(f"OK: {name}: explicit permission denial (HTTP {status})", flush=True)


def gh_api(*args):
    return subprocess.check_output(["gh", "api", *args], text=True)


def main():
    repo = os.environ["REPO"]
    pr = os.environ["PR"]
    head = os.environ["HEAD_SHA"]
    branch = "codex/claude-boundary-test"
    if repo != "FunKite/solunatus" or os.environ.get("HEAD_REF") != branch or pr != "122":
        raise RuntimeError("Refusing to probe anything except the authorized temporary PR #122")

    expect_api_denied("commit through the contents API", [
        "-X", "PUT", f"repos/{repo}/contents/boundary-probe.txt",
        "-f", "message=probe", "-f", "content=cHJvYmUK", "-f", f"branch={branch}",
    ])
    expect_api_denied("create a branch", [
        "-X", "POST", f"repos/{repo}/git/refs",
        "-f", "ref=refs/heads/boundary-probe-should-not-exist", "-f", f"sha={head}",
    ])
    # A deliberately wrong SHA prevents an allowed merge. HTTP 409 is a FAILURE,
    # not proof of denied permission. Never remove this safety interlock.
    expect_api_denied("merge", [
        "-X", "PUT", f"repos/{repo}/pulls/{pr}/merge", "-f", f"sha={'0' * 40}",
    ])
    expect_api_denied("approve", [
        "-X", "POST", f"repos/{repo}/pulls/{pr}/reviews",
        "-f", "event=APPROVE", "-f", "body=Boundary probe approval test",
    ], approval=True)

    with tempfile.TemporaryDirectory(prefix="boundary-probe-") as work:
        subprocess.run(["git", "init", "-q", work], check=True)
        subprocess.run([
            "git", "-C", work, "-c", "user.name=probe", "-c", "user.email=probe@invalid",
            "commit", "-q", "--allow-empty", "-m", "probe",
        ], check=True)
        env = os.environ.copy()
        auth = base64.b64encode(f"x-access-token:{env['GH_TOKEN']}".encode()).decode()
        env.update({
            "GIT_CONFIG_COUNT": "2",
            "GIT_CONFIG_KEY_0": "http.https://github.com/.extraheader",
            "GIT_CONFIG_VALUE_0": f"AUTHORIZATION: basic {auth}",
            "GIT_CONFIG_KEY_1": "credential.helper",
            "GIT_CONFIG_VALUE_1": "",
            "GIT_TERMINAL_PROMPT": "0",
        })
        result = subprocess.run([
            "git", "-C", work, "push", f"https://github.com/{repo}.git",
            "HEAD:refs/heads/boundary-probe-should-not-exist",
        ], env=env, capture_output=True, text=True)
        if not is_push_denial(result.returncode, result.stdout + result.stderr):
            raise RuntimeError("FAIL: git push did not produce an explicit write-permission denial; inspect before retrying")
        print("OK: git push: explicit write-permission denial (HTTP 403)", flush=True)

    # The reviewer must still be able to comment. Clean up each temporary comment
    # in a finally block, and fail the job if cleanup fails.
    for kind, endpoint, fields in [
        ("summary", f"repos/{repo}/issues/{pr}/comments", []),
        ("inline", f"repos/{repo}/pulls/{pr}/comments", [
            "-f", f"commit_id={head}", "-f", "path=docs/agent-boundary-test.md",
            "-F", "line=3", "-f", "side=RIGHT",
        ]),
    ]:
        comment = json.loads(gh_api("-X", "POST", endpoint,
            "-f", f"body=Boundary probe: {kind} comment test. Temporary.", *fields))
        try:
            if comment["user"]["login"] != "github-actions[bot]":
                raise RuntimeError(f"Unexpected {kind} comment author")
            print(f"OK: {kind} comment posted as github-actions[bot]", flush=True)
        finally:
            resource = "issues" if kind == "summary" else "pulls"
            gh_api("-X", "DELETE", f"repos/{repo}/{resource}/comments/{comment['id']}")
            print(f"Cleaned up {kind} comment", flush=True)


if __name__ == "__main__":
    main()
