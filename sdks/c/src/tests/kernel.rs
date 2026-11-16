// Modified by ACRYL, 2026
use super::*;

#[test]
fn kernel_path_setter_validates_and_copies_the_path() {
    unsafe {
        let path = unique_test_home("kernel-file");
        std::fs::write(&path, b"\x7fELFfixture").unwrap();
        let advanced = new_test_advanced_options();
        let mut error = FFIError::default();
        let input = CString::new(path.to_str().unwrap()).unwrap();
        assert_eq!(
            acrylmicrovm_advanced_options_set_kernel_path(advanced, input.as_ptr(), &mut error),
            AcrylMicrovmErrorCode::Ok
        );
        drop(input);
        assert_eq!((*advanced).options.kernel.as_ref().unwrap().path, path);
        for (invalid, expected) in [
            (ptr::null(), AcrylMicrovmErrorCode::InvalidArgument),
            (c"/kernel-\xff".as_ptr(), AcrylMicrovmErrorCode::InvalidArgument),
            (c"relative/kernel".as_ptr(), AcrylMicrovmErrorCode::Config),
            (c"/missing/kernel.bin".as_ptr(), AcrylMicrovmErrorCode::Config),
        ] {
            assert_eq!(
                acrylmicrovm_advanced_options_set_kernel_path(advanced, invalid, &mut error),
                expected
            );
            assert_eq!(error.code, expected);
            acrylmicrovm_error_free(&mut error);
            assert_eq!((*advanced).options.kernel.as_ref().unwrap().path, path);
        }
        assert_eq!(
            acrylmicrovm_advanced_options_set_kernel_path(
                ptr::null_mut(),
                c"/missing".as_ptr(),
                &mut error
            ),
            AcrylMicrovmErrorCode::InvalidArgument
        );
        acrylmicrovm_error_free(&mut error);
        acrylmicrovm_advanced_options_free(advanced);
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn kernel_runtime_constructor_applies_the_feature_gate() {
    for features in [c"", c"custom-kernel"] {
        unsafe {
            let home = unique_test_home("kernel-runtime");
            let c_home = CString::new(home.to_str().unwrap()).unwrap();
            let mut handle = MaybeUninit::<*mut RuntimeHandle>::uninit();
            let mut error = FFIError::default();
            assert_eq!(
                acrylmicrovm_runtime_new_experimental(
                    c_home.as_ptr(),
                    ptr::null(),
                    0,
                    features.as_ptr(),
                    handle.as_mut_ptr(),
                    &mut error
                ),
                AcrylMicrovmErrorCode::Ok
            );
            let handle = handle.assume_init();
            assert!(!handle.is_null());
            let mut options = acrylmicrovm::BoxOptions::default();
            options.advanced.kernel = Some(
                acrylmicrovm::experimental::custom_kernel::KernelOptions::new("/missing/kernel.bin"),
            );
            let error = (*handle)
                .tokio_rt
                .block_on((*handle).runtime.create(options, None))
                .err()
                .unwrap();
            let expected = if features == c"" {
                "disabled"
            } else {
                "regular file"
            };
            assert!(error.to_string().contains(expected), "{error}");
            acrylmicrovm_runtime_free(handle);
            std::fs::remove_dir_all(home).unwrap();
        }
    }
}

#[test]
fn kernel_runtime_constructor_rejects_unknown_features_before_initializing() {
    assert_invalid_runtime_features(
        c"custom-kernel,unknown",
        AcrylMicrovmErrorCode::Config,
        "unknown feature",
    );
}

#[test]
fn kernel_runtime_constructor_rejects_empty_features_before_initializing() {
    assert_invalid_runtime_features(
        c"custom-kernel,",
        AcrylMicrovmErrorCode::Config,
        "empty feature name",
    );
}

#[test]
fn kernel_runtime_constructor_rejects_invalid_utf8_before_initializing() {
    assert_invalid_runtime_features(
        c"custom-kernel,\xff",
        AcrylMicrovmErrorCode::InvalidArgument,
        "features is not valid UTF-8",
    );
}

fn assert_invalid_runtime_features(
    features: &CStr,
    expected_code: AcrylMicrovmErrorCode,
    expected: &str,
) {
    unsafe {
        let mut runtime = ptr::null_mut();
        let mut error = FFIError::default();
        assert_eq!(
            acrylmicrovm_runtime_new_experimental(
                ptr::null(),
                ptr::null(),
                0,
                features.as_ptr(),
                &mut runtime,
                &mut error
            ),
            expected_code
        );
        assert_eq!(error.code, expected_code);
        assert!(runtime.is_null());
        let message = CStr::from_ptr(error.message).to_str().unwrap().to_owned();
        acrylmicrovm_error_free(&mut error);
        assert!(message.contains(expected), "{message}");
        assert!(!message.contains("ACRYLMICROVM_EXPERIMENTAL"), "{message}");
    }
}
