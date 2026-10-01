"""Runtime evidence for diagnostics/one-shot-agent-classification/f88f1527.

Uses a disposable Git fixture; never commits or publishes the user's workspace.
Run from outside the repository using uv run --project <repo>/cores/python.
"""
from pathlib import Path
import time

from skim_search.diagnostics.one_shot import pipeline as p
from tests.one_shot.test_pipeline import PipelineTests


def main():
    fixture = PipelineTests()
    fixture.setUp()
    started = time.monotonic()
    logs = p.ROOT / 'ref/actual/logs/one-shot' / ('agent-live-' + p.uuid.uuid4().hex[:8])
    try:
        fixture.change()
        fixture.git('add', '.')
        fixture.git('commit', '-m', 'Add fixture text data')
        cfg = dict(fixture.cfg, codex=['codex'])
        run = p.Run(cfg, logs)
        target = fixture.git('rev-parse', 'HEAD')
        run.state['sha'] = target
        p.assign_version(run, None)
        data = run.state['assignment']
        assert data['level'] in p.LEVELS and data['decision_by'] == 'agent', data
        assert data['version'] == p.bump_version(cfg['initial_version'], data['level']), data
        before = run.state['assignment'].copy()
        p.assign_version(run, None)
        assert run.state['assignment'] == before
        p.write_json(logs / 'verification.json', {'result': 'PASS', 'classification': data,
                     'seconds': time.monotonic() - started, 'version': before['version'],
                     'scope': 'real CLI automatic classification and assignment, SHA reuse; disposable repository'})
        print('PASS real Codex classification; logs:', logs)
    finally:
        fixture.tearDown()


if __name__ == '__main__':
    main()
