"""security_policy self-test (temporary repositories and bare remotes only)."""
import unittest

from skim_search.diagnostics import security_policy


class SecurityPolicyTests(unittest.TestCase):
    def test_self_test_cases_pass(self):
        self.assertEqual(security_policy.self_test(), 0)


if __name__ == "__main__":
    unittest.main(verbosity=2)
