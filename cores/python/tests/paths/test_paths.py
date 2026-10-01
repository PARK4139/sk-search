"""Path SSOT (build_env/paths-ssot): generated Python constants match cores/common/paths.ini.

The Rust side checks the same file (common::paths::tests::constants_match_paths_ini), so both
languages resolve every key to the same relative path.
"""
import ast
import os
import unittest

import skim_search
from skim_search import gen_paths


class PathsTests(unittest.TestCase):
    def setUp(self):
        self.ini = gen_paths.find_ini()
        self.entries = gen_paths.parse(self.ini.read_text(encoding="utf-8"))

    def test_generated_region_is_up_to_date(self):
        self.assertEqual(gen_paths.main(["--check"]), 0)

    def test_constants_match_paths_ini(self):
        self.assertEqual(skim_search.REL, {k: v for _, k, v in self.entries})
        self.assertTrue(os.path.samefile(skim_search.PATHS_INI, self.ini))
        for section, key, value in self.entries:
            if section == "repo":
                self.assertEqual(getattr(skim_search, key), skim_search.ROOT + "/" + value)
        tp = skim_search.THIRD_PARTY
        self.assertTrue(os.path.isfile(skim_search.RG))
        self.assertEqual(skim_search.SK, tp + "/" + skim_search.REL["SK"])

    def test_values_are_relative_and_generated_code_is_import_free(self):
        self.assertTrue(all(":" not in v and not v.startswith("/") and "\\" not in v for _, _, v in self.entries))
        source = (gen_paths.INIT).read_text(encoding="utf-8")
        region = source[source.index(gen_paths.BEGIN):source.index(gen_paths.END)]
        tree = ast.parse(region)
        top_imports = [n for n in tree.body if isinstance(n, (ast.Import, ast.ImportFrom))]
        self.assertEqual(top_imports, [])  # only the lazy third-party lookup imports os, inside __getattr__
        self.assertNotIn(skim_search.ROOT, region)  # nothing absolute is generated

    def test_key_limit_warning(self):
        many = "[repo]\n" + "".join(f"K{i}=d/{i}\n" for i in range(gen_paths.MAX_KEYS + 1))
        entries = gen_paths.parse(many)
        self.assertGreater(len(entries), gen_paths.MAX_KEYS)

    def test_invalid_values_rejected(self):
        for bad in ("[repo]\nA=C:/abs\n", "[repo]\nA=/abs\n", "[repo]\nA=a\\b\n", "[repo]\nA=a\nA=b\n", "A=a\n"):
            with self.assertRaises(ValueError):
                gen_paths.parse(bad)


if __name__ == "__main__":
    unittest.main(verbosity=2)
