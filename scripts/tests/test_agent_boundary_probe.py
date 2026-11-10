"""Offline tests: only explicit permission failures may pass the temporary probe."""
import importlib.util
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("probe", Path(__file__).parents[1] / "agent_boundary_probe.py")
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


def response(status, message):
    return f"HTTP/2.0 {status} Status\r\nContent-Type: application/json\r\n\r\n" + json.dumps({"message": message})


class DenialTests(unittest.TestCase):
    def test_explicit_permission_denied(self):
        self.assertTrue(probe.is_api_denial(1, response(403, "Resource not accessible by integration")))

    def test_approval_disabled_only_for_approval(self):
        output = response(422, probe.APPROVAL_DISABLED)
        self.assertTrue(probe.is_api_denial(1, output, approval=True))
        self.assertFalse(probe.is_api_denial(1, output))

    def test_unrelated_http_failures_do_not_pass(self):
        for status, message in [(409, "Head branch was modified"), (422, "Validation Failed"),
                                (404, "Not Found"), (403, "API rate limit exceeded"),
                                (429, "Too Many Requests"), (500, "Internal Server Error")]:
            with self.subTest(status=status):
                self.assertFalse(probe.is_api_denial(1, response(status, message), approval=True))

    def test_success_is_not_denial(self):
        self.assertFalse(probe.is_api_denial(0, response(200, "ok")))
        self.assertFalse(probe.is_api_denial(0, response(403, "Resource not accessible by integration")))

    def test_transport_or_malformed_output_does_not_pass(self):
        for output in ["", "gh: connection refused", "HTTP/2.0 403 Forbidden\n", "HTTP/2.0 403 Forbidden\n\nnot json"]:
            self.assertFalse(probe.is_api_denial(1, output))

    def test_push_explicit_denial(self):
        self.assertTrue(probe.is_push_denial(128, "remote: Write access to repository not granted.\nfatal: The requested URL returned error: 403"))

    def test_push_transport_or_auth_failures_do_not_pass(self):
        for output in ["Could not resolve host: github.com", "SSL certificate problem", "The requested URL returned error: 403", "Authentication failed"]:
            self.assertFalse(probe.is_push_denial(128, output))
        self.assertFalse(probe.is_push_denial(0, "remote: Write access to repository not granted. The requested URL returned error: 403"))


if __name__ == "__main__":
    unittest.main()
