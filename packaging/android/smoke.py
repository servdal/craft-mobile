#!/usr/bin/env python3
"""Install and launch release APKs on a dedicated emulator, saving startup evidence."""
from pathlib import Path
import argparse,json,os,re,subprocess,time
ROOT=Path(__file__).resolve().parents[2]
NAMES={'gridcraft':'GridCraft','wordcraft':'WordCraft','deckcraft':'DeckCraft','cadcraft':'CADCraft','vectorcraft':'VectorCraft','photocraft':'PhotoCraft','pdfcraft':'PdfCraft','filmcraft':'FilmCraft','effectcraft':'EffectCraft','lightcraft':'LightCraft'}
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--serial',required=True);p.add_argument('projects',nargs='*',default=list(NAMES));args=p.parse_args()
    if not args.serial.startswith('emulator-'):p.error('Use a dedicated emulator; this startup test force-stops each tested application')
    sdk=Path(os.environ.get('ANDROID_HOME',Path.home()/'Library/Android/sdk'));adb=[str(sdk/'platform-tools/adb'),'-s',args.serial];qa=ROOT/'dist/android/qa';qa.mkdir(parents=True,exist_ok=True);results={}
    def run(*command):return subprocess.run(adb+list(command),capture_output=True,check=True).stdout
    previous=None
    for project in args.projects:
        title=NAMES[project];bundle='com.duidev.craftmobile.'+project
        try:
            if previous:run('shell','am','force-stop',previous)
            run('install','-r',str(ROOT/'dist/android'/(title+'-release.apk')))
            run('shell','am','force-stop',bundle)
            launch=run('shell','am','start','-W','-n',bundle+'/com.duidev.craftmobile.CraftActivity').decode()
            (qa/(project+'-launch.txt')).write_text(launch)
            if 'Status: ok' not in launch:raise RuntimeError('Activity launch did not succeed')
            time.sleep(4)
            pid=run('shell','pidof',bundle).decode().strip().split()[0]
            log=run('logcat','-d','--pid='+pid,'-t','500').decode(errors='replace');(qa/(project+'-logcat.txt')).write_text(log)
            if re.search(r'FATAL EXCEPTION|Fatal signal|panicked at|Craft startup failed',log):raise RuntimeError('Application reported a fatal startup error')
            (qa/(project+'.png')).write_bytes(run('exec-out','screencap','-p'))
            results[project]={'status':'launched','pid':pid,'screenshot':str(qa/(project+'.png'))};previous=bundle
            print('LAUNCHED',title,flush=True)
        except (subprocess.CalledProcessError,RuntimeError,IndexError) as error:
            results[project]={'status':'failed','error':str(error)};print('FAILED',title,error,flush=True)
        (qa/'launch-status.json').write_text(json.dumps(results,indent=2)+'\n')
    if previous:run('shell','am','force-stop',previous)
    return 1 if any(v['status']=='failed' for v in results.values()) else 0
if __name__=='__main__':raise SystemExit(main())
