#!/usr/bin/env python3
"""Exercise the themed native UI under Xvfb and capture real screenshots."""
import os, subprocess as sp, time, tempfile, json
from pathlib import Path
root=Path(__file__).resolve().parents[1]
display=next(f':{n}' for n in range(120,150) if not Path(f'/tmp/.X11-unix/X{n}').exists())
tmp=tempfile.TemporaryDirectory(); env=dict(os.environ,DISPLAY=display,XDG_CONFIG_HOME=tmp.name+'/config',XDG_DATA_HOME=tmp.name+'/data',QT_QPA_PLATFORM='xcb')
conf=Path(tmp.name+'/config/HappyCatKitten/Brainfuck Calculator.conf');conf.parent.mkdir(parents=True);conf.write_text('[General]\ncatacalc=true\nhistoryOpen=true\nscientific=false\nonTop=false\n')
xv=sp.Popen(['Xvfb',display,'-screen','0','1400x1200x24']);time.sleep(.4)
log=open(tmp.name+'/log','w');app=None
try:
 def x(*args):return sp.check_output(['xdotool',*args],env=env,text=True).strip()
 def launch():
  global app
  app=sp.Popen([os.environ.get('CATACALC_TEST_LAUNCHER',str(root/'run.sh'))],env=env,stdout=log,stderr=log);time.sleep(1)
  if app.poll() is not None:raise RuntimeError(Path(tmp.name+'/log').read_text())
  w=x('search','--onlyvisible','--name','^Brainfuck Calculator$').splitlines()[-1];x('windowfocus',w);return w
 def calc(expr):x('key','ctrl+l');x('type','--clearmodifiers',expr);x('key','Return');time.sleep(.15)
 def capture(name):
  x('mousemove','1300','1100');time.sleep(.25);sp.run(['import','-display',display,'-window',w,str(root/'docs/screenshots'/name)],check=True)
 def click(dx,dy):x('mousemove','--window',w,str(round(dx*.7)),str(round(dy*.7)));x('click','1');time.sleep(.15)
 def history():return json.loads((Path(tmp.name)/'data/brainfuck-calculator/history.json').read_text())
 w=launch();calc('128*4');calc('200+10%');calc('1234567.89*2');assert history()[0]['result']=='2469135.78';capture('catacalc.png')
 click(88,90);capture('catacalc-options.png');x('key','Escape')
 # Filter history, recall the filtered entry and calculate from its result.
 click(1040,338);x('type','128');click(1030,470);x('key','Tab');click(740,856);click(348,956);click(742,1008);assert history()[0]['result']=='514',history()[0]
 click(1040,338);x('key','ctrl+a');x('key','BackSpace');x('key','Escape')
 # Use actual mouse hit targets, not just keyboard input.
 click(153,757);click(740,856);click(348,956);click(742,1008);assert history()[0]['result']=='9',history()[0]
 x('key','ctrl+shift+s');time.sleep(.2);assert 'HEIGHT=973' in x('getwindowgeometry','--shell',w)
 calc('sin(30)+sqrt(81)');assert history()[0]['result']=='9.5';capture('catacalc-scientific.png')
 x('key','ctrl+h');time.sleep(.2);assert 'WIDTH=629' in x('getwindowgeometry','--shell',w);capture('catacalc-compact.png')
 app.terminate();app.wait();w=launch();assert 'WIDTH=629' in x('getwindowgeometry','--shell',w);assert 'HEIGHT=973' in x('getwindowgeometry','--shell',w)
 x('key','ctrl+h');x('key','ctrl+shift+s');time.sleep(.2)
 click(88,90);x('mousemove','--window',w,'180','145');x('click','1');time.sleep(.2);assert 'HEIGHT=584' in x('getwindowgeometry','--shell',w)
 x('key','Escape');calc('12*12');assert history()[0]['result']=='144';capture('catacalc-mode-off.png')
 app.terminate();app.wait();log.flush();logs=Path(tmp.name+'/log').read_text();print(logs);assert not logs.strip(),logs
 print('PASS: mouse keypad, editable expressions, arithmetic, scientific keys, history resizing, restart persistence, theme off and clean Qt log')
finally:
 if app and app.poll() is None:app.terminate();app.wait()
 xv.terminate();xv.wait()
