#!/usr/bin/env python3
"""Validate signed Android release artifacts without accessing private signing material."""
from pathlib import Path
import argparse,hashlib,json,os,re,struct,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[2]
NAMES={'gridcraft':'GridCraft','wordcraft':'WordCraft','deckcraft':'DeckCraft','cadcraft':'CADCraft','vectorcraft':'VectorCraft','photocraft':'PhotoCraft','pdfcraft':'PdfCraft','filmcraft':'FilmCraft','effectcraft':'EffectCraft','lightcraft':'LightCraft'}
def run(command):
    result=subprocess.run(command,capture_output=True,text=True)
    if result.returncode:raise RuntimeError(result.stderr or result.stdout)
    return result.stdout+result.stderr

def loadable_digest(data):
    # Gradle strips non-loaded symbols; runtime code/data must remain byte-for-byte identical.
    offset=struct.unpack_from('<Q',data,32)[0];size,count=struct.unpack_from('<HH',data,54);digest=hashlib.sha256()
    for index in range(count):
        p=struct.unpack_from('<IIQQQQQQ',data,offset+index*size)
        if p[0]!=1:continue
        digest.update(struct.pack('<IQQQ',p[1],p[3],p[5],p[6]))
        start=p[2]+(64 if p[2]==0 else 0)  # ELF section-table offsets change when stripped.
        digest.update(data[start:p[2]+p[5]])
    return digest.hexdigest()

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('projects',nargs='*',default=list(NAMES));parser.add_argument('--bundletool',type=Path,default=ROOT/'build/tools/bundletool-all-1.18.3.jar');args=parser.parse_args()
    sdk=Path(os.environ.get('ANDROID_HOME',Path.home()/'Library/Android/sdk'));tools=sdk/'build-tools/36.0.0';dist=ROOT/'dist/android';results={}
    for project in args.projects:
        title=NAMES[project];aab=dist/(title+'-release.aab');apk=dist/(title+'-release.apk')
        try:
            result=run(['java','-jar',str(args.bundletool),'validate','--bundle='+str(aab)]);(dist/(project+'-bundle-validation.log')).write_text(result)
            signed=run(['jarsigner','-verify',str(aab)]);assert 'jar verified.' in signed,'AAB JAR signature not verified'
            (dist/(project+'-aab-signature.log')).write_text(signed)
            native=(ROOT/'packaging/android/app/src'/project/'jniLibs/arm64-v8a/libcraft.so').read_bytes()
            assert native[:6]==b'\x7fELF\x02\x01' and struct.unpack_from('<H',native,18)[0]==183,'Expected ELF64 ARM64'
            assert (title+' Mobile').encode() in native,'Editor identity missing from native library'
            source=ROOT/'build/android-rust/aarch64-linux-android/release'/('lib'+project+'_android.so')
            assert source.read_bytes()==native,'Packaged native library differs from named editor output'
            off=struct.unpack_from('<Q',native,32)[0];size,count=struct.unpack_from('<HH',native,54)
            loads=[struct.unpack_from('<IIQQQQQQ',native,off+i*size) for i in range(count) if struct.unpack_from('<I',native,off+i*size)[0]==1]
            assert loads and all(p[-1]>=16384 and p[2]%16384==p[3]%16384 for p in loads),'ELF requires 16 KiB alignment'
            with zipfile.ZipFile(aab) as z:
                assert z.testzip() is None
                assert not any(n.lower().endswith(('.jks','.keystore','key.properties')) for n in z.namelist()),'Private signing material included in bundle'
                packed=z.read('base/lib/arm64-v8a/libcraft.so')
                assert loadable_digest(packed)==loadable_digest(native),'Packaged code/data differs from named editor source'
                assert (title+' Mobile').encode() in packed,'Packaged editor identity differs'
                assert any(n.startswith('base/assets/Licenses/') for n in z.namelist())
                assert 'base/manifest/AndroidManifest.xml' in z.namelist() and 'BundleConfig.pb' in z.namelist()
            certificate=run([str(tools/'apksigner'),'verify','--verbose','--print-certs',str(apk)])
            (dist/(project+'-apk-signature.log')).write_text(certificate)
            sha=re.search(r'certificate SHA-256 digest: ([0-9a-f]+)',certificate)
            assert sha,'APK certificate fingerprint missing'
            badging=run([str(tools/'aapt2'),'dump','badging',str(apk)])
            assert "package: name='com.duidev.craftmobile."+project+"'" in badging
            assert re.search(r"(?:minSdkVersion|sdkVersion):'29'",badging) and "targetSdkVersion:'36'" in badging, "Unexpected Android SDK target"
            assert "native-code: 'arm64-v8a'" in badging
            assert 'application-debuggable' not in badging
            assert "launchable-activity: name='com.duidev.craftmobile.CraftActivity'" in badging
            run([str(tools/'zipalign'),'-c','-P','16','4',str(apk)])
            with zipfile.ZipFile(apk) as z:
                assert z.read('lib/arm64-v8a/libcraft.so')==packed
                assert not any(n.lower().endswith(('.jks','.keystore','key.properties')) for n in z.namelist()),'Private signing material included in APK'
            results[project]={'status':'verified','application_id':'com.duidev.craftmobile.'+project,'abi':'arm64-v8a','min_sdk':29,'target_sdk':36,'signed_aab':True,'signed_apk':True,'native_16kb_alignment':True,'apk_16kb_alignment':True,'bundletool_valid':True,'release_non_debuggable':True,'native_loadable_sha256':loadable_digest(packed),'native_sha256':hashlib.sha256(packed).hexdigest(),'aab_sha256':hashlib.sha256(aab.read_bytes()).hexdigest(),'apk_sha256':hashlib.sha256(apk.read_bytes()).hexdigest(),'certificate_sha256':sha.group(1)}
            print('VERIFIED',title,flush=True)
        except (OSError,RuntimeError,AssertionError,KeyError,zipfile.BadZipFile) as e:
            results[project]={'status':'failed','error':str(e)};print('FAILED',title,str(e),flush=True)
        (dist/'validation.json').write_text(json.dumps(results,indent=2)+'\n')
    certificates={v.get('certificate_sha256') for v in results.values() if v['status']=='verified'}
    if len(certificates)>1:raise SystemExit('Release APKs have inconsistent signing certificates')
    return 1 if any(v['status']=='failed' for v in results.values()) else 0
if __name__=='__main__':raise SystemExit(main())
