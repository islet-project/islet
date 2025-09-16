#!/usr/bin/env python3

import os
from config import *

PREBUILT_EDK2 = os.path.join(PREBUILT, "FVP_AARCH64_EFI.fd")
PREBUILT_GRUB = os.path.join(PREBUILT, "bootaa64.efi")

FVP_CONFIG = os.path.join(SCRIPT, ".config")
FVP_DIR = os.path.join(THIRD_PARTY, "fvp/Base_RevC_AEMvA_pkg/models/Linux64_GCC-9.3")
FVP_BIN = os.path.join(FVP_DIR, "./FVP_Base_RevC-2xAEMvA")
PLUGIN_PATH = os.path.join(THIRD_PARTY, "fvp/Base_RevC_AEMvA_pkg/plugins/Linux64_GCC-9.3/")
TRACE_LIB = os.path.join(PLUGIN_PATH, "TarmacTrace.so")
TOGGLE_LIB = os.path.join(PLUGIN_PATH, "ToggleMTIPlugin.so")
SVE_LIB = os.path.join(PLUGIN_PATH, "ScalableVectorExtension.so")
