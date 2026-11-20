#!/usr/bin/env python3
import argparse
import datetime
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent.parent
WORKLOADS = {
    'count_generated_corpus': 'walk', 'count_fixtures': 'walk',
    'count_rust_in_memory': 'count', 'detect_and_count_rust_in_memory': 'count',
    'count_blank_heavy_in_memory': 'count', 'count_minified_in_memory': 'count',
    'count_non_ascii_in_memory': 'count', 'count_r_dense_in_memory': 'count',
    'count_json_in_memory': 'count', 'render_json_compact': 'render', 'render_human': 'render',
}


def select(names, areas, skips):
    for value, choices in ((names, WORKLOADS), (areas, set(WORKLOADS.values())), (skips, WORKLOADS)):
        unknown = set(value) - set(choices)
        if unknown:
            raise ValueError('unknown selector: ' + ', '.join(sorted(unknown)))
    selected = [name for name, area in WORKLOADS.items()
                if (not names or name in names) and (not areas or area in areas) and name not in skips]
    if not selected:
        raise ValueError('selection is empty')
    return selected


def parser():
    p = argparse.ArgumentParser()
    p.add_argument('--tier', choices=['edit', 'change', 'scenario', 'full'], default='edit')
    p.add_argument('--name', action='append', default=[])
    p.add_argument('--area', action='append', default=[])
    p.add_argument('--skip', action='append', default=[])
    p.add_argument('--dry-run', action='store_true')
    p.add_argument('--check', action='store_true')
    p.add_argument('--list', action='store_true')
    return p


def preflight(names):
    missing = [name for name in ('cargo', 'rustc', 'python3') if not shutil.which(name)]
    if missing:
        raise ValueError('install missing tools: ' + ', '.join(missing) + '; needed by ' + ', '.join(names))
    fixture_users = [name for name in names if name in ('count_fixtures', 'render_human', 'render_json_compact')]
    if fixture_users and not (ROOT / 'tests/fixtures').is_dir():
        raise ValueError('restore tests/fixtures; needed by ' + ', '.join(fixture_users))
    if shutil.disk_usage(ROOT).free < 1024 ** 3:
        raise ValueError('free at least 1 GiB on the worktree filesystem')
    subprocess.run(['cargo', '--version'], check=True)
    subprocess.run(['rustc', '--version'], check=True)
    ready = shutil.which('perf-gate-ready')
    if ready:
        status = subprocess.run([ready], check=False).returncode
        print(f'host readiness exit: {status}; timings need an independent paired comparison', flush=True)
    else:
        print('host readiness unavailable; timing validity unknown', flush=True)
    print('Cargo rebuilds the benchmark when inputs are stale.', flush=True)


def command(tier, name):
    if tier == 'edit':
        return [sys.executable, '-B', '-m', 'unittest', 'discover', '-s', 'tools', '-p', 'test_bench.py']
    if tier == 'full':
        return ['make', 'bench-baseline']
    sample_count = '2' if tier == 'change' else '10'
    return ['cargo', 'bench', '--bench', 'bench', '--', '--exact', f'bench::{name}',
            '--sample-count', sample_count, '--sample-size', '1', '--format', 'json']


def main():
    args = parser().parse_args()
    if args.list:
        print('edit: driver selection tests; change: short workload; scenario: 10 samples; full: manual only')
        for name, area in WORKLOADS.items():
            print(f'{name}\t{area}\tchange')
        return 0
    try:
        names = select(args.name, args.area, args.skip)
        if args.tier == 'edit' and (args.name or args.area or args.skip):
            raise ValueError('edit tier has no workload selectors')
        if args.tier == 'full' and (args.name or args.area or args.skip):
            raise ValueError('full baseline does not accept selectors; use scenario')
        cmds = [command(args.tier, name) for name in names] if args.tier not in ('edit', 'full') else [command(args.tier, '')]
        if args.dry_run:
            for cmd in cmds:
                print(subprocess.list2cmdline(cmd))
            return 0
        preflight(names if args.tier != 'edit' else ['edit'])
        if args.check:
            return 0
        run_dir = ROOT / '.work' / 'bench-runs' / (datetime.datetime.now().strftime('%Y%m%d-%H%M%S-%f') + f'-{os.getpid()}')
        run_dir.mkdir(parents=True)
        results = []
        for idx, cmd in enumerate(cmds):
            name = names[idx] if args.tier not in ('edit', 'full') else args.tier
            start = time.monotonic()
            with (run_dir / (name + '.log')).open('w') as log:
                result = subprocess.run(cmd, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, check=False)
            status = result.returncode
            if status == 0 and args.tier in ('change', 'scenario'):
                reports = []
                for line in (run_dir / (name + '.log')).read_text().splitlines():
                    try:
                        reports.append(json.loads(line))
                    except json.JSONDecodeError:
                        continue
                if not any(isinstance(report, dict) and any(item.get('name') == 'bench::' + name for item in report.get('benchmarks', [])) for report in reports):
                    status = 1
                    with (run_dir / (name + '.log')).open('a') as log:
                        log.write('No matching benchmark measurement found.\n')
            results.append({'name': name, 'exit': status, 'seconds': round(time.monotonic() - start, 3)})
        summary = {'tier': args.tier, 'results': results, 'excluded': [n for n in WORKLOADS if n not in names] if args.tier != 'edit' else list(WORKLOADS), 'timing': 'unqualified; no paired baseline', 'verdict': 'completion is not a performance verdict'}
        (run_dir / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
        print(json.dumps(summary, indent=2))
        print(f'logs: {run_dir}')
        return int(any(item['exit'] for item in results))
    except (ValueError, OSError, subprocess.CalledProcessError) as exc:
        print(f'bench: {exc}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    sys.exit(main())
