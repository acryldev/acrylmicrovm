// Modified by ACRYL, 2026
package acrylmicrovm

/*
#include "bridge.h"
*/
import "C"

import "unsafe"

//export goAcrylMicrovmOnCheckpointInfo
func goAcrylMicrovmOnCheckpointInfo(info *C.CCheckpointInfo, nativeError *C.CAcrylMicrovmError, userData unsafe.Pointer) {
	dispose := func(value **C.CCheckpointInfo) {
		if value != nil {
			C.acrylmicrovm_free_checkpoint_info(*value)
		}
	}
	handle := ptrToHandle(userData)
	if handle == 0 {
		dispose(&info)
		return
	}
	if !claimOrFreePayload(handle, &info, dispose) {
		return
	}
	defer handle.Delete()
	result, ok := handle.Value().(chan handleResult[*C.CCheckpointInfo])
	if !ok {
		dispose(&info)
		return
	}
	result <- handleResult[*C.CCheckpointInfo]{value: info, err: errorFromCError(nativeError)}
}

//export goAcrylMicrovmOnCheckpointList
func goAcrylMicrovmOnCheckpointList(list *C.CCheckpointList, nativeError *C.CAcrylMicrovmError, userData unsafe.Pointer) {
	dispose := func(value **C.CCheckpointList) {
		if value != nil {
			C.acrylmicrovm_free_checkpoint_list(*value)
		}
	}
	handle := ptrToHandle(userData)
	if handle == 0 {
		dispose(&list)
		return
	}
	if !claimOrFreePayload(handle, &list, dispose) {
		return
	}
	defer handle.Delete()
	result, ok := handle.Value().(chan handleResult[*C.CCheckpointList])
	if !ok {
		dispose(&list)
		return
	}
	result <- handleResult[*C.CCheckpointList]{value: list, err: errorFromCError(nativeError)}
}
