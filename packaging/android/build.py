#!/usr/bin/env python3
"""Build signed, standalone ARM64 Android release bundles and optional installable APKs."""
from pathlib import Path
import argparse,json,os,shutil,subprocess,sys
ROOT=Path(__file__).resolve().parents[2]
NAMES={'gridcraft':'GridCraft','wordcraft':'WordCraft','deckcraft':'DeckCraft','cadcraft':'CADCraft','vectorcraft':'VectorCraft','photocraft':'PhotoCraft','pdfcraft':'PdfCraft','filmcraft':'FilmCraft','effectcraft':'EffectCraft','lightcraft':'LightCraft'}

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('projects',nargs='*',default=list(NAMES));p.add_argument('--package-only',action='store_true');p.add_argument('--native-only',action='store_true');p.add_argument('--apk',action='store_true');p.add_argument('--offline',action='store_true');p.add_argument('--key-properties',type=Path);p.add_argument('--target-dir',type=Path,default=ROOT/'build/android-rust')
    args=p.parse_args();args.target_dir=args.target_dir.resolve();unknown=set(args.projects)-NAMES.keys()
    if unknown:p.error('Unknown project: '+', '.join(sorted(unknown)))
    sdk=Path(os.environ.get('ANDROID_HOME',Path.home()/'Library/Android/sdk'))
    ndk=Path(os.environ.get('ANDROID_NDK_HOME',sdk/'ndk/28.2.13676358'))
    host='darwin-x86_64' if sys.platform=='darwin' else 'linux-x86_64'
    tools=ndk/'toolchains/llvm/prebuilt'/host/'bin'
    if not (tools/'aarch64-linux-android29-clang').is_file():p.error('Install Android NDK 28.2.13676358 and set ANDROID_NDK_HOME')
    env=os.environ.copy();env.update(ANDROID_HOME=str(sdk),ANDROID_NDK_HOME=str(ndk),CARGO_BUILD_TARGET='aarch64-linux-android',CARGO_BUILD_JOBS=env.get('CARGO_BUILD_JOBS','4'),CARGO_TARGET_DIR=str(args.target_dir.resolve()),CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=str(tools/'aarch64-linux-android29-clang'),CC_aarch64_linux_android=str(tools/'aarch64-linux-android29-clang'),CXX_aarch64_linux_android=str(tools/'aarch64-linux-android29-clang++'),AR_aarch64_linux_android=str(tools/'llvm-ar'),RUSTFLAGS='-C link-arg=-Wl,-z,max-page-size=16384')
    if args.offline:env['CARGO_NET_OFFLINE']='true'
    if args.key_properties:env['CRAFT_KEY_PROPERTIES']=str(args.key_properties.resolve())
    dist=ROOT/'dist/android';dist.mkdir(parents=True,exist_ok=True);results={}
    status=dist/'release-status.json'
    for project in args.projects:
        title=NAMES[project];jni=ROOT/'packaging/android/app/src'/project/'jniLibs/arm64-v8a/libcraft.so'
        print('Preparing',title,flush=True)
        if not args.package_only:
            with (dist/(project+'-rust.log')).open('w') as log:
                result=subprocess.run(['cargo','build','--locked','--release'],cwd=ROOT/'android-apps'/project,env=env,stdout=log,stderr=subprocess.STDOUT)
            if result.returncode:
                results[project]={'status':'failed','stage':'Rust release compilation'};status.write_text(json.dumps(results,indent=2)+'\n');continue
            jni.parent.mkdir(parents=True,exist_ok=True)
            shutil.copy2(args.target_dir/'aarch64-linux-android/release'/('lib'+project+'_android.so'),jni)
        if not jni.is_file():raise SystemExit('Native library missing for '+project+'; run without --package-only')
        if args.native_only:
            results[project]={'status':'native-built'};status.write_text(json.dumps(results,indent=2)+'\n');continue
        licenses=ROOT/'packaging/android/app/src'/project/'assets/Licenses';licenses.mkdir(parents=True,exist_ok=True)
        for name in ('LICENSE-MIT','LICENSE-APACHE','NOTICE','ATTRIBUTION.md','ASSETS.md'):
            source=ROOT/project/name
            if source.is_file():shutil.copy2(source,licenses/name)
        shutil.copy2(ROOT/'assets/launcher/README.md',licenses/'MASCOT.md')
        task=project[0].upper()+project[1:]
        command=['./gradlew','--no-daemon','--console=plain',':app:bundle'+task+'Release']
        if args.apk:command+=[':app:assemble'+task+'Release']
        if args.offline:command+=['--offline']
        with (dist/(project+'-gradle.log')).open('w') as log:
            result=subprocess.run(command,cwd=ROOT/'packaging/android',env=env,stdout=log,stderr=subprocess.STDOUT)
        if result.returncode:
            results[project]={'status':'failed','stage':'Android packaging/signing'};status.write_text(json.dumps(results,indent=2)+'\n');continue
        source=ROOT/'packaging/android/app/build/outputs/bundle'/(project+'Release')/('app-'+project+'-release.aab')
        aab=dist/(title+'-release.aab');shutil.copy2(source,aab)
        results[project]={'status':'built','aab':str(aab),'application_id':'com.duidev.craftmobile.'+project,'abi':'arm64-v8a','min_sdk':29,'target_sdk':36}
        if args.apk:
            apk=dist/(title+'-release.apk');shutil.copy2(ROOT/'packaging/android/app/build/outputs/apk'/project/'release'/('app-'+project+'-release.apk'),apk);results[project]['apk']=str(apk)
        status.write_text(json.dumps(results,indent=2)+'\n');print('BUILT',aab,flush=True)
    return 1 if any(x['status']=='failed' for x in results.values()) else 0
if __name__=='__main__':raise SystemExit(main())
