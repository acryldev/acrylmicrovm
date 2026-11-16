# Modified by ACRYL, 2026
"""
Unit tests for acrylmicrovm error types (no VM required).

Tests the error hierarchy and exception behavior.
"""

import pytest

from acrylmicrovm import errors
from acrylmicrovm.errors import AcrylMicrovmError, ExecError, ParseError, TimeoutError


class TestAcrylMicrovmError:
    """Test base AcrylMicrovmError exception."""

    def test_is_exception(self):
        """Test that AcrylMicrovmError is an Exception."""
        assert issubclass(AcrylMicrovmError, Exception)

    def test_can_raise(self):
        """Test that AcrylMicrovmError can be raised."""
        with pytest.raises(AcrylMicrovmError):
            raise AcrylMicrovmError("test error")

    def test_message(self):
        """Test that AcrylMicrovmError stores message."""
        err = AcrylMicrovmError("test message")
        assert str(err) == "test message"

    def test_empty_message(self):
        """Test AcrylMicrovmError with empty message."""
        err = AcrylMicrovmError()
        assert str(err) == ""


class TestExecError:
    """Test ExecError exception."""

    def test_inherits_acrylmicrovm_error(self):
        """Test that ExecError inherits from AcrylMicrovmError."""
        assert issubclass(ExecError, AcrylMicrovmError)

    def test_attributes(self):
        """Test that ExecError stores command, exit_code, and stderr."""
        err = ExecError(command="ls -la", exit_code=1, stderr="file not found")
        assert err.command == "ls -la"
        assert err.exit_code == 1
        assert err.stderr == "file not found"

    def test_message_format(self):
        """Test ExecError message format."""
        err = ExecError(command="cat /nonexistent", exit_code=2, stderr="No such file")
        assert "cat /nonexistent" in str(err)
        assert "2" in str(err)
        assert "No such file" in str(err)

    def test_can_catch_as_acrylmicrovm_error(self):
        """Test that ExecError can be caught as AcrylMicrovmError."""
        with pytest.raises(AcrylMicrovmError):
            raise ExecError("cmd", 1, "error")

    def test_negative_exit_code(self):
        """Test ExecError with negative exit code (signal termination)."""
        err = ExecError(command="sleep 100", exit_code=-9, stderr="killed")
        assert err.exit_code == -9

    def test_empty_stderr(self):
        """Test ExecError with empty stderr."""
        err = ExecError(command="false", exit_code=1, stderr="")
        assert err.stderr == ""


class TestTimeoutError:
    """Test TimeoutError exception."""

    def test_inherits_acrylmicrovm_error(self):
        """Test that TimeoutError inherits from AcrylMicrovmError."""
        assert issubclass(TimeoutError, AcrylMicrovmError)

    def test_can_raise(self):
        """Test that TimeoutError can be raised."""
        with pytest.raises(TimeoutError):
            raise TimeoutError("operation timed out")

    def test_can_catch_as_acrylmicrovm_error(self):
        """Test that TimeoutError can be caught as AcrylMicrovmError."""
        with pytest.raises(AcrylMicrovmError):
            raise TimeoutError("timeout")


class TestParseError:
    """Test ParseError exception."""

    def test_inherits_acrylmicrovm_error(self):
        """Test that ParseError inherits from AcrylMicrovmError."""
        assert issubclass(ParseError, AcrylMicrovmError)

    def test_can_raise(self):
        """Test that ParseError can be raised."""
        with pytest.raises(ParseError):
            raise ParseError("invalid JSON output")

    def test_can_catch_as_acrylmicrovm_error(self):
        """Test that ParseError can be caught as AcrylMicrovmError."""
        with pytest.raises(AcrylMicrovmError):
            raise ParseError("parse error")


class TestErrorHierarchy:
    """Test the complete error hierarchy."""

    def test_all_errors_inherit_from_base(self):
        """Test that all error types inherit from AcrylMicrovmError."""
        assert issubclass(ExecError, AcrylMicrovmError)
        assert issubclass(TimeoutError, AcrylMicrovmError)
        assert issubclass(ParseError, AcrylMicrovmError)

    def test_all_errors_are_exceptions(self):
        """Test that all error types are Exceptions."""
        assert issubclass(AcrylMicrovmError, Exception)
        assert issubclass(ExecError, Exception)
        assert issubclass(TimeoutError, Exception)
        assert issubclass(ParseError, Exception)

    def test_catch_all_with_base_class(self):
        """Test catching all acrylmicrovm errors with base class."""
        errors = [
            AcrylMicrovmError("base"),
            ExecError("cmd", 1, "err"),
            TimeoutError("timeout"),
            ParseError("parse"),
        ]

        for error in errors:
            try:
                raise error
            except AcrylMicrovmError as e:
                assert e is error


NATIVE_ERRORS = errors._NativeError.__subclasses__()


class TestNativeErrors:
    """The classes a failure from the native runtime is raised as."""

    def test_there_is_one_per_runtime_error_class(self):
        # `AcrylMicrovmError::http()` (src/shared/src/errors.rs) names 18 classes.
        assert len(NATIVE_ERRORS) == 18

    @pytest.mark.parametrize("cls", NATIVE_ERRORS, ids=lambda cls: cls.__name__)
    def test_is_caught_by_older_runtime_error_handlers(self, cls):
        assert issubclass(cls, AcrylMicrovmError)
        assert issubclass(cls, RuntimeError)

    @pytest.mark.parametrize("cls", NATIVE_ERRORS, ids=lambda cls: cls.__name__)
    def test_is_exported_from_the_package(self, cls):
        import acrylmicrovm

        assert getattr(acrylmicrovm, cls.__name__) is cls

    def test_each_names_its_own_code(self):
        codes = [cls.code for cls in NATIVE_ERRORS]
        assert len(set(codes)) == len(codes), codes


class TestErrorExports:
    """Test that errors are properly exported."""

    def test_errors_in_module(self):
        """Test that errors are exported from acrylmicrovm module."""
        import acrylmicrovm

        assert hasattr(acrylmicrovm, "AcrylMicrovmError")
        assert hasattr(acrylmicrovm, "ExecError")
        assert hasattr(acrylmicrovm, "TimeoutError")
        assert hasattr(acrylmicrovm, "ParseError")

    def test_errors_from_errors_module(self):
        """Test that errors can be imported from errors module."""
        from acrylmicrovm.errors import AcrylMicrovmError, ExecError, ParseError, TimeoutError

        assert AcrylMicrovmError is not None
        assert ExecError is not None
        assert TimeoutError is not None
        assert ParseError is not None


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
