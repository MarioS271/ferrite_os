// QEMU Wasm module configuration for FerriteOS.
// Loaded as a plain <script> before the ES-module that boots QEMU.
if (typeof Module === 'undefined') {
    var Module = {};
}

Module.arguments = [
    '-nographic',
    '-M', 'q35',
    '-m', '256M',
    '-accel', 'tcg,tb-size=500',
    '-vga', 'none',
    '-bios', '/pack-uefi/OVMF_CODE.fd',
    '-cdrom', '/pack-cdrom/ferrite_os.iso',
    '-nic', 'none',
    '-serial', 'mon:stdio',
];

// Use an absolute URL derived from this page's location: web workers
// resolve relative paths against the worker script's URL, not the page's,
// which would mangle "./assets/..." into ".../assets/assets/...".
const FERRITE_ASSET_BASE = (() => {
    const here = location.pathname.replace(/[^/]*$/, '');
    return location.origin + here + 'assets/';
})();

Module.locateFile = function (path) {
    return FERRITE_ASSET_BASE + path;
};

Module.mainScriptUrlOrBlob = FERRITE_ASSET_BASE + 'out.js';
Module.preRun = Module.preRun || [];
