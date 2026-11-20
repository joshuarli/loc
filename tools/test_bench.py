import unittest

from bench import WORKLOADS, select


class SelectionTest(unittest.TestCase):
    def test_name_area_and_skip(self):
        self.assertEqual(select(['count_fixtures'], ['walk'], []), ['count_fixtures'])
        self.assertNotIn('count_fixtures', select([], ['walk'], ['count_fixtures']))
        self.assertEqual(set(select([], [], [])), set(WORKLOADS))

    def test_unknown_and_empty_fail(self):
        for args in [(['missing'], [], []), ([], ['missing'], []), ([], [], ['missing']),
                     (['count_fixtures'], ['render'], [])]:
            with self.assertRaises(ValueError):
                select(*args)


if __name__ == '__main__':
    unittest.main()
