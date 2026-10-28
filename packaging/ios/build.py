#!/usr/bin/env python3
"""Build Rust native iOS app bundles; simulator bundles are ad-hoc signed."""
from pathlib import Path
import argparse,os,subprocess,plistlib,json,shutil
root=Path(__file__).resolve().parents[2]
a=argparse.ArgumentParser();a.add_argument('projects',nargs='*',default=['gridcraft','wordcraft','deckcraft','cadcraft','vectorcraft','photocraft','pdfcraft','filmcraft','effectcraft','lightcraft']);a.add_argument('--package-only',action='store_true',help='Package existing binaries without compiling');a.add_argument('--device',action='store_true');a.add_argument('--release',action='store_true');a.add_argument('--target-dir',type=Path,default=root/'build/rust');args=a.parse_args()
target='aarch64-apple-ios' if args.device else 'aarch64-apple-ios-sim';profile='release' if args.release else 'debug';dist=root/'dist'/('device-unsigned' if args.device else 'simulator');dist.mkdir(parents=True,exist_ok=True)
names={'photocraft':'PhotoCraft','vectorcraft':'VectorCraft','pdfcraft':'PdfCraft','wordcraft':'WordCraft','gridcraft':'GridCraft','deckcraft':'DeckCraft','cadcraft':'CADCraft','filmcraft':'FilmCraft','effectcraft':'EffectCraft','lightcraft':'LightCraft'}
e=os.environ.copy();e.update(CARGO_TARGET_DIR=str(args.target_dir.resolve()),CARGO_BUILD_JOBS=e.get('CARGO_BUILD_JOBS','2'),CARGO_PROFILE_DEV_DEBUG='0',IPHONEOS_DEPLOYMENT_TARGET='16.0')
results={}
for project in args.projects:
 if project not in names:raise SystemExit('Unknown project: '+project)
 cmd=['cargo','build','--locked','--target',target,'-p',project+'-ios'];cmd+=['--release'] if args.release else []
 if not args.package_only:
  with (dist/('build-'+project+'.log')).open('w') as log:
   print('Building',project,flush=True);r=subprocess.run(cmd,cwd=root/project,env=e,stdout=log,stderr=subprocess.STDOUT)
  if r.returncode:results[project]={'status':'failed','log':'build-'+project+'.log'};print('FAILED',project,flush=True);continue
 title=names[project];app=dist/(title+'.app');app.mkdir(parents=True,exist_ok=True)
 executable=app/title;shutil.copy2(args.target_dir/target/profile/(project+'-ios'),executable);executable.chmod(0o755)
 if b'CraftSceneDelegate\0' not in executable.read_bytes():
  results[project]={'status':'failed','reason':'Scene lifecycle adapter is missing; rebuild after updating mobile-shell'};print('FAILED lifecycle adapter',project,flush=True);continue
 info={'CFBundleExecutable':title,'CFBundleIdentifier':'com.duidev.craftmobile.'+project,'CFBundleName':title,'CFBundleDisplayName':title,'CFBundlePackageType':'APPL','CFBundleVersion':'7','CFBundleShortVersionString':'0.1.5','LSRequiresIPhoneOS':True,'MinimumOSVersion':'16.0','UIDeviceFamily':[1,2],'UISupportedInterfaceOrientations':['UIInterfaceOrientationPortrait','UIInterfaceOrientationLandscapeLeft','UIInterfaceOrientationLandscapeRight'],'UISupportedInterfaceOrientations~ipad':['UIInterfaceOrientationPortrait','UIInterfaceOrientationPortraitUpsideDown','UIInterfaceOrientationLandscapeLeft','UIInterfaceOrientationLandscapeRight'],'UILaunchScreen':{},'UIFileSharingEnabled':True,'LSSupportsOpeningDocumentsInPlace':True}
 info['NSPhotoLibraryAddUsageDescription']=title+' menyimpan gambar yang Anda pilih ke galeri Photos.'
 info['UIApplicationSceneManifest']={'UIApplicationSupportsMultipleScenes':False,'UISceneConfigurations':{'UIWindowSceneSessionRoleApplication':[{'UISceneConfigurationName':'Craft Mobile','UISceneDelegateClassName':'CraftSceneDelegate'}]}}
 with (app/'Info.plist').open('wb') as f:plistlib.dump(info,f)
 # Build platform-native asset catalogs, including every required iPhone/iPad icon size.
 catalog=root/'packaging/ios/Assets'/(project+'.xcassets')
 partial=dist/(project+'-icons.plist')
 subprocess.run(['xcrun','actool','--compile',str(app),'--platform','iphoneos' if args.device else 'iphonesimulator','--minimum-deployment-target','16.0','--app-icon','AppIcon','--output-partial-info-plist',str(partial),'--target-device','iphone','--target-device','ipad','--product-type','com.apple.product-type.application',str(catalog)],check=True,stdout=subprocess.DEVNULL)
 with partial.open('rb') as f:info.update(plistlib.load(f))
 with (app/'Info.plist').open('wb') as f:plistlib.dump(info,f)
 # Bundle source notices; do not add system fonts or model/runtime assets.
 licenses=app/'Licenses';licenses.mkdir(exist_ok=True)
 for name in ['LICENSE-MIT','LICENSE-APACHE','NOTICE','ATTRIBUTION.md','ASSETS.md']:
  path=root/project/name
  if path.is_file():shutil.copy2(path,licenses/name)
 if not args.device:
  subprocess.run(['codesign','--force','--sign','-','--timestamp=none',str(app)],check=True)
  subprocess.run(['codesign','--verify','--strict',str(app)],check=True)
 results[project]={'status':'packaged' if args.package_only else 'built','app':str(app),'target':target,'device_installable':False if args.device else None};print('PACKAGED' if args.package_only else 'BUILT',app,flush=True)
 (dist/'build-status.json').write_text(json.dumps(results,indent=2)+'\n')
(dist/'build-status.json').write_text(json.dumps(results,indent=2)+'\n')
raise SystemExit(1 if any(v['status']=='failed' for v in results.values()) else 0)
