// Modified by ACRYL, 2026
// Bridge between Go-exported callbacks and the typed C callback pointers
// expected by the ACRYL microVMs C API. Each accessor returns the callback as the
// strongly-typed function-pointer typedef so cgo wrappers in any Go file in
// this package can pass `C.cbXxx()` directly into the SDK.

#include "bridge.h"

// Forward declarations for the //export'd Go callbacks. The signatures must
// match the cbindgen-generated typedefs exactly; the casts in the accessor
// bodies validate that at compile time.
extern void goAcrylMicrovmOnStdout(uint8_t const *data, size_t len, void *ud);
extern void goAcrylMicrovmOnStderr(uint8_t const *data, size_t len, void *ud);
extern void goAcrylMicrovmOnExit(int exit_code, void *ud);

extern void goAcrylMicrovmOnCreateBox(CBoxHandle *box, CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnBoxExport(char *archive_path, CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnRuntimeImport(CBoxHandle *box, CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnCheckpointInfo(CCheckpointInfo *info, CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnCheckpointList(CCheckpointList *list, CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnGetOrCreateBox(CBoxHandle *box, bool created, CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnGetBox(CBoxHandle *box, CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnStartBox(CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnStopBox(CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnRemoveBox(CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnCopy(CAcrylMicrovmError *err, void *ud);

extern void goAcrylMicrovmOnImagePull(CImagePullResult *res, CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnImageList(CImageInfoList *list, CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnImageGet(CImageDetail *detail, CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnImageRemove(CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnImageUsage(CImageUsage *usage, CAcrylMicrovmError *err, void *ud);

extern void goAcrylMicrovmOnVolume(CVolumeInfo *info, CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnVolumeList(CVolumeInfoList *list, CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnVolumeRemove(CAcrylMicrovmError *err, void *ud);

extern void goAcrylMicrovmOnRegistryCreate(CRegistryCredential *login, CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnRegistryList(CRegistryCredentialList *list, CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnRegistryRemove(CAcrylMicrovmError *err, void *ud);

extern void goAcrylMicrovmOnInfo(CBoxInfo *info, CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnInfoList(CBoxInfoList *list, CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnExecutionInfoList(CExecutionInfoList *list, CAcrylMicrovmError *err, void *ud);

extern void goAcrylMicrovmOnBoxMetrics(CBoxMetrics *m, CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnRuntimeMetrics(CRuntimeMetrics *m, CAcrylMicrovmError *err, void *ud);

extern void goAcrylMicrovmOnRuntimeShutdown(CAcrylMicrovmError *err, void *ud);

extern void goAcrylMicrovmOnExecutionWait(int exit_code, CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnExecutionKill(CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnExecutionSignal(CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnExecutionResize(CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnTunnelForwarderWait(CAcrylMicrovmError *err, void *ud);
extern void goAcrylMicrovmOnTunnelForwarderClose(CAcrylMicrovmError *err, void *ud);

extern void goAcrylMicrovmOnSSHStatus(CSshStatus *status, CAcrylMicrovmError *err, void *ud);

CBoxStdoutCb cbStdout(void) { return (CBoxStdoutCb)goAcrylMicrovmOnStdout; }
CBoxStderrCb cbStderr(void) { return (CBoxStderrCb)goAcrylMicrovmOnStderr; }
CBoxExitCb cbExit(void) { return (CBoxExitCb)goAcrylMicrovmOnExit; }

CBoxCreateBoxCb cbCreateBox(void) { return (CBoxCreateBoxCb)goAcrylMicrovmOnCreateBox; }
CBoxExportCb cbBoxExport(void) {
    return (CBoxExportCb)goAcrylMicrovmOnBoxExport;
}
CRuntimeImportCb cbRuntimeImport(void) {
    return (CRuntimeImportCb)goAcrylMicrovmOnRuntimeImport;
}

CCheckpointInfoCb cbCheckpointInfo(void) { return (CCheckpointInfoCb)goAcrylMicrovmOnCheckpointInfo; }
CCheckpointListCb cbCheckpointList(void) { return (CCheckpointListCb)goAcrylMicrovmOnCheckpointList; }
CBoxGetOrCreateBoxCb cbGetOrCreateBox(void) { return (CBoxGetOrCreateBoxCb)goAcrylMicrovmOnGetOrCreateBox; }
CBoxGetBoxCb cbGetBox(void) { return (CBoxGetBoxCb)goAcrylMicrovmOnGetBox; }
CBoxStartBoxCb cbStartBox(void) { return (CBoxStartBoxCb)goAcrylMicrovmOnStartBox; }
CBoxStopBoxCb cbStopBox(void) { return (CBoxStopBoxCb)goAcrylMicrovmOnStopBox; }
CBoxRemoveBoxCb cbRemoveBox(void) { return (CBoxRemoveBoxCb)goAcrylMicrovmOnRemoveBox; }
CBoxCopyCb cbCopy(void) { return (CBoxCopyCb)goAcrylMicrovmOnCopy; }

CBoxImagePullCb cbImagePull(void) { return (CBoxImagePullCb)goAcrylMicrovmOnImagePull; }
CBoxImageListCb cbImageList(void) { return (CBoxImageListCb)goAcrylMicrovmOnImageList; }
CBoxImageGetCb cbImageGet(void) { return (CBoxImageGetCb)goAcrylMicrovmOnImageGet; }
CBoxImageRemoveCb cbImageRemove(void) { return (CBoxImageRemoveCb)goAcrylMicrovmOnImageRemove; }
CBoxImageUsageCb cbImageUsage(void) { return (CBoxImageUsageCb)goAcrylMicrovmOnImageUsage; }

// Create and Get both deliver a single CVolumeInfo, so they share the
// goAcrylMicrovmOnVolume trampoline (cast to their distinct typedef).
CBoxVolumeCreateCb cbVolumeCreate(void) { return (CBoxVolumeCreateCb)goAcrylMicrovmOnVolume; }
CBoxVolumeGetCb cbVolumeGet(void) { return (CBoxVolumeGetCb)goAcrylMicrovmOnVolume; }
CBoxVolumeListCb cbVolumeList(void) { return (CBoxVolumeListCb)goAcrylMicrovmOnVolumeList; }
CBoxVolumeRemoveCb cbVolumeRemove(void) { return (CBoxVolumeRemoveCb)goAcrylMicrovmOnVolumeRemove; }

CBoxRegistryCreateCb cbRegistryCreate(void) { return (CBoxRegistryCreateCb)goAcrylMicrovmOnRegistryCreate; }
CBoxRegistryListCb cbRegistryList(void) { return (CBoxRegistryListCb)goAcrylMicrovmOnRegistryList; }
CBoxRegistryRemoveCb cbRegistryRemove(void) { return (CBoxRegistryRemoveCb)goAcrylMicrovmOnRegistryRemove; }

CBoxInfoCb cbInfo(void) { return (CBoxInfoCb)goAcrylMicrovmOnInfo; }
CBoxInfoListCb cbInfoList(void) { return (CBoxInfoListCb)goAcrylMicrovmOnInfoList; }
CExecutionInfoListCb cbExecutionInfoList(void) { return (CExecutionInfoListCb)goAcrylMicrovmOnExecutionInfoList; }

CBoxMetricsCb cbBoxMetrics(void) { return (CBoxMetricsCb)goAcrylMicrovmOnBoxMetrics; }
CRuntimeMetricsCb cbRuntimeMetrics(void) { return (CRuntimeMetricsCb)goAcrylMicrovmOnRuntimeMetrics; }

CRuntimeShutdownCb cbRuntimeShutdown(void) { return (CRuntimeShutdownCb)goAcrylMicrovmOnRuntimeShutdown; }

CExecutionWaitCb cbExecutionWait(void) { return (CExecutionWaitCb)goAcrylMicrovmOnExecutionWait; }
CExecutionKillCb cbExecutionKill(void) { return (CExecutionKillCb)goAcrylMicrovmOnExecutionKill; }
CExecutionSignalCb cbExecutionSignal(void) { return (CExecutionSignalCb)goAcrylMicrovmOnExecutionSignal; }
CExecutionResizeCb cbExecutionResize(void) { return (CExecutionResizeCb)goAcrylMicrovmOnExecutionResize; }
CTunnelForwarderWaitCb cbTunnelForwarderWait(void) { return (CTunnelForwarderWaitCb)goAcrylMicrovmOnTunnelForwarderWait; }
CTunnelForwarderCloseCb cbTunnelForwarderClose(void) { return (CTunnelForwarderCloseCb)goAcrylMicrovmOnTunnelForwarderClose; }

// All SSH operations deliver one CSshStatus, so they share the
// goAcrylMicrovmOnSSHStatus trampoline (cast to their distinct typedef).
CBoxSshConfigureCb cbSSHConfigure(void) { return (CBoxSshConfigureCb)goAcrylMicrovmOnSSHStatus; }
CBoxSshStatusCb cbSSHStatus(void) { return (CBoxSshStatusCb)goAcrylMicrovmOnSSHStatus; }
CBoxSshDisableCb cbSSHDisable(void) { return (CBoxSshDisableCb)goAcrylMicrovmOnSSHStatus; }
CBoxSshEnableCb cbSSHEnable(void) { return (CBoxSshEnableCb)goAcrylMicrovmOnSSHStatus; }
