import json
import unittest

from bench import WORKLOADS, has_measurement, select


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


class MeasurementTest(unittest.TestCase):
    def test_selected_workload_must_complete_its_samples(self):
        name = 'count_fixtures'
        complete = {'name': 'bench::count_fixtures', 'sample_count': 2, 'iter_count': 2, 'median_ns': 100}
        for entry in ({'name': complete['name']}, {**complete, 'sample_count': 1}, {**complete, 'iter_count': 1},
                      {**complete, 'median_ns': False}, {**complete, 'name': 'bench::count_generated_corpus'}, None):
            with self.subTest(entry=entry):
                self.assertFalse(has_measurement(json.dumps({'benchmarks': [entry]}), name, 2))
        report = json.dumps({'benchmarks': [complete]})
        self.assertTrue(has_measurement(f'cargo output\n{report}\n', name, 2))


if __name__ == '__main__':
    unittest.main()
