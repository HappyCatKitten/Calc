#!/usr/bin/env python3
"""Capture real app windows with isolated settings/history. Needs Xvfb, xdotool, ImageMagick."""
import os,sys,subprocess,time,tempfile,json
from pathlib import Path
root=Path(__file__).resolve().parents[1]
launcher=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else root/'run.sh'
out=root/'docs/screenshots';out.mkdir(parents=True,exist_ok=True)
with tempfile.TemporaryDirectory(prefix='brainfuck-screenshots-') as scratch:
    temp=Path(scratch);display=next(f':{n}' for n in range(93,120) if not Path(f'/tmp/.X11-unix/X{n}').exists())
    env=dict(os.environ,DISPLAY=display,XDG_DATA_HOME=str(temp/'data'),XDG_CONFIG_HOME=str(temp/'config'),QT_QPA_PLATFORM='xcb')
    for key in ['LD_LIBRARY_PATH','QT_PLUGIN_PATH','QML_IMPORT_PATH','QML2_IMPORT_PATH','QT_QPA_PLATFORM_PLUGIN_PATH']:env.pop(key,None)
    xserver=subprocess.Popen(['Xvfb',display,'-screen','0','1200x1000x24','-nolisten','tcp'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    app=None
    try:
        time.sleep(.4)
        with (temp/'app.log').open('w') as log:
            app=subprocess.Popen([str(launcher)],env=env,stdout=log,stderr=log)
        def run(*args):return subprocess.check_output(args,env=env,text=True,stderr=subprocess.DEVNULL).strip()
        window=None
        for _ in range(60):
            if app.poll() is not None:raise RuntimeError((temp/'app.log').read_text())
            try:window=run('xdotool','search','--onlyvisible','--name','^Brainfuck Calculator$').splitlines()[-1];break
            except subprocess.CalledProcessError:time.sleep(.15)
        if not window:raise RuntimeError('App did not open a visible window')
        def x(*args):return run('xdotool',*args)
        def calculate(expression):
            x('key','--clearmodifiers','ctrl+l');x('type','--clearmodifiers',expression);x('key','Return');time.sleep(.15)
        def capture(name):
            x('mousemove','1100','900');time.sleep(.25);subprocess.run(['import','-display',display,'-window',window,str(out/name)],check=True)
        x('windowfocus','--sync',window)
        calculate('128*4');calculate('200+10%');calculate('1234567.89*2');capture('standard.png')
        x('key','ctrl+shift+s');calculate('sin(30)+sqrt(81)');capture('scientific.png')
        x('key','ctrl+shift+s');x('mousemove','--window',window,'28','21');x('click','1');capture('options.png')
        maps=Path(f'/proc/{app.pid}/maps').read_text()
        sdk=str(Path.home()/'.local/share/Qt')
        if launcher.name=='AppRun' and sdk in maps:raise RuntimeError('Bundle used the developer Qt SDK')
        history=json.loads((temp/'data/brainfuck-calculator/history.json').read_text())
        assert history[0]['result']=='9.5' and history[1]['result']=='2469135.78'
        print(json.dumps({'screenshots':[str(out/f) for f in ['standard.png','scientific.png','options.png']],'visible_window':True,'history_verified':True,'developer_qt_sdk_loaded':sdk in maps,'log':(temp/'app.log').read_text()},indent=2))
    finally:
        if app and app.poll() is None:app.terminate();app.wait(timeout=10)
        xserver.terminate();xserver.wait(timeout=10)
