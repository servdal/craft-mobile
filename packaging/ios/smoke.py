#!/usr/bin/env python3
"""Install, launch and capture each application in a dedicated simulator."""
import argparse,json,plistlib,subprocess,time
from pathlib import Path
root=Path(__file__).resolve().parents[2]
p=argparse.ArgumentParser();p.add_argument('device',help='Dedicated simulator UUID');p.add_argument('projects',nargs='*');p.add_argument('--label',choices=['ipad','iphone'],default='ipad');a=p.parse_args()
out=root/'dist/simulator/qa'/a.label;out.mkdir(parents=True,exist_ok=True)
apps=sorted((root/'dist/simulator').glob('*.app'));results={}
previous=out/'launch-status.json'
if previous.exists():
 saved=json.loads(previous.read_text())
 if all(a.device in item.get('process','') for item in saved.values()):results=saved
if not apps:raise SystemExit('Build simulator applications before running startup QA')
for app in apps:
 info=plistlib.loads((app/'Info.plist').read_bytes());bundle=info['CFBundleIdentifier'];project=bundle.rsplit('.',1)[-1]
 if a.projects and project not in a.projects:continue
 import hashlib
 executable=app/info['CFBundleExecutable']
 r={'bundle':bundle,'binary_sha256':hashlib.file_digest(executable.open('rb'),'sha256').hexdigest()};results[project]=r
 try:
  subprocess.run(['xcrun','simctl','install',a.device,str(app)],check=True,capture_output=True,text=True)
  launch=subprocess.run(['xcrun','simctl','launch','--terminate-running-process',a.device,bundle],check=True,capture_output=True,text=True)
  r['launch']=launch.stdout.strip();time.sleep(3)
  pid=launch.stdout.strip().rsplit(':',1)[-1].strip()
  processes=subprocess.run(['ps','-p',pid,'-o','comm='],capture_output=True,text=True)
  r['running']=processes.returncode==0 and app.name in processes.stdout and a.device in processes.stdout;r['process']=processes.stdout.strip()
  if r['running']:
   shot=out/(project+'.png');subprocess.run(['xcrun','simctl','io',a.device,'screenshot',str(shot)],check=True,capture_output=True,text=True);r['screenshot']=str(shot)
   subprocess.run(['xcrun','simctl','terminate',a.device,bundle],check=False,capture_output=True)
 except subprocess.CalledProcessError as e:r['error']=str(e.stderr or e);r['running']=False
 (out/'launch-status.json').write_text(json.dumps(results,indent=2)+'\n');print(project,r,flush=True)
raise SystemExit(1 if any(not r.get('running') for r in results.values()) else 0)
