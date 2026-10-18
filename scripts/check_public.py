"""Exact-byte public checks for the Windows-only offline CLI experiment."""
from pathlib import Path
import argparse
import base64
import json
import os
import subprocess
import tempfile


def matches(result, case):
    return (result.returncode==case['exit_code']
            and result.stdout==base64.b64decode(case['stdout_b64'],validate=True)
            and result.stderr==base64.b64decode(case['stderr_b64'],validate=True))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--rust',type=Path)
    parser.add_argument('--go',type=Path)
    args=parser.parse_args()
    if os.name!='nt':
        parser.error('These vectors describe the pinned Windows executable, not cross-platform equivalence.')
    binaries={name:path.resolve(strict=True) for name,path in vars(args).items() if path is not None}
    if not binaries:
        parser.error('Provide --rust and/or --go paths to built executables.')
    document=json.loads((Path(__file__).resolve().parents[1]/'evidence/public-observations.json').read_text(encoding='utf-8'))
    cases=document['observations']
    assert cases and len({c['id'] for c in cases})==len(cases),'Duplicate or empty case list'
    success=True
    with tempfile.TemporaryDirectory(prefix='mote-xp-public-') as temp:
        cwd=Path(temp)
        for name,text in document['fixtures'].items():
            target=(cwd/name).resolve()
            assert target.is_relative_to(cwd) and target!=cwd,'Unsafe fixture path'
            target.parent.mkdir(parents=True,exist_ok=True)
            target.write_text(text,encoding='utf-8')
        for language,binary in binaries.items():
            passed=0
            for case in cases:
                result=subprocess.run([str(binary),*case['argv']],cwd=cwd,capture_output=True,timeout=12)
                if matches(result,case):
                    passed+=1
                else:
                    print('MISMATCH',language,case['id'])
                    success=False
            print(f'{language}: {passed}/{len(cases)} exact public comparisons passed')
    raise SystemExit(0 if success else 1)


if __name__=='__main__':
    main()
