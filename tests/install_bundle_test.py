"""Exercise the real archive installer with isolated homes and running ELF files."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


class BundleInstallTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="calc-install-test-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.bundle = self.root / "bundle"
        self.bundle.mkdir()
        source = Path(__file__).resolve().parents[1] / "packaging/install-bundle.sh"
        shutil.copy2(source, self.bundle / "install.sh")
        (self.bundle / "VERSION").write_text("test-version\n")
        (self.bundle / "AppRun").write_text('#!/bin/sh\nexec "$(dirname "$0")/calc" "$@"\n')
        (self.bundle / "AppRun").chmod(0o755)
        (self.bundle / "icon.svg").write_text('<svg xmlns="http://www.w3.org/2000/svg"/>')
        self.compile(self.bundle / "calc", "#include <unistd.h>\nint main(int argc, char **argv) { if (argc > 1) sleep(60); return 0; }\n")
        self.replacement = self.root / "replacement"
        self.compile(self.replacement, "int main(void) { return 0; }\n")
        self.env = dict(os.environ, HOME=str(self.root / "home"), XDG_DATA_HOME=str(self.root / "data"))
        self.installed = self.root / "data/calc/versions/test-version"

    def compile(self, target, code):
        subprocess.run(["cc", "-x", "c", "-", "-o", str(target)], input=code,
                       text=True, capture_output=True, check=True, timeout=30)

    def install(self, expected=0):
        result = subprocess.run(["bash", str(self.bundle / "install.sh")], env=self.env,
                                capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, expected, result.stderr)

    def test_reinstall_preserves_running_binary_user_data_and_previous_tree(self):
        self.install()
        history = self.root / "data/calc/history.json"
        history.write_text('["preserve history"]')
        process = subprocess.Popen(["sleep", "60"], executable=str(self.installed / "calc"))
        self.addCleanup(lambda: (process.terminate(), process.wait(timeout=5)) if process.poll() is None else None)
        old_inode = (self.installed / "calc").stat().st_ino
        shutil.copy2(self.replacement, self.bundle / "calc")
        self.install()
        self.assertIsNone(process.poll())
        self.assertEqual(Path(f"/proc/{process.pid}/exe").stat().st_ino, old_inode)
        self.assertNotEqual((self.installed / "calc").stat().st_ino, old_inode)
        self.assertEqual((self.installed / "calc").read_bytes(), self.replacement.read_bytes())
        self.assertEqual(subprocess.run(["true"], executable=str(self.installed / "calc"), timeout=5).returncode, 0)
        previous = list(self.installed.parent.glob("test-version.previous.*"))
        self.assertEqual(len(previous), 1)
        self.assertEqual((previous[0] / "calc").stat().st_ino, old_inode)
        self.assertEqual(history.read_text(), '["preserve history"]')
        # Running the installer from its installed location must remain harmless.
        subprocess.run(["bash", str(self.installed / "install.sh")], env=self.env,
                       check=True, capture_output=True, timeout=10)

    def test_failed_activation_restores_previous_installation(self):
        self.install()
        original = (self.installed / "calc").read_bytes()
        shim = self.root / "bin"
        shim.mkdir()
        mv = shim / "mv"
        mv.write_text('#!/bin/bash\nif [[ "$1" == -- ]]; then shift; fi\n'
                      'if [[ "$1" == */.test-version.install.* ]]; then exit 42; fi\n'
                      'exec /bin/mv -- "$@"\n')
        mv.chmod(0o755)
        self.env["PATH"] = str(shim) + os.pathsep + self.env["PATH"]
        shutil.copy2(self.replacement, self.bundle / "calc")
        self.install(expected=42)
        self.assertEqual((self.installed / "calc").read_bytes(), original)
        self.assertFalse(list(self.installed.parent.glob(".test-version.install.*")))


if __name__ == "__main__":
    unittest.main()
