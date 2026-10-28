#!/usr/bin/env python3
"""Sign native device builds with an existing Apple profile, then install over USB."""
from pathlib import Path
import argparse
import datetime
import fnmatch
import hashlib
import json
import plistlib
import re
import shutil
import subprocess
import tempfile
import time
import zipfile

ROOT = Path(__file__).resolve().parents[2]
NAMES = {'gridcraft': 'GridCraft', 'wordcraft': 'WordCraft', 'deckcraft': 'DeckCraft',
         'cadcraft': 'CADCraft', 'vectorcraft': 'VectorCraft', 'photocraft': 'PhotoCraft',
         'pdfcraft': 'PdfCraft', 'filmcraft': 'FilmCraft', 'effectcraft': 'EffectCraft',
         'lightcraft': 'LightCraft'}


def run(arguments, log=None):
    result = subprocess.run(arguments, capture_output=True, text=True)
    if log:
        log.write(result.stdout + result.stderr)
        log.flush()
    if result.returncode:
        raise RuntimeError(f'{arguments[0]} failed: {result.stderr or result.stdout}')
    return result.stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--device', required=True, help='Connected iPhone/iPad UDID')
    parser.add_argument('--profile', required=True, type=Path, help='Apple .mobileprovision file')
    parser.add_argument('--identity', help='Optional certificate SHA-1 from security find-identity')
    parser.add_argument('--wait', action='store_true', help='Wait for build.py --device outputs')
    parser.add_argument('--wait-timeout', type=int, default=7200)
    parser.add_argument('projects', nargs='*', default=list(NAMES))
    args = parser.parse_args()
    profile = plistlib.loads(subprocess.check_output(['security', 'cms', '-D', '-i', str(args.profile)]))
    expires = profile['ExpirationDate']
    if expires <= datetime.datetime.now(datetime.UTC).replace(tzinfo=None):
        raise SystemExit('Provisioning profile has expired')
    if args.device not in profile.get('ProvisionedDevices', []) and not profile.get('ProvisionsAllDevices'):
        raise SystemExit('The selected profile does not authorize this device')
    if 'iOS' not in profile.get('Platform', []):
        raise SystemExit('An iOS provisioning profile is required')
    allowed = {hashlib.sha1(certificate).hexdigest().upper() for certificate in profile['DeveloperCertificates']}
    identities = run(['security', 'find-identity', '-v', '-p', 'codesigning'])
    available = set(re.findall(r'\b[0-9A-F]{40}\b', identities))
    matching = sorted(allowed & available)
    identity = args.identity.upper() if args.identity else (matching[0] if matching else None)
    if not identity or identity not in matching:
        raise SystemExit('No valid local signing identity matches this profile')
    entitlements = profile['Entitlements']
    team = profile['TeamIdentifier'][0]
    pattern = entitlements['application-identifier']
    destination = ROOT / 'dist/device'
    destination.mkdir(parents=True, exist_ok=True)
    sdk_version = run(['xcrun', '--sdk', 'iphoneos', '--show-sdk-version']).strip()
    results = {}
    deadline = time.monotonic() + args.wait_timeout
    for project in args.projects:
        if project not in NAMES:
            raise SystemExit(f'Unknown project: {project}')
        title = NAMES[project]
        source = ROOT / 'dist/device-unsigned' / (title + '.app')
        status_file = source.parent / 'build-status.json'
        print(f'Preparing {title}', flush=True)
        try:
            while True:
                try:
                    status = json.loads(status_file.read_text()).get(project, {})
                except (OSError, json.JSONDecodeError):
                    status = {}
                if status.get('status') == 'failed':
                    raise RuntimeError('Device compilation failed; inspect the device build log')
                if source.is_dir() and status.get('status') in ('built', 'packaged'):
                    break
                if not args.wait or time.monotonic() >= deadline:
                    raise RuntimeError('Device build is not available')
                time.sleep(2)
            info = plistlib.loads((source / 'Info.plist').read_bytes())
            bundle = info['CFBundleIdentifier']
            application_id = team + '.' + bundle
            if not fnmatch.fnmatchcase(application_id, pattern):
                raise RuntimeError('Provisioning profile does not authorize this bundle identifier')
            with (destination / (project + '-deploy.log')).open('w') as log:
                platform = run(['xcrun', 'vtool', '-show-build', str(source / info['CFBundleExecutable'])], log)
                if not re.search(r'platform\s+IOS\s', platform):
                    raise RuntimeError('A physical iOS binary is required; Simulator bundles cannot be installed')
                with tempfile.TemporaryDirectory(prefix='sign-', dir=destination) as temporary:
                    stage = Path(temporary) / (title + '.app')
                    shutil.copytree(source, stage)
                    info.update(CFBundleSupportedPlatforms=['iPhoneOS'], CFBundleInfoDictionaryVersion='6.0',
                                DTPlatformName='iphoneos', DTSDKName='iphoneos' + sdk_version,
                                DTPlatformVersion=sdk_version)
                    (stage / 'Info.plist').write_bytes(plistlib.dumps(info))
                    shutil.copy2(args.profile, stage / 'embedded.mobileprovision')
                    template = ROOT / project / 'packaging/ios/entitlements.plist'
                    signed_entitlements = plistlib.loads(template.read_bytes())
                    substitutions = {'$(AppIdentifierPrefix)': team+'.', '$(CFBundleIdentifier)': bundle, '$(DEVELOPMENT_TEAM)': team}
                    def expand(value):
                        if isinstance(value, str):
                            for key, replacement in substitutions.items(): value=value.replace(key,replacement)
                            return value
                        if isinstance(value, list): return [expand(item) for item in value]
                        return value
                    signed_entitlements = {key:expand(value) for key,value in signed_entitlements.items()}
                    signed_entitlements['get-task-allow'] = bool(entitlements.get('get-task-allow',False))
                    for key in signed_entitlements:
                        if key not in entitlements: raise RuntimeError('Entitlement not authorized by selected profile: '+key)
                    (destination/(project+'-entitlements.plist')).write_bytes(plistlib.dumps(signed_entitlements))
                    entitlement_file = Path(temporary) / 'entitlements.plist'
                    entitlement_file.write_bytes(plistlib.dumps(signed_entitlements))
                    run(['codesign', '--force', '--sign', identity, '--timestamp=none',
                         '--generate-entitlement-der', '--entitlements', str(entitlement_file), str(stage)], log)
                    run(['codesign', '--verify', '--strict', str(stage)], log)
                    final = destination / stage.name
                    if final.exists():
                        shutil.rmtree(final)
                    shutil.move(str(stage), final)
                ipa = destination / (title + '.ipa')
                with zipfile.ZipFile(ipa, 'w', zipfile.ZIP_DEFLATED) as archive:
                    for path in final.rglob('*'):
                        if path.is_file():
                            archive.write(path, Path('Payload') / final.name / path.relative_to(final))
                install_report = destination / (project + '-install.json')
                run(['xcrun', 'devicectl', 'device', 'install', 'app', '--device', args.device,
                     '--timeout', '120', '--json-output', str(install_report), str(final)], log)
                verification = destination / (project + '-installed.json')
                run(['xcrun', 'devicectl', 'device', 'info', 'apps', '--device', args.device,
                     '--bundle-id', bundle, '--json-output', str(verification)], log)
                installed = json.loads(verification.read_text()).get('result', {}).get('apps', [])
                if not any(item.get('bundleIdentifier') == bundle for item in installed):
                    raise RuntimeError('Install completed but application presence could not be confirmed')
            results[project] = {'status': 'installed', 'bundle': bundle, 'app': str(final), 'ipa': str(ipa)}
            print(f'INSTALLED {title}', flush=True)
        except (OSError, ValueError, RuntimeError, subprocess.CalledProcessError) as error:
            results[project] = {'status': 'failed', 'error': str(error)}
            print(f'FAILED {title}: {error}', flush=True)
        (destination / 'deployment-status.json').write_text(json.dumps(results, indent=2) + '\n')
    return 1 if any(item['status'] == 'failed' for item in results.values()) else 0


if __name__ == '__main__':
    raise SystemExit(main())
