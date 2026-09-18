#!/usr/bin/env python3
"""Build a relocatable Qt runtime archive and a Debian package from the release build."""
import json,os,re,shutil,subprocess,tarfile,hashlib
from pathlib import Path
root=Path(__file__).resolve().parents[1]
metadata=json.loads(subprocess.check_output(['cargo','metadata','--locked','--format-version','1'],cwd=root))
version=next(p['version'] for p in metadata['packages'] if p['id']==metadata['resolve']['root'])
qt=Path(os.environ.get('QT_PREFIX',str(Path.home()/'.local/share/Qt/6.8.3/gcc_64'))).resolve()
binary=root/'build/calc'
assert binary.is_file(),'Run ./build.sh first'
dist=root/'dist';dist.mkdir(exist_ok=True)
name=f'calc-{version}-linux-x86_64';bundle=dist/name
if bundle.exists():raise SystemExit(f'{bundle} already exists; use a fresh dist directory to avoid overwriting a release')
for d in ['lib','qml','plugins','licenses/qt','licenses/rust']:(bundle/d).mkdir(parents=True,exist_ok=True)
shutil.copy2(binary,bundle/'calc')
shutil.copy2(root/'packaging/AppRun',bundle/'AppRun');shutil.copy2(root/'packaging/install-bundle.sh',bundle/'install.sh')
for f in ['icon.svg','LICENSE','THIRD_PARTY_NOTICES.md']:shutil.copy2(root/f,bundle/f)
shutil.copytree(root/'licenses',bundle/'licenses',dirs_exist_ok=True)
(bundle/'VERSION').write_text(version+'\n')
(bundle/'qt.conf').write_text('[Paths]\nPrefix=.\nLibraries=lib\nPlugins=plugins\nQmlImports=qml\n')
(bundle/'README.txt').write_text('Calc '+version+'\nRun ./AppRun, or ./install.sh for a per-user installation.\nQt is bundled. Built on Linux x86_64 with glibc 2.35; tested on Pop!_OS 22.04 X11.\nSource and updates: https://github.com/HappyCatKitten/Calc\n')
imports=json.loads(subprocess.check_output([str(qt/'libexec/qmlimportscanner'),'-rootPath',str(root/'qml'),'-importPath',str(qt/'qml')]))
for mod in imports:
 if not mod.get('path') or not mod.get('relativePath'):continue
 src=Path(mod['path']);dst=bundle/'qml'/mod['relativePath'];dst.mkdir(parents=True,exist_ok=True)
 for f in src.iterdir():
  if f.is_file() and not f.name.endswith(('.qmltypes','.prl','.a')):shutil.copy2(f,dst/f.name)
for category in ['platforms','xcbglintegrations','wayland-decoration-client','wayland-graphics-integration-client','wayland-shell-integration','platforminputcontexts','imageformats']:
 src=qt/'plugins'/category
 if not src.exists():continue
 dst=bundle/'plugins'/category;dst.mkdir(parents=True,exist_ok=True)
 for f in src.glob('*.so'):
  if category=='platforms' and f.name not in ['libqxcb.so','libqwayland-egl.so','libqwayland-generic.so','libqoffscreen.so']:continue
  if category=='imageformats' and f.name!='libqsvg.so':continue
  shutil.copy2(f,dst/f.name)
env=dict(os.environ,LD_LIBRARY_PATH=str(qt/'lib'))
system_paths=set();copied=set();queue=[bundle/'calc',*bundle.rglob('*.so')]
while queue:
 f=queue.pop();output=subprocess.check_output(['ldd',str(f)],env=env,text=True)
 if 'not found' in output:raise RuntimeError(output)
 for line in output.splitlines():
  match=re.search(r'=> (/\S+)',line)
  if not match:continue
  path=Path(match.group(1));soname=line.strip().split()[0]
  if path.is_relative_to(qt):
   if soname not in copied:shutil.copy2(path,bundle/'lib'/soname);copied.add(soname);queue.append(bundle/'lib'/soname)
  else:system_paths.add(str(path))
# Ship Qt's SBOMs and embedded license/copyright text alongside the dynamic libraries.
for f in (qt/'sbom').glob('*.spdx.json'):
 if not f.name.startswith(('qtbase-','qtdeclarative-','qtwayland-','qtsvg-','qtshadertools-')):continue
 shutil.copy2(f,bundle/'licenses/qt'/f.name)
 doc=json.loads(f.read_text());texts=[]
 for item in doc.get('hasExtractedLicensingInfos',[]):texts.append(item.get('licenseId','')+'\n'+item.get('extractedText',''))
 (bundle/'licenses/qt'/(f.stem+'.licenses.txt')).write_text('\n\n'.join(texts))
registry=Path.home()/'.cargo/registry/src'
for package in metadata['packages']:
 if not (package.get('source') or '').startswith('registry+'):continue
 for crate in registry.glob(f'*/{package["name"]}-{package["version"]}'):
  dest=bundle/'licenses/rust'/crate.name;dest.mkdir(parents=True,exist_ok=True)
  for f in crate.iterdir():
   if f.is_file() and f.name.upper().startswith(('LICENSE','COPYING','NOTICE')):shutil.copy2(f,dest/f.name)
  break
# Debian dependencies come from the host package ownership of external libraries.
packages=set()
for value in system_paths:
 candidates=[value,str(Path(value).resolve())]
 candidates += [p.replace('/usr/lib/','/lib/') for p in candidates]
 for p in candidates:
  result=subprocess.run(['dpkg-query','-S',p],capture_output=True,text=True)
  if result.returncode==0:
   package=result.stdout.split(': ')[0].split(':')[0];packages.add(package);break
packages.discard('libc6');dependencies=['libc6 (>= 2.35)',*sorted(packages)]
(bundle/'runtime-manifest.json').write_text(json.dumps({'version':version,'qt':'6.8.3','bundled_libraries':sorted(copied),'external_packages':dependencies},indent=2)+'\n')
archive=dist/(name+'.tar.gz')
with tarfile.open(archive,'w:gz') as tf:tf.add(bundle,arcname=name)
debroot=dist/(name+'-deb-root');(debroot/'DEBIAN').mkdir(parents=True);(debroot/'opt').mkdir()
shutil.copytree(bundle,debroot/'opt/calc',symlinks=True)
for d in ['usr/bin','usr/share/applications','usr/share/icons/hicolor/scalable/apps']:(debroot/d).mkdir(parents=True)
(debroot/'usr/bin/calc').symlink_to('/opt/calc/AppRun')
(debroot/'usr/share/applications/calc.desktop').write_text('[Desktop Entry]\nType=Application\nName=Calc\nComment=A native calculator with a Brainfuck numeric engine\nExec=/opt/calc/AppRun\nIcon=io.github.HappyCatKitten.Calc\nTerminal=false\nCategories=Utility;Calculator;\nStartupWMClass=Calc\n')
shutil.copy2(root/'icon.svg',debroot/'usr/share/icons/hicolor/scalable/apps/io.github.HappyCatKitten.Calc.svg')
size=sum(p.stat().st_size for p in (debroot/'opt').rglob('*') if p.is_file())//1024
(debroot/'DEBIAN/control').write_text(f'Package: calc\nVersion: {version}\nArchitecture: amd64\nMaintainer: HappyCatKitten <1046264+HappyCatKitten@users.noreply.github.com>\nSection: utils\nPriority: optional\nInstalled-Size: {size}\nDepends: {", ".join(dependencies)}\nHomepage: https://github.com/HappyCatKitten/Calc\nDescription: Native calculator with a real Brainfuck numeric engine\n Rust-hosted Brainfuck arithmetic and scientific functions with a Qt/QML UI.\n')
deb=dist/f'calc_{version}_amd64.deb'
subprocess.run(['dpkg-deb','--root-owner-group','--build',str(debroot),str(deb)],check=True)
(dist/'SHA256SUMS').write_text(''.join(hashlib.sha256(p.read_bytes()).hexdigest()+'  '+p.name+'\n' for p in [archive,deb]))
print(json.dumps({'archive':str(archive),'deb':str(deb),'qt_libraries':len(copied),'dependencies':dependencies},indent=2))
